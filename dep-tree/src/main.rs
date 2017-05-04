#![feature(retain_hash_collection)]

extern crate futures;
extern crate futures_cpupool;

use std::env;
use std::fs;
use std::io;

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use futures::Future;
use futures_cpupool::CpuPool;

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
    let mut namer = PkgIdLookup {
        cache: HashMap::new(),
    };
    for line in file.lines() {
        let line = line?;
        if line.ends_with(":") {
            if !set.is_empty() {
                map.insert(key, set.clone());
                set.clear();
            }
            key = namer.id_of(&line[0..line.len()-1]);
            continue;
        }
        assert_eq!(" - ", &line[0..3]);

        // TODO: hack, skipping first entry (_essential)
        if 0 == key {
            continue;
        }

        set.insert(namer.id_of(&line[3..]));
    }

    Ok((namer, map))
}

type DomResult = Option<HashSet<PkgId>>;

fn find_dominators(bin: PkgId, map: Arc<HashMap<PkgId, HashSet<PkgId>>>) -> DomResult {
    let mut found: Option<HashSet<PkgId>> = None;

    for deps in map.values() {
        if !deps.contains(&bin) {
            continue;
        }

        if let Some(ref mut so_far) = found {
            so_far.retain(|x| deps.contains(x));
        } else {
            let mut initial = deps.clone();
            initial.remove(&bin);
            found = Some(initial);
            continue;
        }

    }
    found
}

fn par_map<IS, S, D, F, C>(wut: IS, with: F, captures: C) -> Vec<D>
where IS: Iterator<Item=S>,
      S: Send,
      D: Send,
      F: 'static + Send + Fn(C, S) -> D,
      C: 'static + Clone + Send,
       {

    let pool = CpuPool::new_num_cpus();
    let mut work = Vec::new();

    for item in wut {
        let captures = captures.clone();
        work.push(pool.spawn_fn(move || 
            Ok::<D, ()>(with(captures, item))
        ));
    }

    // TODO: don't really get why this can't be a map:
    //return work.iter().map(|ref future| future.wait().unwrap()).collect();

    let mut ret = Vec::with_capacity(work.len());
    for future in work {
        ret.push(future.wait().unwrap());
    }
    ret
}

fn main() {
    let (namer, map) = load().expect("loading file");
    let mappy = Arc::new(map);

    let names = namer.reverse();

    let mut all_bins = HashSet::with_capacity(names.len());
    for bins in mappy.values() {
        for bin in bins {
            all_bins.insert(*bin);
        }
    }

    let pool = CpuPool::new_num_cpus();
    for found in par_map(all_bins.iter(), |mappy, bin| (bin, find_dominators(*bin, mappy)), mappy) {
        let (bin, deps) = found;
        let deps = deps.unwrap();
        if deps.len() < 10 {
            println!("{}: {:?}", names[&bin], deps.iter().map(|id|
                        names[id].clone()).collect::<Vec<String>>());
        }
    }
}
