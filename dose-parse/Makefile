all: ordered-deps

ordered-deps: ../docker/dose-amd64.yaml ../docker/essentials.list src/main.rs
	cargo run --release ../docker/dose-amd64.yaml ../docker/essentials.list > ordered-deps


