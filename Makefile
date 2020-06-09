debug:
	cargo build
	cp -f assets/* target/debug/

release:
	cargo build --release
	cp -f assets/* target/release/

check:
	cargo check

all: release

clean:
	rm -r target