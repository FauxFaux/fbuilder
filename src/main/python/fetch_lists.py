#!/usr/bin/python3

import apt

c = apt.cache.Cache(rootdir='.')
c.open() # or something, makes it wake up

> etc/apt/sources.list
# deb     http://urika:3142/ftp.debian.org/debian sid main
# deb-src http://urika:3142/ftp.debian.org/debian sid main

c.open() # wake up again
c.update()

#  ~/code/dose/deb-buildcheck.native --deb-native-arch=amd64 var/lib/apt/lists/*Packages var/lib/apt/lists/*Sources -s -e > a
