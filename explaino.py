#!/usr/bin/env python3
import unittest
from typing import List

import bashlex
from bashlex import ast


def evaluate_cond(cond: ast.node) -> bool:
    assert 'list' == cond.kind
    assert 2 == len(cond.parts)

    # trailing semicolon, pointless
    assert 'operator' == cond.parts[1].kind
    assert ';' == cond.parts[1].op

    # TODO: eliminate stuff

    return True


def scan(statement: ast.node) -> List:
    if 'command' == statement.kind:
        cmd = statement.parts
        assert 'word' == cmd[0].kind
        if cmd[0].word in (
                'set',
        ):
            return set()

        if 1 == len(cmd) and cmd[0].word in (
                'ldconfig'
        ):
            return {'ldconfig'}

    elif 'compound' == statement.kind:
        assert 1 == len(statement.list)
        un_compound = statement.list.pop(0)
        if 'if' == un_compound.kind:
            assert 5 == len(un_compound.parts)
            # if
            cond = un_compound.parts[1]
            # then
            body = un_compound.parts[3]
            # fi

            if evaluate_cond(cond):
                return scan(body)

    raise Exception("unsupported statement: " + str(statement))


def explaino(body: str):
    line_end = body.index('\n')
    shebang = body[:line_end]
    remain = body[line_end + 1:]
    if shebang not in (
            '#!/bin/sh',
            '#!/bin/bash',
            '#!/usr/bin/env bash',
            '#! /bin/sh',
    ):
        raise Exception("unsupported shebang: " + shebang)

    print (remain)

    actions = set()
    for statement in bashlex.parse(remain):
        actions.update(scan(statement))

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
