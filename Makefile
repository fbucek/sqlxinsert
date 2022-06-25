.PHONY: testall doc initdocker rmdocker test

all: check initdocker test rmdocker doc
testall: initdocker test rmdocker

# Docker used only for testing postgres database
initdocker:
	docker-compose up -d
rmdocker:
	docker stop sqlxinsert-db-test
	docker rm sqlxinsert-db-test

# Cargo commands
build:
	cargo build --all-targets
test:
	cargo test
check:
	cargo check
	cargo clippy
	cargo fmt
	cargo audit
# Create documentaion
doc:
	cargo doc --no-deps --open
clean:
	cargo clean --doc
