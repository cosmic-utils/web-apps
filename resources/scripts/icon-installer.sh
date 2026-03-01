#!/bin/sh

set -e

APP_ID="dev.heppen.webapps"

gh_repo="papirus-icon-theme"
gh_desc="Papirus icon theme"

: "${XDG_DATA_HOME:=$HOME/.local/share}"
: "${EXTRA_THEMES=Papirus Papirus-Dark Papirus-Light}"
: "${TAG:=master}"

temp_file="$(mktemp -u)"
temp_dir="$(mktemp -d)"

download() {
    echo "Getting the latest version from GitHub ..."
    wget -O "$temp_file" \
        "https://github.com/PapirusDevelopmentTeam/$gh_repo/archive/$TAG.tar.gz"
    echo "Unpacking archive ..."
    tar -xzf "$temp_file" -C "$temp_dir"
}


install() {
    # shellcheck disable=2068
    set -- $@  # split args by space

    for theme in "$@"; do
        test -d "$temp_dir/$gh_repo-$TAG/$theme" || continue
        echo "Installing '$theme' ..."
        cp -R "$temp_dir/$gh_repo-$TAG/$theme" $1
    done
}

cleanup() {
    echo "Clearing cache ..."
    rm -rf "$temp_file" "$temp_dir"
    echo "Done!"
}


download

install_path="$XDG_DATA_HOME/$APP_ID/icons"

error_message=$(mkdir -p $install_path 2>&1)

install $install_path $EXTRA_THEMES

trap cleanup EXIT HUP INT TERM
