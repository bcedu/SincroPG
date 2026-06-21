APP=clipg
VERSION=2.15.0
BUILD=target/release/$(APP)

.PHONY: build deb rpm clean

# -------------------------
# BUILD RUST
# -------------------------
build:
	cargo build --release
	cp target/release/CliPG target/release/$(APP)

# -------------------------
# DEB
# -------------------------
deb: build
	rm -rf build/deb
	mkdir -p build/deb/DEBIAN
	mkdir -p build/deb/usr/bin
	mkdir -p build/deb/usr/share/applications
	mkdir -p build/deb/usr/share/icons/hicolor/256x256/apps

	cp $(BUILD) build/deb/usr/bin/$(APP)
	cp packaging/deb/control build/deb/DEBIAN/control
	cp packaging/shared/clipg.desktop build/deb/usr/share/applications/
	cp packaging/shared/icons/png/clipg_inv_256.png build/deb/usr/share/icons/hicolor/256x256/apps/clipg.png

	dpkg-deb --build build/deb $(APP)_$(VERSION)_amd64.deb

# -------------------------
# RPM
# -------------------------
rpm: build
	rm -rf build/rpm
	mkdir -p build/rpm/RPMS
	mkdir -p build/rpm/BUILD
	mkdir -p build/rpm/SOURCES
	mkdir -p build/rpm/SPECS
	mkdir -p build/rpm/SRPMS

	rpmbuild \
		--define "_topdir $(PWD)/build/rpm" \
		-ba packaging/rpm/clipg.spec

# -------------------------
# CLEAN
# -------------------------
clean:
	cargo clean
	rm -rf build *.deb
