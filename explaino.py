#!/usr/bin/env python3
import json
import subprocess
import unittest
from typing import List

import munch  # python3-munch


def evaluate_cond(cond) -> bool:
    assert 'CompoundList' == cond.type
    assert 1 == len(cond.commands)

    # TODO: eliminate stuff
    # print('assuming condition is true:', cond.commands)

    return True


def scan(statement) -> List:
    if 'Command' == statement.type:
        if 'name' not in statement:
            # variable assignment
            assert 'prefix' in statement
            return set()
        cmd = statement.name
        assert 'Word' == cmd.type

        # these commands need *some* action, but maybe they're triggerable?
        if cmd.text in (
                'add-shell',
                'dpkg-divert',
                'dpkg-trigger',
                'dpkg-maintscript-helper',
                'ucf',  # update-configuration-file
                'update-alternatives',
                'update-icon-caches',
                'update-perl-sax-parsers',
                'update-menus',
                'update-mime',
                'update-mime-database.real',
                'update-xmlcatalog',
                'cp',  # TODO: obviously not okay
                'ln',  # TODO: obviously not okay
                'rm',  # TODO: obviously not okay
                'mv',  # TODO: obviously not okay
                'mkdir',  # TODO: obviously not okay
                'rmdir',  # TODO: obviously not okay
                'db_get',  # TODO: debconf? What year is it??
                'db_purge',  # TODO: debconf? What year is it??
                'db_version',  # TODO: debconf? What year is it??
                'py3compile',
                'libgvc6-config-update',
                'update-rc.d'
        ):
            return {cmd.text}

        if '.' == cmd.text:
            assert 1 == len(statement.suffix)
            what = statement.suffix[0]
            assert 'Word' == what.type
            if what.text in (
                    '/usr/share/debconf/confmodule'
            ):
                return set()

        if cmd.text in (
                'ldconfig'
        ) and 'suffix' not in statement:
            return {'ldconfig'}

        # lol no
        if cmd.text in (
                'set',
                'echo',
                'print',  # TODO: is this even valid?
                'exit',
                'cd',
                'umask',
        ):
            return set()

        raise Exception('unsupported command: ' + cmd.text)
    elif 'If' == statement.type:
        if evaluate_cond(statement.clause):
            return scan(statement.then)
    elif 'CompoundList' == statement.type:
        actions = set()
        for sub in statement.commands:
            actions.update(scan(sub))
        return actions

    raise Exception("unsupported statement: " + statement.type + " // " + str(statement))


def explaino(body: str):
    line_end = body.index('\n')
    shebang = body[:line_end]
    remain = body[line_end + 1:]
    if shebang not in (
            '#!/bin/sh',
            '#!/bin/sh -e',
            '#! /bin/sh -e',
            '#!/bin/bash',
            '#!/usr/bin/env bash',
            '#! /bin/sh',
    ):
        raise Exception("unsupported shebang: " + shebang)

    parse = subprocess.Popen(['parse-shell/parse-shell'],
                             stdin=subprocess.PIPE,
                             stderr=subprocess.DEVNULL,  # TODO
                             stdout=subprocess.PIPE,
                             )
    out, _ = parse.communicate(remain.encode('utf-8'))
    raw_dict = json.loads(out.decode('utf-8'))
    doc = munch.munchify(raw_dict)
    actions = set()
    try:
        for statement in doc.commands:
            actions.update(scan(statement))
    except:
        # print(body)
        # print(json.dumps(raw_dict, indent=' '))
        # print(doc)
        raise

    return actions


if '__main__' == __name__:
    import sys

    explaino(sys.stdin.read())


class SmokeTests(unittest.TestCase):
    def test_gettext(self):
        self.assertEqual({'ldconfig'}, explaino("""#!/bin/sh
set -e
if [ "$1" = "configure" ]; then
  ldconfig
fi
"""))

    def test_libxml2(self):
        self.assertEqual(set(), explaino("""#!/bin/sh

set -e

[ "$1" = "upgrade" ] &&
[ -L /usr/share/doc/libxml2-utils ] &&
rm -f /usr/share/doc/libxml2-utils



exit 0"""))

    def test_assignment(self):
        self.assertEqual(set(), explaino("""#!/bin/sh
A=5
"""))

    def test_fake_env(self):
        self.assertEqual({'ldconfig'}, explaino("""#!/bin/sh
A=5 ldconfig
"""))
