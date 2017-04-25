#!/bin/sh
lxc-ls | fgrep fbuild | while read x; do ./destroy.sh $x; done
find $(lxc-config lxc.lxcpath) -maxdepth 1 -name fbuild-\* -exec ./destroy.sh {} \;
