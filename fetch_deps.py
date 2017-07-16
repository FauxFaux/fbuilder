#!/usr/bin/env python3
import json
import re
import subprocess
import tarfile
from collections import defaultdict

import sys

import explaino


def main():
    # fetch_deps.py http://urika:3142/debian/pool/ openjdk-8 zzuf | (cd debs && xargs -P4 -n 10 wget -nc)
    mirror = sys.argv[1]
    wanted = set(sys.argv[2:])
    needed = set()
    with open('data/all-build-deps-amd64.txt') as deps:
        lines = iter(deps)
        _ = next(lines)
        for line in lines:
            parts = line.strip().split(' ')
            if parts[0] in wanted:
                wanted.remove(parts[0])
                needed.update(parts[1:])

    assert set() != needed

    # deb2pg/apt% egrep '^Filename: pool' fakedroot/var/lib/apt/lists/deb.debian.org_debian_dists_unstable_main_binary-amd64_Packages | cut -d/ -f 2- > ~/code/fbuilder/unstable.lst
    RE = re.compile('/([^/_]*)_')
    debs = set()
    urls = []
    with open('data/unstable.lst') as listing:
        for line in listing:
            pkg = RE.search(line).group(1)
            if pkg in needed:
                needed.remove(pkg)
                parts = line.split('/')
                debs.add(parts[len(parts) - 1].strip())
                urls.append(mirror + line.strip())

    assert set() == needed

    subprocess.check_call(['wget', '-nc'] + urls, cwd='debs')

    control = defaultdict(dict)
    for deb in debs:
        control_gz = subprocess.Popen(['bsdtar', '-Oxf', 'debs/' + deb, 'control.tar.gz'], stdin=subprocess.DEVNULL,
                                      stdout=subprocess.PIPE)
        tar = tarfile.open(fileobj=control_gz.stdout, mode='r|gz')
        for member in tar:
            contents = tar.extractfile(member)
            if contents:
                control[deb][member.name] = contents.read()

    scripties = []
    for deb, content in control.items():
        for name, data in content.items():
            if name in ('./postinst', './preinst'):
                scripties.append((deb, name, data))

    scripties.sort(key=lambda x: len(x[2]))

    for deb, name, data in scripties:
        try:
            print(deb, name, explaino.explaino(data.decode('utf-8')))
        except json.decoder.JSONDecodeError as e:
            print(deb, name, 'parse failed :((((((((((((((')
        except Exception as e:
            msg = str(e)
            print(deb, name, type(e), msg[0:160])


if '__main__' == __name__:
    main()
