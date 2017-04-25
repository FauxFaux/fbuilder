#![feature(io)]

extern crate yaml_rust;

use std::env;
use std::fs;
use std::io;

use std::collections::HashSet;

use yaml_rust::parser::*;

// magic:
use std::io::BufRead;
use std::io::Read;

#[derive(Eq, PartialEq, Debug)]
enum NextIs {
    Key,
    Package,
    Version,
    Type,
    Ignore,
}

#[derive(Eq, PartialEq, Debug, Hash)]
struct Package {
    name: String,
    version: String,
}

impl Package {
    fn new() -> Package {
        Package { name: String::new(), version: String::new() }
    }

    fn clear(&mut self) {
        self.name.clear();
        self.version.clear();
    }

    fn is_set(&self) -> bool {
        !self.name.is_empty() && !self.version.is_empty()
    }

    fn print(&self) {
        assert!(self.is_set());
        print!("{}:{}", self.name, self.version);
    }
}

struct FirstPass {
    depth: u8,
    source: Package,
    dep: Package,
    map_state: NextIs,
    ignored: HashSet<String>,
}

impl FirstPass {
    fn new(ignored: HashSet<String>) -> FirstPass {
        FirstPass {
            depth: 0,
            source: Package::new(),
            dep: Package::new(),
            map_state: NextIs::Key,
            ignored,
        }
    }

    fn relevant_field<F>(&mut self, f: F)
            where F: FnOnce(&mut Package) {
        match self.depth {
            0 => panic!("can't have a scalar outside of a map"),
            1 => {},
            2 => f(&mut self.source),
            3 => f(&mut self.dep),
            _ => unreachable!(),
        };
    }
}

impl EventReceiver for FirstPass {
    fn on_event(&mut self, ev: &Event) {
        match ev {
            &Event::StreamStart
                | &Event::DocumentStart
                | &Event::DocumentEnd
                | &Event::StreamEnd => {},
            &Event::MappingStart ( 0 ) => {
                assert!(self.depth < 3);
                self.depth += 1;
                self.map_state = NextIs::Key;
            },
            &Event::MappingEnd => {
                if 3 == self.depth && self.dep.is_set() && !self.ignored.contains(&self.dep.name) {
                    self.source.print();
                    print!("\t");
                    self.dep.print();
                    println!();
                }
                self.depth -= 1;
                self.dep.clear();
            }
            &Event::SequenceStart ( 0 ) => {},
            &Event::SequenceEnd => {}
            &Event::Scalar ( ref label, _, 0, None ) => {
                match self.map_state {
                    NextIs::Key => {
                        self.map_state = match label.as_str() {
                            "package" => NextIs::Package,
                            "version" => NextIs::Version,
                            "type" => NextIs::Type,
                            _ => NextIs::Ignore,
                        };
                    },
                    NextIs::Package => {
                        self.relevant_field(|pkg| pkg.name = label.clone());
                        self.map_state = NextIs::Key;
                    },
                    NextIs::Version => {
                        self.relevant_field(|pkg| pkg.version = label.clone());
                        self.map_state = NextIs::Key;
                    },
                    NextIs::Type => {
                        self.dep.clear();
                        self.map_state = NextIs::Key;
                    },
                    NextIs::Ignore => {
                        self.map_state = NextIs::Key;
                    }
                };
            }
            &ref a => panic!(format!("{:?}", a)),
        }
    }
}

fn load_list(path: &str) -> io::Result<HashSet<String>> {

    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut ret = HashSet::with_capacity(300);
    for line in reader.lines() {
        ret.insert(line?);
    }
    Ok(ret)
}

fn main() {
    let input_path = env::args().nth(1).expect("first argument: input file");
    let ignored = load_list(
        env::args().nth(2).expect("second argument: ignored package list").as_str())
        .expect("reading ignored file");

    let file = fs::File::open(input_path).expect("input file must be readable");
    let mut parser = Parser::new(io::BufReader::new(file).chars().map(|r| r.expect("file read as utf-8")));
    let mut pass = FirstPass::new(ignored);
    parser.load(&mut pass, false).expect("yaml parse successs");
}

