#!/bin/sh
lxc-stop -k -n "$1"
lxc-destroy -n "$1"
btrfs subvolume del ~/.local/share/lxc/$1/rootfs/var/lib/machines
btrfs subvolume del ~/.local/share/lxc/$1/rootfs
rm -rf ~/.local/share/lxc/$1

