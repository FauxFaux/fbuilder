#!/bin/sh
M=$(basename $1)
lxc-stop -k -n "$M"
lxc-destroy -n "$M"
sudo btrfs subvolume del ~/.local/share/lxc/$M/rootfs/var/lib/machines
sudo btrfs subvolume del ~/.local/share/lxc/$M/rootfs
sudo rm -rf ~/.local/share/lxc/$M

