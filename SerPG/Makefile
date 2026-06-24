APP=serpg
VERSION=2.15.0
BUILD=target/x86_64-unknown-linux-musl/release/SerPG

.PHONY: build deb clean

# -------------------------
# BUILD RUST
# -------------------------
build:
	cargo build --release --target x86_64-unknown-linux-musl

# -------------------------
# DEB
# -------------------------
deb: build
	rm -rf build/deb
	mkdir -p build/deb/DEBIAN
	mkdir -p build/deb/usr/bin

	cp $(BUILD) build/deb/usr/bin/$(APP)
	cp packaging/deb/control build/deb/DEBIAN/control

	dpkg-deb -Zxz --build build/deb $(APP)_$(VERSION)_amd64.deb

# -------------------------
# CLEAN
# -------------------------
clean:
	cargo clean
	rm -rf build *.deb
