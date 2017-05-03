extern crate yaml_rust;

use std::env;
use std::fs;
use std::io;

use std::collections::HashMap;
use std::collections::HashSet;

// magic:
use std::io::BufRead;

struct StringPool {
    pool: HashMap<String, String>,
}

impl StringPool {
    fn new() -> StringPool {
        StringPool {
            pool: HashMap::with_capacity(30_000),
        }
    }

    fn fixup(&mut self, s: &str) -> &String {
        &*self.pool.entry(s.to_string()).or_insert_with(|| s.to_string())
    }
}

fn load<'a>() -> io::Result<HashMap<String, HashSet<&'a String>>> {
    let input_path = env::args().nth(1).expect("first argument: input file");
    let file = io::BufReader::new(fs::File::open(input_path)?);
    let mut key: String = "".to_string();
    let mut set: HashSet<&String> = HashSet::new();
    let mut map: HashMap<String, HashSet<&String>> = HashMap::with_capacity(30_000);
    let mut all = StringPool::new();
    for line in file.lines() {
        let line = line?;
        if line.ends_with(":") {
            if !set.is_empty() {
                map.insert(key, set.clone());
                set.clear();
            }
            key = line[0..line.len()-1].to_string();
            continue;
        }
        assert_eq!(" - ", &line[0..3]);
        set.insert(all.fixup(&line[3..]));
    }

    Ok(map)
}

fn main() {
    let map = load().expect("loading file");
    println!("{}", map.len());
    println!("{:?}", map.keys().nth(0));
}
