include ../ENV.list
export $(shell sed 's/=.*//' ../ENV.list)

.PHONY: release, test, run

release:
	cargo update
	cargo build --release
	strip target/release/product_microservice

build:
	cargo update
	cargo build
	cargo test

run:
	cargo run

test:
	cargo test