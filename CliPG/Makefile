APP=clipg
VERSION=2.16.3
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

	dpkg-deb -Zxz --build build/deb $(APP)_$(VERSION)_amd64.deb

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
	cp target/release/clipg build/rpm/SOURCES/clipg
	cp packaging/shared/clipg.desktop build/rpm/SOURCES/clipg.desktop
	cp packaging/shared/icons/png/clipg_inv_256.png build/rpm/SOURCES/clipg.png

	rpmbuild \
		--define "_topdir $(PWD)/build/rpm" \
		-ba packaging/rpm/clipg.spec

# -------------------------
# PKGBUILD
# -------------------------
pkgbuild: build
	rm -rf build/arch
	mkdir -p build/arch
	cp target/release/clipg build/arch/clipg
	cp packaging/arch/PKGBUILD build/arch/PKGBUILD
	cp packaging/shared/clipg.desktop build/arch/clipg.desktop
	cp packaging/shared/icons/png/clipg_inv_256.png build/arch/clipg.png

	cd build/arch && makepkg -f --noconfirm
	mkdir -p dist/arch
	cp build/arch/*.pkg.tar.* dist/arch/
	rm dist/arch/clipg-debug-*

# -------------------------
# CLEAN
# -------------------------
clean:
	cargo clean
	rm -rf build *.deb
