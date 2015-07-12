#!/bin/sh
lxc-ls | fgrep fbuild | while read x; do ./destroy.sh $x; done
