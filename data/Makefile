all: sources amd64/packages

# directory layout here to work around wget -O -N being a bit dumb
%/Packages.xz:
	mkdir -p $*
	cd $* && wget -N "http://deb.debian.org/debian/dists/sid/main/binary-$*/Packages.xz"

Sources.xz:
	cd data && wget -N "http://deb.debian.org/debian/dists/sid/main/source/Sources.xz"

%/packages: %/Packages.xz
	xz -d < $^ > $@
	touch -r $^ $@

sources: Sources.xz
	xz -d < $^ > $@
	touch -r $^ $@
