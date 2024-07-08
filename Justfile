default:
    @just --list

test:
    @cargo nextest run

lint:
    @cargo clippy --workspace --all-features -- -D warnings
    @cargo fmt --all -- --check

build:
    @cargo build

build-release:
    @cargo build --release

update:
    @cargo upgrade

build-release-linux:
    @cargo build --release --target=x86_64-unknown-linux-musl
    @strip target/x86_64-unknown-linux-musl/release/infini

build-release-macos-x86:
    @cargo build --release --target=x86_64-apple-darwin
build-release-macos-aarch:
    @cargo build --release --target=aarch64-apple-darwin

deploy:
    @cargo build --release
    @cp ./target/release/infini ~/.cargo/bin
