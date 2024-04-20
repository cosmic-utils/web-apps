name := 'cosmic-wam'
export APPID := 'org.cosmic.Wam'

rootdir := ''
prefix := '/usr/local'

base-dir := absolute_path(clean(rootdir / prefix))

export INSTALL_DIR := base-dir / 'share'

bin-src := 'target' / 'release' / name
bin-dst := base-dir / 'bin' / name

desktop := APPID + '.desktop'
desktop-src := 'data' / desktop
desktop-dst := clean(rootdir / prefix) / 'share' / 'applications' / desktop

icon-src := 'data' / APPID + '.png'
icon-dst := clean(rootdir / prefix) / 'share' / 'icons' / APPID + '.png'

runtime-dst := INSTALL_DIR / name

firefox-src := 'data' / 'runtime' / 'firefox'
firefox-dst := runtime-dst / 'runtime' / 'firefox'

profile-src := 'data' / 'runtime' / 'firefox' / 'profile'
profile-dst := runtime-dst / 'profile'

chrome-src := profile-src / 'chrome'
chrome-dst := profile-dst / 'chrome'

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
    env RUST_LOG=cosmic_wam=info RUST_BACKTRACE=full cargo run --release {{args}}

# Installs files
install:
     install -Dm0755 {{bin-src}} {{bin-dst}}
     install -Dm0644 {{desktop-src}} {{desktop-dst}}
     install -Dm0644 {{icon-src}} {{icon-dst}}

     # install firefox profile
     for file in `ls {{profile-src}}`; do \
     	install -Dm0644 "{{profile-src}}/$file" "{{profile-dst}}/$file"; \
     done

     for file in `ls {{chrome-src}}`; do \
     	install -Dm0644 "{{chrome-src}}/$file" "{{chrome-dst}}/$file"; \
     done


# Uninstalls installed files
uninstall:
    rm {{bin-dst}}
    rm {{desktop-dst}}
    rm {{icon-dst}}
    rm -r {{runtime-dst}}

