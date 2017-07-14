all: all-build-deps-amd64.txt

all-build-deps-%.txt: ../docker/dose-%.yaml ../docker/essentials.list src/main.rs
	cargo run --release -- ../docker/dose-amd64.yaml ../docker/essentials.list > $@
