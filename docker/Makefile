ARCH ?= amd64

all: dose-$(ARCH).yaml

%.latest: $(wildcard $*/*)
	docker build --tag=$*:latest $*

dose.latest: sid-be.latest


# build is run twice just so there's some progress reporting
dose-%.yaml: dose.latest dose-yaml/%/packages dose-yaml/sources
	docker build --build-arg arch=$(ARCH) dose-yaml
	docker cp $(shell docker run -d \
		$(shell docker build --quiet --build-arg arch=$(ARCH) dose-yaml) \
		true):/tmp/buildcheck $@

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
