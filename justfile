export APPID := 'dev.heppen.webapps'
BINARY_PREFIX := 'dev-heppen-webapps'

prefix := '/usr/local'
base-dir := prefix

target-dir := 'target' / 'release'

webview := APPID + '.webview'
helper := APPID + '.webview-helper'

bin-src := target-dir / BINARY_PREFIX
webview-src := target-dir / (BINARY_PREFIX + '-webview')
helper-src := target-dir / (BINARY_PREFIX + '-webview-helper')

bin-dst := base-dir / 'bin' / APPID
webview-lib-dst := base-dir / 'lib' / APPID / webview
helper-lib-dst := base-dir / 'lib' / APPID / helper
webview-bin-dst := base-dir / 'bin' / webview

desktop-src := 'resources' / (APPID + '.desktop')
desktop-dst := base-dir / 'share/applications' / (APPID + '.desktop')

metainfo-src := 'resources' / (APPID + '.metainfo.xml')
metainfo-dst := base-dir / 'share/metainfo' / (APPID + '.metainfo.xml')

icons-src := 'resources/icons/hicolor'
icons-dst := base-dir / 'share/icons/hicolor'

# Default task
default: build

# Builds the project
build:
    cargo build --release 

# Checks the project
check:
    cargo check

# Runs tests
test:
    cargo test

# Runs the application
run: build
    {{bin-src}}

# Installs files
install:
    install -Dm0755 {{bin-src}} {{bin-dst}}
    install -Dm0755 {{webview-src}} {{webview-lib-dst}}
    install -Dm0755 {{helper-src}} {{helper-lib-dst}}
    install -Dm0644 {{desktop-src}} {{desktop-dst}}

    install -Dm0644 {{metainfo-src}} {{metainfo-dst}}

    for size in `ls {{icons-src}}`; do \
        install -Dm0644 "{{icons-src}}/$size/apps/{{APPID}}.png" "{{icons-dst}}/$size/apps/{{APPID}}.png"; \
    done

    mkdir -p {{base-dir}}/lib/{{APPID}}

    # Also copy from target where the build process downloads it
    find target -name "cef_linux_x86_64" -type d | head -n 1 | xargs -I {} cp -r {}/. {{base-dir}}/lib/{{APPID}}/
    
    # Create a symlink in bin to the webview in lib
    ln -sf ../lib/{{APPID}}/{{webview}} {{webview-bin-dst}}

# Uninstalls files
uninstall:
    rm -v {{bin-dst}}
    rm -v {{webview-bin-dst}}
    rm -v {{desktop-dst}}
    rm -v {{metainfo-dst}}

    rm -v {{icons-dst}}/*/apps/{{APPID}}.png

    rm -rv {{base-dir}}/lib/{{APPID}}

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
