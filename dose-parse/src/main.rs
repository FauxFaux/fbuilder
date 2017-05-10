#![feature(io)]
#![feature(retain_hash_collection)]
#![feature(vec_remove_item)]

extern crate clap;
extern crate yaml_rust;

use std::fs;
use std::io;

use std::collections::HashSet;
use std::collections::HashMap;

use clap::Arg;
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
    Arch,
    Ignore,
}

#[derive(Eq, PartialEq, Debug, Hash)]
struct Package {
    name: String,
    version: String,
    arch: String,
}

impl Package {
    fn new() -> Package {
        Package {
            name: String::new(),
            version: String::new(),
            arch: String::new(),
        }
    }

    fn clear(&mut self) {
        self.name.clear();
        self.version.clear();
    }

    fn is_set(&self) -> bool {
        !self.name.is_empty() && !self.version.is_empty()
    }

    fn to_string(&self, version: bool) -> String {
        assert!(self.is_set());
        if version {
            format!("{}:{}:{}", self.name, self.version, self.arch)
        } else {
            self.name.clone()
        }
    }
}

struct FirstPass {
    versions: bool,
    depth: u8,
    source: Package,
    dep: Package,
    map_state: NextIs,
    ignored: HashSet<String>,
    deps: Vec<String>,
    packages: HashMap<String, Vec<String>>,
}

impl FirstPass {
    fn new(versions: bool, ignored: HashSet<String>) -> FirstPass {
        FirstPass {
            versions,
            depth: 0,
            source: Package::new(),
            dep: Package::new(),
            map_state: NextIs::Key,
            ignored,
            deps: Vec::with_capacity(200),
            packages: HashMap::with_capacity(30_000),
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
                    self.deps.push(self.dep.to_string(self.versions));
                }
                self.depth -= 1;
                self.dep.clear();
            }
            &Event::SequenceStart ( 0 ) => {},
            &Event::SequenceEnd => {
                if 2 == self.depth {
                    self.packages.insert(self.source.to_string(false), self.deps.clone());
                    self.deps.clear();
                }
            }
            &Event::Scalar ( ref label, _, 0, None ) => {
                match self.map_state {
                    NextIs::Key => {
                        self.map_state = match label.as_str() {
                            "package" => NextIs::Package,
                            "version" => NextIs::Version,
                            "type" => NextIs::Type,
                            "architecture" => NextIs::Arch,
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
                    NextIs::Arch => {
                        self.relevant_field(|pkg| pkg.arch = label.clone());
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

fn real_main() -> u8 {
    let matches = clap::App::new("dose-parse")
        .arg(Arg::with_name("include-versions")
             .long("include-versions")
             .help("output pkg:version:arch strings"))
        .arg(Arg::with_name("INPUT")
             .help("dose output file to read")
             .required(true))
        .arg(Arg::with_name("excluded")
             .long("excluded")
             .takes_value(true)
             .help("exclude packages from processing (memory hack only)"))
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap();
    let ignored =
        if let Some(path) = matches.value_of("excluded") {
            load_list(path).expect("loading excluded file")
        } else {
            HashSet::new()
        };

    let versions = matches.is_present("include-versions");

    if versions != ignored.is_empty() {
        println!("--include-versions and --excluded are incompatible");
        return 2;
    }

    let file = fs::File::open(input_path).expect("input file must be readable");
    let mut parser = Parser::new(io::BufReader::new(file).chars().map(|r| r.expect("file read as utf-8")));
    let mut pass = FirstPass::new(versions, ignored.clone());
    parser.load(&mut pass, false).expect("yaml parse successs");
    let packages = pass.packages;

    let mut extra_essential: HashSet<String> = packages.values()
        .nth(0).expect("at least one package")
        .iter().map(|s| s.clone()).collect();

    for dep in packages.values() {
        extra_essential.retain(|pkg| dep.contains(pkg));
    }

    for ignoree in ignored {
        extra_essential.insert(ignoree);
    }

    let mut sorted: Vec<&String> = extra_essential.iter().collect();
    sorted.sort();

    println!("_essential:");
    for extra in sorted {
        println!(" - {}", extra);
    }

    let mut keys: Vec<&String> = packages.keys().collect();
    keys.sort();

    for src in keys.iter() {
        println!("{}:", src);

        let mut new = packages[src.as_str()].clone();
        for extra in extra_essential.iter() {
            new.remove_item(&extra);
        }
        new.sort();
        for item in new {
            println!(" - {}", item);
        }
    }

    return 0;
}

fn main() {
    std::process::exit(real_main() as i32);
}
