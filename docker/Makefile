ARCH ?= amd64

all: dose.latest

%.latest: $(wildcard $*/*)
	docker build --no-cache --network=mope --tag=$*:latest $*
	touch $@

dose.latest: sid-be.latest

clean-cache:
	docker pull debian:sid
	$(RM) -f dose-yaml/*.xz dose-yaml/**/*.xz dose-yaml/sources dose-yaml/**/packages

clean: clean-cache
	-docker rmi dose:latest sid-be:latest
	$(RM) -f dose-*.yaml sid-be.latest dose.latest
