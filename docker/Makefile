all: sid-be.latest

%.latest: $(wildcard $*/*)
	docker build --tag=$*:latest $*
