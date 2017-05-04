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

type OutstandingDeps = HashMap<PkgId, HashSet<PkgId>>;

impl PkgIdLookup {
    fn id_of(&mut self, pkg: &str) -> PkgId {
        let len = self.cache.len();

        // TODO: how to reference PkgId here?
        if len >= std::u16::MAX as usize {
            panic!("too many packages!");
        }

        *self.cache.entry(pkg.to_string()).or_insert(len as PkgId)
    }

    fn get(&self, pkg: &str) -> PkgId {
        self.cache[pkg]
    }

    fn len(&self) -> usize {
        self.cache.len()
    }

    fn reverse(&self) -> HashMap<PkgId, String> {
        self.cache.iter().map(|(k, v)| (*v, k.to_string())).collect()
    }
}

fn load() -> io::Result<(PkgIdLookup, OutstandingDeps)> {
    let input_path = env::args().nth(1).expect("first argument: input file");
    let file = io::BufReader::new(fs::File::open(input_path)?);
    let mut key: PkgId = 0;
    let mut set: HashSet<PkgId> = HashSet::new();
    let mut map: OutstandingDeps = HashMap::with_capacity(30_000);
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

fn find_dominators(bin: PkgId, map: Arc<OutstandingDeps>) -> DomResult {
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

#[derive(Debug, Clone)]
struct Instruction {
    install: HashSet<PkgId>,
    satisfies: HashSet<PkgId>,
}

#[derive(Debug)]
struct State {
    instructions: Vec<Instruction>,
    outstanding: OutstandingDeps,
}

impl State {
    fn new(outstanding: OutstandingDeps) -> State {
        State {
            instructions: Vec::new(),
            outstanding
        }
    }

    fn install(&self, dep: PkgId) -> (State, State) {
        let mut ours: OutstandingDeps = HashMap::new();
        let mut others: OutstandingDeps = HashMap::new();

        for (pkg, ref mut deps) in &self.outstanding {
            if deps.contains(&dep) {
                ours.insert(*pkg, deps.clone());
            } else {
                others.insert(*pkg, deps.clone());
            }
        }

        assert_ne!(0, ours.len());

        let mut common_deps = ours.values().next().unwrap().clone();
        for (_, ref mut deps) in &mut ours {
            deps.remove(&dep);
            common_deps.retain(|x| deps.contains(x));
        }

        let mut satisfied = HashSet::new();

        for (pkg, ref mut deps) in &mut ours {
            for dep in &common_deps {
                deps.remove(&dep);
            }
            if 0 == deps.len() {
                satisfied.insert(*pkg);
            }
        }

        for pkg in &satisfied {
            ours.remove(&pkg);
        }

        common_deps.insert(dep);

        let mut new_instructions = self.instructions.clone();
        new_instructions.push(Instruction {
               satisfies: satisfied,
               install: common_deps,
        });

        (State {
            outstanding: ours,
            instructions: new_instructions,
        }, State {
            outstanding: others,
            instructions: self.instructions.clone(),
        })
    }

    fn outstanding_bins(&self) -> HashSet<PkgId> {
        let mut ret = HashSet::with_capacity(self.outstanding.len());
        for deps in self.outstanding.values() {
            for dep in deps {
                ret.insert(*dep);
            }
        }
        ret
    }

    fn print(&self, names: &HashMap<PkgId, String>)
        //TODO: where N: std::ops::Index<PkgId, Output=String>
    {
        println!("State [ remaining packages: {}, install steps: ", self.outstanding.len());
        for (idx, ins) in self.instructions.iter().enumerate() {
            print!(" - step {}: apt install ", idx);
            for pkg in &ins.install {
                print!(" {}", names[&pkg]);
            }
            println!("   happy: ");
            for pkg in &ins.satisfies {
                print!(" {}", names[&pkg]);
            }
            println!();
        }
        println!("]");
    }
}

fn main() {
    let (namer, map) = load().expect("loading file");
    let names = namer.reverse();

    let mut ours = State::new(map);
    loop {
        let (or, _) = ours.install(*ours.outstanding_bins().iter().next().unwrap());
        ours = or;

        ours.print(&names);
    }
}

fn dominators(map: OutstandingDeps, namer: PkgIdLookup) {
    let mappy = Arc::new(map);
    let names = namer.reverse();

    let mut all_bins = HashSet::with_capacity(names.len());
    for bins in mappy.values() {
        for bin in bins {
            all_bins.insert(*bin);
        }
    }

    let pool = CpuPool::new_num_cpus();
    let mut work = Vec::with_capacity(all_bins.len());

    for bin in all_bins {
        let mappy = mappy.clone();
        work.push(pool.spawn_fn(move || {
            Ok::<(PkgId, DomResult), ()>((bin, find_dominators(bin, mappy)))
        }));
    }

    for future in work {
        let found = future.wait().unwrap();
        let (bin, deps) = found;
        let deps = deps.unwrap();
        if deps.len() < 10 {
            println!("{}: {:?}", names[&bin], deps.iter().map(|id|
                        names[id].clone()).collect::<Vec<String>>());
        }
    }
}

#[cfg(test)]
mod tests {
    use PkgId;
    use State;

    use HashSet;
    use HashMap;

    fn set_of(args: &[PkgId]) -> HashSet<PkgId> {
        let mut ret = HashSet::with_capacity(args.len());
        for arg in args {
            ret.insert(*arg);
        }
        ret
    }

    #[test]
    fn install() {
        let mut out = HashMap::new();
        out.insert(1, set_of(&[12, 13]));
        out.insert(2, set_of(&[12, 13, 14]));
        out.insert(3, set_of(&[12, 13, 14, 16]));
        out.insert(4, set_of(&[11]));
        out.insert(5, set_of(&[13]));
        let initial = State::new(out);
        let (ours, theirs) = initial.install(12);
        assert_eq!(1, ours.instructions.len());
        assert_eq!(set_of(&[12, 13]), ours.instructions[0].install);
        assert_eq!(set_of(&[1]), ours.instructions[0].satisfies);
        assert_eq!(set_of(&[2, 3]), ours.outstanding.keys().map(|x| *x).collect());
        // TODO: vals of outstanding

        let (ours, theirs) = ours.install(14);
        assert_eq!(2, ours.instructions.len());
        assert_eq!(set_of(&[14]), ours.instructions[1].install);
        assert_eq!(set_of(&[2]), ours.instructions[1].satisfies);
        assert_eq!(set_of(&[3]), ours.outstanding.keys().map(|x| *x).collect());

        assert_eq!(set_of(&[16]), ours.outstanding_bins());
    }
}

