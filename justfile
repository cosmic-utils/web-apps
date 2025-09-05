export APPID := 'dev.heppen.webapps'
webview := APPID + '.webview'

rootdir := ''
prefix := '/app'

base-dir := absolute_path(clean(rootdir / prefix))

bin-src := 'target' / 'release' / 'dev-heppen-webapps'
bin-dst := base-dir / 'bin' / APPID

webview-src := 'target' / 'release' / 'dev-heppen-webapps-webview'
webview-dst := base-dir / 'bin' / webview

desktop := APPID + '.desktop'
desktop-src := 'resources' / desktop
desktop-dst := base-dir / 'share' / 'applications' / desktop

metainfo := APPID + '.metainfo.xml'
metainfo-src := 'resources' / metainfo
metainfo-dst := base-dir / 'share' / 'metainfo' / metainfo

icons-src := 'resources' / 'icons' / 'hicolor'
icons-dst := base-dir / 'share' / 'icons' / 'hicolor'

# Default recipe which runs `just build-release`
default: build-release

# Runs `cargo clean`
clean:
    cargo clean

# Compiles with debug profile
build-debug *args:
    cargo build {{args}}

# Compiles with release profile
build-release *args: (build-debug '--release' args)

# Compiles release profile with vendored dependencies
build-vendored *args: vendor-extract (build-release '--frozen --offline' args)

# Runs a clippy check
check *args:
    cargo clippy --all-features {{args}} -- -W clippy::pedantic

# Runs a clippy check with JSON message format
check-json: (check '--message-format=json')

dev *args:
    cargo fmt
    just run {{args}}

# Run with debug logs
run *args:
    env RUST_BACKTRACE=full cargo run {{args}}

# Installs files
install:
    install -Dm0755 {{bin-src}} {{bin-dst}}
    install -Dm0755 {{webview-src}} {{webview-dst}}
    install -Dm0644 {{desktop-src}} {{desktop-dst}}
    install -Dm0644 {{metainfo-src}} {{metainfo-dst}}

    for size in `ls {{icons-src}}`; do \
        install -Dm0644 "{{icons-src}}/$size/apps/{{APPID}}.png" "{{icons-dst}}/$size/apps/{{APPID}}.png"; \
    done

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
