#!/usr/bin/env python3

import collections
import json

import apt


def deterator(o):
    while o.step():
        yield o


def deepdict():
    return collections.defaultdict(deepdict)


def main():
    sources = apt.apt_pkg.SourceRecords()

    sout = deepdict()
    for source in deterator(sources):
        sout[source.package][source.version] = {
            'deps2': source.build_depends.get('Build-Depends', None),
            'deps': [(x[0][0], x[0][2] or None, x[0][1] or None) for x in source.build_depends.get('Build-Depends', {})]
        }

    print(json.dumps(sout))

if __name__ == '__main__':
    main()
