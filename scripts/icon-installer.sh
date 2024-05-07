#!/bin/sh

set -e

gh_repo="papirus-icon-theme"
gh_desc="Papirus icon theme"

cat <<- EOF



      ppppp                         ii
      pp   pp     aaaaa   ppppp          rr  rrr   uu   uu     sssss
      ppppp     aa   aa   pp   pp   ii   rrrr      uu   uu   ssss
      pp        aa   aa   pp   pp   ii   rr        uu   uu      ssss
      pp          aaaaa   ppppp     ii   rr          uuuuu   sssss
                          pp
                          pp


  $gh_desc
  https://github.com/PapirusDevelopmentTeam/$gh_repo


EOF

: "${LOCAL_DESTDIR:=$HOME/.local/share/icons}"
: "${FLATPAK_DESTDIR:=$HOME/.var/app/io.github.elevenhsoft.WebApps/data/icons}"
: "${EXTRA_THEMES=ePapirus ePapirus-Dark Papirus-Dark Papirus-Light}"
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
    rm -f "$HOME/.cache/icon-cache.kcache"
    echo "Done!"
}


download

error_message=$(mkdir -p $LOCAL_DESTDIR 2>&1)

# Check if app is flatpak sandboxed
if [ -n "$FLATPAK_ID" ]; then
    echo "COSMIC Web Apps is probably sandboxed."
    echo "You do NOT have write permission on $LOCAL_DESTDIR."
    echo "Writing to $FLATPAK_DESTDIR."
    install $FLATPAK_DESTDIR Papirus "$EXTRA_THEMES"
else
    echo "COSMIC Web Apps is not sandboxed."
    echo "You have write permission on $LOCAL_DESTDIR."
    install $LOCAL_DESTDIR Papirus "$EXTRA_THEMES"
fi

trap cleanup EXIT HUP INT TERM
