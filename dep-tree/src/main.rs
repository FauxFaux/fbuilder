extern crate yaml_rust;

use std::env;
use std::fs;
use std::io;

use std::collections::HashMap;
use std::collections::HashSet;

// magic:
use std::io::BufRead;

fn load() -> io::Result<(HashSet<String>, HashMap<String, HashSet<String>>)> {
    let input_path = env::args().nth(1).expect("first argument: input file");
    let file = io::BufReader::new(fs::File::open(input_path)?);
    let mut key: String = "".to_string();
    let mut set: HashSet<String> = HashSet::new();
    let mut map: HashMap<String, HashSet<String>> = HashMap::with_capacity(30_000);
    let mut all = HashSet::new();
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
        if key.bytes().nth(0).expect("key must have been seen") == b'_' {
            continue;
        }

        let pkg = line[3..].to_string();
        set.insert(pkg.clone());
        all.insert(pkg);
    }

    Ok((all, map))
}

fn main() {
    let (all, map) = load().expect("loading file");
    println!("{}", map.len());
    println!("{}", all.len());
    println!("{:?}", map.keys().nth(0));
}
