use std::env;
use std::fs;
use std::io;

use std::collections::HashMap;
use std::collections::HashSet;

// magic:
use std::io::BufRead;

type PkgId = u16;

struct PkgIdLookup {
    cache: HashMap<String, PkgId>,
}

impl PkgIdLookup {
    fn id_of(&mut self, pkg: &str) -> PkgId {
        let len = self.cache.len();

        // TODO: how to reference PkgId here?
        if len >= std::u16::MAX as usize {
            panic!("too many packages!");
        }

        *self.cache.entry(pkg.to_string()).or_insert(len as PkgId)
    }

    fn len(&self) -> usize {
        self.cache.len()
    }

    fn reverse(&self) -> HashMap<PkgId, String> {
        self.cache.iter().map(|(k, v)| (*v, k.to_string())).collect()
    }
}

fn load() -> io::Result<(PkgIdLookup, HashMap<PkgId, HashSet<PkgId>>)> {
    let input_path = env::args().nth(1).expect("first argument: input file");
    let file = io::BufReader::new(fs::File::open(input_path)?);
    let mut key: PkgId = 0;
    let mut set: HashSet<PkgId> = HashSet::new();
    let mut map: HashMap<PkgId, HashSet<PkgId>> = HashMap::with_capacity(30_000);
    let mut all = PkgIdLookup {
        cache: HashMap::new(),
    };
    for line in file.lines() {
        let line = line?;
        if line.ends_with(":") {
            if !set.is_empty() {
                map.insert(key, set.clone());
                set.clear();
            }
            key = all.id_of(&line[0..line.len()-1]);
            continue;
        }
        assert_eq!(" - ", &line[0..3]);

        // TODO: hack, skipping first entry (_essential)
        if 0 == key {
            continue;
        }

        set.insert(all.id_of(&line[3..]));
    }

    Ok((all, map))
}

fn main() {
    let (all, map) = load().expect("loading file");
    println!("{}", map.len());
    println!("{}", all.len());
    println!("{}", all.reverse()[map.keys().nth(0).unwrap()]);
}
