#![feature(retain_hash_collection)]

extern crate rand;
extern crate futures;
extern crate futures_cpupool;
extern crate num_cpus;

use std::env;
use std::fs;
use std::io;

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;

use futures::Future;
use futures_cpupool::CpuPool;

// magic:
use std::io::BufRead;
use std::io::Write;
use rand::distributions::IndependentSample;

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

#[derive(Debug, Clone)]
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
               satisfies: satisfied.clone(),
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

    fn next_guess(&self) -> PkgId {

        let mut counter = HashMap::with_capacity(self.outstanding.len());
        let mut count = 0;
        for deps in self.outstanding.values() {
            for dep in deps {
                *counter.entry(*dep).or_insert(0) += 1u32;
                count += 1u32;
            }
        }

        let mut rng = rand::thread_rng();
        let mut pos = rand::distributions::Range::new(0, count).ind_sample(&mut rng);

        let mut it = counter.iter();
        while let Some((pkg, count)) = it.next() {
            if pos < *count {
                return *pkg;
            }
            pos -= *count;
        }

        panic!("ran out of iterator before we ran out of points");
    }

    fn print(&self, names: &HashMap<PkgId, String>)
        //TODO: where N: std::ops::Index<PkgId, Output=String>
    {
        println!("State [ remaining packages: {}, install steps: ", self.outstanding.len());
        for (idx, ins) in self.instructions.iter().enumerate() {
            print!(" - step {}: apt install", idx);
            for pkg in &ins.install {
                print!(" {}", names[&pkg]);
            }
            println!();
            print!("   happy:");
            for pkg in &ins.satisfies {
                print!(" {}", names[&pkg]);
            }
            println!();
        }
        println!("]");
    }
}

fn sorted<I, T>(it: I) -> Vec<T>
where I: Iterator<Item=T>,
      T: Ord {
    let mut ret: Vec<T> = it.collect();
    ret.sort();
    ret
}

fn render(pkg: PkgId, instructions: &Vec<Instruction>, names: &HashMap<PkgId, String>) {
    let mut out = fs::File::create(
            format!("target/{}.Dockerfile", names[&pkg])
        ).expect("create output");

    writeln!(out, "FROM sid-be:latest");
    writeln!(out, "WORKDIR /build");

    for instruction in instructions {
        write!(out, "RUN apt-get install -y --no-install-recommends");
        for dep in sorted(instruction.install.iter().map(|ref dep| names[dep].to_string())) {
            write!(out, " {}", dep);
        }
        writeln!(out);
    }
    writeln!(out, "RUN apt-get source {}", names[&pkg]);
}

fn divide_up(state: State) -> (usize, Vec<(PkgId, Vec<Instruction>)>) {
    let mut to_do = vec![state];

    let mut bid = 0usize;
    let mut result = Vec::with_capacity(to_do[0].outstanding.len());

    while let Some(state) = to_do.pop() {
        let pick = state.next_guess();
        let (left, right) = state.install(pick);
        if !right.outstanding.is_empty() {
            to_do.push(right);
        }

        {
            let satisfied = &left.instructions[left.instructions.len() - 1].satisfies;
            for pkg in satisfied {
                result.push((*pkg, left.instructions.clone()));
                bid += left.instructions.len();
            }
        }

        if !left.outstanding.is_empty() {
            to_do.push(left);
        }
    }

    (bid, result)
}

fn main() {
    let (namer, map) = load().expect("loading file");
    let names = namer.reverse();

    let init = State::new(map);
    let best_bid = Arc::new(Mutex::new(std::usize::MAX));

    for i in 0..num_cpus::get() {
        let init = init.clone();
        let best_bid = best_bid.clone();
        let names = names.clone();
        std::thread::spawn(move || {
            loop {
                let (bid, solution) = divide_up(init.clone());
                let mut best_bid = best_bid.lock().expect("no poison");
                if bid < *best_bid {
                    *best_bid = bid;
                    for (pkg, instructions) in solution {
                        render(pkg, &instructions, &names);
                    }
                    println!("new winner! {}", bid);
                } else {
                    println!("loser: {}", bid);
                }
            }
        });
    }
    println!("press return to exit:");
    let mut ignored = String::new();
    io::stdin().read_line(&mut ignored).unwrap();
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

