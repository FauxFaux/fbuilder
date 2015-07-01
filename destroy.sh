#!/bin/sh
lxc-stop -k -n "$1"
lxc-destroy -n "$1"
sudo btrfs subvolume del ~/.local/share/lxc/$1/rootfs/var/lib/machines
sudo btrfs subvolume del ~/.local/share/lxc/$1/rootfs
sudo rm -rf ~/.local/share/lxc/$1

