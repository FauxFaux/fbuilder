#!/usr/bin/env python3
import json
import subprocess
import unittest
from enum import Enum
from typing import List, Tuple

import munch  # python3-munch


class Cond(Enum):
    ALWAYS_YES = 1
    ALWAYS_NO = 2
    DUNNO = 3


def evaluate_cond(cond) -> Tuple[List[str], Cond]:
    assert 'CompoundList' == cond.type
    assert 1 == len(cond.commands)

    # TODO: eliminate stuff
    # print('assuming condition is true:', cond.commands)

    statement = cond.commands[0]
    if 'Command' != statement.type:
        raise Exception("unexpected if statement: " + str(statement))

    cmd = statement.name
    if 'Word' != cmd.type:
        raise Exception("Unexpected command type: " + str(cmd.type))

    if cmd.text in (
        'which'
    ):
        return set(), Cond.DUNNO

    if cmd.text not in (
        '[',
        'test'
    ):
        raise Exception("non-conditional if statement, running: " + cmd.text)

    side_effects = set()
    for part in statement.suffix:
        if 'Word' == part.type:
            continue
        side_effects.update(scan(part))

    return side_effects, Cond.DUNNO


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
                'db_get',  # TODO: debconf?
                'db_purge',  # TODO: debconf?
                'db_version',  # TODO: debconf?
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
        side_effects, res = evaluate_cond(statement.clause)
        if Cond.DUNNO == res:
            side_effects.update(scan(statement.then))
            if 'otherwise' in statement:
                side_effects.update(scan(statement.otherwise))
        elif Cond.ALWAYS_YES == res:
            side_effects.update(scan(statement.then))
        elif Cond.ALWAYS_NO == res:
            side_effects.update(scan(statement.otherwise))
        else:
            raise Exception('Impossible cond: ' + res)

        return side_effects

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

    # don't support this type of conditional expression
    @unittest.expectedFailure
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
