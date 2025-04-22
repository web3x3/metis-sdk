build:
	cargo build -r

check:
	cargo check -r --all

test:
	cargo test -r --workspace

test-all:
	cargo test -r --workspace --all-features

accept:
	cargo insta accept --all

fmt:
	cargo fmt --all

clippy-all:
	cargo clippy --workspace --all-features --benches --examples --tests -- -D warnings

clippy:
	cargo clippy --workspace --benches --tests --examples -- -D warnings

fix:
	cargo clippy --workspace --all-features --benches --examples --tests --fix --allow-dirty

bench:
	JEMALLOC_SYS_WITH_MALLOC_CONF="thp:always,metadata_thp:always" cargo bench -p metis-pe --features jemalloc
