# Compiles and packages deb with release profile
build-deb:
    command -v cargo-deb || cargo install cargo-deb
    cargo deb

# Compiles and packages rpm with release profile
build-rpm: build-release
    command -v cargo-generate-rpm || cargo install cargo-generate-rpm
    strip -s {{bin-src}}
    cargo generate-rpm

# Compiles and packages PKGBUILD with release profile
build-aur:
    command -v cargo-aur || cargo install cargo-aur
    cargo aur

install-deb:
    apt install --reinstall ./target/debian/*.deb

install-rpm:
    dnf localinstall ./target/generate-rpm/*.rpm

install-aur:
    makepkg -siD ./target/cargo-aur