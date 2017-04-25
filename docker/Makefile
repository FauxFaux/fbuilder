ARCH ?= amd64

all: dose-$(ARCH).yaml

%.latest: $(wildcard $*/*)
	docker build --tag=$*:latest $*

dose.latest: sid-be.latest


# can't use --uild-arg or --env to affect CMD, so do it in make :(
dose-%.yaml: dose.latest dose-yaml/%/packages dose-yaml/sources
	docker run \
		$(shell docker build --quiet --build-arg arch=$(ARCH) dose-yaml) \
		deb-buildcheck --explain --successes --deb-native-arch=$(ARCH) /tmp/packages /tmp/sources \
		| sponge $@

# directory layout here to work around wget -O -N being a bit dumb
dose-yaml/%/Packages.xz:
	mkdir -p dose-yaml/$*
	cd dose-yaml/$* && wget -N "http://deb.debian.org/debian/dists/sid/main/binary-$*/Packages.xz"

dose-yaml/Sources.xz:
	cd dose-yaml && wget -N "http://deb.debian.org/debian/dists/sid/main/source/Sources.xz"

dose-yaml/%/packages: dose-yaml/%/Packages.xz
	xz -d < $^ > $@

dose-yaml/sources: dose-yaml/Sources.xz
	xz -d < $^ > $@


clean-cache:
	docker pull debian:sid
	$(RM) -f dose-yaml/*.xz

clean: clean-cache
	docker rmi dose:latest sid-be:latest


deps:
	apt-get install wget moreutils

.PRECIOUS: dose-yaml/%/Packages.xz dose-yaml/Sources.xz
