#!/bin/sh
lxc-ls | fgrep qbuild | while read x; do ./destroy.sh $x; done
