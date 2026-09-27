APPID       := 'dev.heppen.webapps'
PREFIX      := if "${HOME}" == "" { "/usr/local" } else { "${'HOME'}" / ".local"}

BASE_DIR := PREFIX

TARGET_DIR := 'target' / 'release'
BIN_SRC := TARGET_DIR / "webapps"

BIN_DST := BASE_DIR / 'bin' / APPID
DESKTOP_SRC := 'resources' / (APPID + '.desktop')
DESKTOP_DST := BASE_DIR / 'share/applications' / (APPID + '.desktop')

METAINFO_SRC := 'resources' / (APPID + '.metainfo.xml')
METAINFO_DST := BASE_DIR / 'share/metainfo' / (APPID + '.metainfo.xml')

ICON_SRC := 'resources/icons/hicolor'
ICON_DST := BASE_DIR / 'share/icons/hicolor'

# Default task
default: build

# Builds the project
build: format check test
    cargo build --release

# Checks the project
check:
    cargo check

# Checks the project
format:
    cargo fmt --all

# Runs tests
test:
    cargo test

# Runs the application
run: build
    {{BIN_SRC}}

# Build + debug
build-dev: format check test
    cargo build

# Run + debug
run-dev: build-dev
    cargo run

# Installs files
install:
    install -Dm0755 {{BIN_SRC}} {{BIN_DST}}
    install -Dm0644 {{DESKTOP_SRC}} {{DESKTOP_DST}}
    install -Dm0644 {{METAINFO_SRC}} {{METAINFO_DST}}

    for size in `ls {{ICON_SRC}}`; do \
        install -Dm0644 "{{ICON_SRC}}/$size/apps/{{APPID}}.png" "{{ICON_DST}}/$size/apps/{{APPID}}.png"; \
    done

# Uninstalls files
uninstall:
    rm -v {{BIN_DST}}
    rm -v {{DESKTOP_DST}}
    rm -v {{METAINFO_DST}}
    rm -v {{ICON_DST}}/*/apps/{{APPID}}.png

# Vendor dependencies locally
vendor:
    #!/usr/bin/env bash
    mkdir -p .cargo
    cargo vendor --sync Cargo.toml | head -n -1 > .cargo/config.toml
    echo 'directory = "vendor"' >> .cargo/config.toml
    echo >> .cargo/config.toml
    echo '[env]' >> .cargo/config.toml
    if [ -n "${SOURCE_DATE_EPOCH}" ]
    then
        source_date="$(date -d "@${SOURCE_DATE_EPOCH}" "+%Y-%m-%d")"
        echo "VERGEN_GIT_COMMIT_DATE = \"${source_date}\"" >> .cargo/config.toml
    fi
    if [ -n "${SOURCE_GIT_HASH}" ]
    then
        echo "VERGEN_GIT_SHA = \"${SOURCE_GIT_HASH}\"" >> .cargo/config.toml
    fi
    tar pcf vendor.tar .cargo vendor
    rm -rf .cargo vendor

# Extracts vendored dependencies
vendor-extract:
    rm -rf vendor
    tar pxf vendor.tar
