ARCH ?= amd64

all: dose-$(ARCH).yaml

%.latest: $(wildcard $*/*)
	docker build --tag=$*:latest $*

dose.latest: sid-be.latest


# can't use --uild-arg or --env to affect CMD, so do it in make :(
dose-%.yaml: dose.latest dose-yaml/%/packages dose-yaml/sources
	$(eval D:=$(shell mktemp -d))
	-docker run \
		-v $(D):/tmp/out \
		$(shell docker build --quiet --build-arg arch=$(ARCH) dose-yaml) \
		deb-buildcheck --deb-ignore-essential --explain --successes --deb-native-arch=$(ARCH) \
			/tmp/packages /tmp/sources \
			--outfile=/tmp/out/dose
	mv $(D)/dose $@
	$(RM) -rf $(D)

# directory layout here to work around wget -O -N being a bit dumb
dose-yaml/%/Packages.xz:
	mkdir -p dose-yaml/$*
	cd dose-yaml/$* && wget -N "http://deb.debian.org/debian/dists/sid/main/binary-$*/Packages.xz"

dose-yaml/Sources.xz:
	cd dose-yaml && wget -N "http://deb.debian.org/debian/dists/sid/main/source/Sources.xz"

dose-yaml/%/packages: dose-yaml/%/Packages.xz
	xz -d < $^ > $@
	touch -r $^ $@

dose-yaml/sources: dose-yaml/Sources.xz
	xz -d < $^ > $@
	touch -r $^ $@


clean-cache:
	docker pull debian:sid
	$(RM) -f dose-yaml/*.xz dose-yaml/**/*.xz dose-yaml/sources dose-yaml/**/packages

clean: clean-cache
	docker rmi dose:latest sid-be:latest
	$(RM) dose-*.yaml


deps:
	apt-get install wget xz-utils

# fixing cacheing
.PRECIOUS: dose-yaml/%/Packages.xz dose-yaml/Sources.xz dose-yaml/%/packages dose-yaml/sources
