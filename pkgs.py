#!/usr/bin/python3
import collections
from typing import Iterator, List, Tuple, Dict

import sys
from collections import Counter


def main():
    instructions = collections.defaultdict(list)
    all = dict(ordered_deps())  # type: Dict[str, List[str]]

    temperature = 0
    while all:

        prefix = find_a_prefix(all.values(), temperature)
        if 1 == len(prefix):
            temperature += 0.001
            sys.stdout.write("\n🔥 heatin' up! 🔥  -  {:.2f}, {} remaining".format(100 * temperature, len(all)))
        else:
            sys.stdout.write('.')
            sys.stdout.flush()

            temperature *= 0.99

        for source, deps in all.items():
            if tuple(deps[0:len(prefix)]) == prefix:
                del deps[0:len(prefix)]
                instructions[source].append(prefix)

        done = [k for k,v in all.items() if not v]

        for k in done:
            del all[k]

    for source, items in instructions.items():
        print('{}: {}'.format(source, items))


def find_a_prefix(values: Iterator[List[str]], threshold: float) -> List[str]:
    last = 0
    i = 1
    prev = None
    while True:
        pkgs, length = Counter(tuple(x[0:i]) for x in values).most_common(1)[0]
        if len(pkgs) < i:
            break
        if last and (last - length) / float(last) > threshold:
            break

        prev = pkgs
        last = length
        i += 1
    return prev


def ordered_deps() -> Iterator[Tuple[str, List[str]]]:
    with open('dose-parse/ordered-deps') as f:
        for line in f:
            src, deps = line.strip().split('\t')
            yield src, deps.split(' ')


if '__main__' == __name__:
    main()
