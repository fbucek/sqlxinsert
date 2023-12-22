.PHONY: testall doc initdocker rmdocker test

all: check rmdocker initdocker test rmdocker doc
testall: rmdocker initdocker test rmdocker

# Docker used only for testing postgres database
initdocker:
	docker-compose up -d
	sleep 1
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
