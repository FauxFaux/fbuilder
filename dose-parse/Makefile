all: deps-versions.yaml ordered-deps

deps-versions.yaml: ../docker/dose-amd64.yaml src/main.rs
	cargo run --release -- ../docker/dose-amd64.yaml --include-versions > deps-versions.yaml

ordered-deps: ../docker/dose-amd64.yaml ../docker/essentials.list src/main.rs
	cargo run --release -- ../docker/dose-amd64.yaml --excluded=../docker/essentials.list > ordered-deps


