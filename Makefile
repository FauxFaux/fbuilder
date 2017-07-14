ARCH ?= amd64

all: data data/essentials.list data/all-build-deps-$(ARCH).txt

docker/%.latest:
	$(MAKE) -C docker $(@F)

data:
	$(MAKE) -C data

# build is run twice just so there's some progress reporting
data/dose-%.yaml: data/%/packages data/sources docker/dose.latest
	mkdir -p vol
	cp -ar --reflink=auto $< data/sources vol
	$(RM) vol/buildcheck
	-docker run -v $(shell pwd)/vol:/vol -i dose:latest \
		deb-buildcheck --deb-ignore-essential --explain --successes --deb-native-arch=$(ARCH) \
        	/vol/packages /vol/sources \
        	--outfile=/vol/buildcheck
	cp -ar vol/buildcheck $@

data/essentials.list: docker/sid-be.latest
	docker run sid-be:latest dpkg --get-selections | awk '{print $$1}' | cut -d: -f 1 > $@

clean-cache:
	docker pull debian:sid
	-docker rmi dose:latest sid-be:latest
	$(RM) data/*.xz data/**/*.xz data/sources data/**/packages

clean:
	$(RM) data/all-build-deps-*.txt data/essential.list data/dose-*.yaml


deps:
	apt-get install wget xz-utils

data/all-build-deps-%.txt: data/dose-%.yaml data/essentials.list dose-parse/src/main.rs
	cd dose-parse && cargo run --release -- ../$< ../data/essentials.list > ../$@

# fixing cacheing
.PRECIOUS: data/%/Packages.xz data/Sources.xz data/%/packages data/sources data/dose-%.yaml
