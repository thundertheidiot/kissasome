# kind of stupid, make is used as a command runner here
.PHONY: all lint test

all: lint test

lint:
	npm run lint
	cargo clippy

test:
	cargo test
