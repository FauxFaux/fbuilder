all: deps-versions.yaml ordered-deps

deps-versions.yaml: ../docker/dose-amd64.yaml ../docker/essentials.list src/main.rs
	cargo run --release ../docker/dose-amd64.yaml ../docker/essentials.list --include-versions > deps-versions.yaml

ordered-deps: ../docker/dose-amd64.yaml ../docker/essentials.list src/main.rs
	cargo run --release ../docker/dose-amd64.yaml ../docker/essentials.list > ordered-deps


