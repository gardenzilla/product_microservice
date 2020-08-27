 
.PHONY: release, test, dev

release:
	cargo build --release
	strip target/release/product_microservice

build:
	cargo build

dev:
	# . ./ENV.sh; backper
	cargo run;

test:
	cargo test