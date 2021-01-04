 
.PHONY: release, test, dev

release:
	cargo update
	cargo build --release
	strip target/release/product_microservice

build:
	cargo update
	cargo build

dev:
	# . ./ENV.sh; backper
	cargo run;

test:
	cargo test