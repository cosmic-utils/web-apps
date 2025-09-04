<!--suppress HtmlDeprecatedAttribute -->
<div align="center">
  <br>
  <img alt="Quick Web Apps" src="https://raw.githubusercontent.com/cosmic-utils/web-apps/master/res/icons/hicolor/256x256/apps/dev.heppen.webapps.png" width="192" />
  <h1>Quick Web Apps</h1>

  <p>Web App Manager for the COSMIC™ desktop written with love and libcosmic. Allow you to simply create web applications from given url working inside separate window of your browser of choice.</p>

  <br>

  <img alt="Quick Web Apps Light window" src="https://raw.githubusercontent.com/cosmic-utils/web-apps/refs/tags/1.0.0/res/screenshots/window-light.png" width="256">
  <img alt="Quick Web Apps Dark window" src="https://raw.githubusercontent.com/cosmic-utils/web-apps/refs/tags/1.0.0/res/screenshots/window-dark.png" width="256">
  <img alt="Quick Web Apps Rose Pine window" src="https://raw.githubusercontent.com/cosmic-utils/web-apps/refs/tags/1.0.0/res/screenshots/window-rose-pine.png" width="256">

<br><br><br>

  <a href='https://flathub.org/apps/dev.heppen.webapps'>
    <img width='240' alt='Download on Flathub' src='https://flathub.org/api/badge?locale=en'/>
  </a>
</div>

# Support

Hey! This app is fully distributed for **free** with **free license**.
I'm doing it with **passion** in my **free time**.
Trying to keep it stable and bug free as long as I can.
However, would be nice if you could bring me some coffee,
so I can work longer on it :)

# Thanks

# Flatpak Usage

If you installed this app via flatpak, make sure your browser of choice have access to Quick Web Apps profiles directory. The default location for custom profiles is `~/.var/app/dev.heppen.webapps/data/quick-webapps/profiles`, so your browser of choice must have `read-write` permission to this directory. Some browser have full access to your `home` location, but otherwise, you need to set it manually.

For Mozilla Firefox, you can do this by running this command:

`flatpak override --filesystem=~/.var/app/dev.heppen.webapps/data/quick-webapps/profiles org.mozilla.firefox`

or if you have `user-wide` installation, add `--user` flag to command, like so:

`flatpak override --user --filesystem=~/.var/app/dev.heppen.webapps/data/quick-webapps/profiles org.mozilla.firefox`

# Installation

Clone the repository:

`git clone https://github.com/cosmic-utils/web-apps.git`

cd into folder

`cd web-apps`

## Just use, [just](https://github.com/casey/just)

For Pop OS make sure you have [just](https://github.com/casey/just) installed.

`sudo apt install just`

Make sure you are in `web-apps` directory. You should be already.

### Building

**NOTE:** Before installation you should build binary file.

### Build prerequisites

Before compilation please add needed dependencies to your system,
or make sure they are installed.

- pkg-config
- libssl-dev
- libxkbcommon-dev
- libwebkit2gtk-4.1-dev

for Pop OS you can install them via this command:

`sudo apt install pkg-config libssl-dev libxkbcommon-dev libwebkit2gtk-4.1-dev`

You need also rust compiler, so we recommend you tu use [rustup.rs](https://rustup.rs/).
Run this command to install full toolchain:

`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

and restart your shell.

**Run** this command and after it, you will be able to install
app.

`just`

### Installation

`sudo just install`

That's all. You can run `Quick Web Apps` from you app launcher.

### Uninstall

`sudo just uninstall`

# License

Code is distributed with [GPL-3.0 license](https://github.com/cosmic-utils/web-apps/blob/master/LICENSE)
