#!/bin/sh
L=$(lxc-config lxc.lxcpath) || exit 2
if [ -z "$L" ]; then
    exit 3
fi
M=$(basename $1)
lxc-stop -k -n "$M"
lxc-destroy -n "$M"
sudo btrfs subvolume del $L/$M/rootfs/var/lib/machines
sudo btrfs subvolume del $L/$M/rootfs
sudo rm -rf $L/$M

