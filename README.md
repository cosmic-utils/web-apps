<!--suppress HtmlDeprecatedAttribute -->
<div align="center">
  <br>
  <img alt="COSMIC Web Apps" src="https://raw.githubusercontent.com/elevenhsoft/WebApps/master/res/icons/hicolor/256x256/apps/io.github.elevenhsoft.WebApps.svg" width="192" />
  <h1>COSMIC Web Apps</h1>

  <h3>Web App Manager for Cosmic desktop written with love and libcosmic. Allow you to simply create web applications from
given url working inside separate window of your browser of choice.</h3>

  <br>

  <img alt="COSMIC Web Apps Dark window" src="https://github.com/elevenhsoft/WebApps/blob/master/res/screenshots/window-dark.png" width="192">
  <img alt="COSMIC Web Apps Creator" src="https://github.com/elevenhsoft/WebApps/blob/master/res/screenshots/window-creator.png" width="192">
  <img alt="COSMIC Web Apps Icon selector" src="https://github.com/elevenhsoft/WebApps/blob/master/res/screenshots/window-icon-picker.png" width="192">

  <br><br><br>

  <a href='https://flathub.org/apps/io.github.elevenhsoft.WebApps'>
    <img width='240' alt='Download on Flathub' src='https://flathub.org/api/badge?locale=en'/>
  </a>
</div>

# BREAKING CHANGE >= 0.4.2

Starting with version `0.4.2` we moved some locations. Icons are now stored (and also downloaded) to `~/.local/share/icons` directory, COSMIC Web Apps specific directory for storing icons for will be placed in `~/.local/share/icons/CosmicWebApps` path. The pros of this change is that, this location is standard for Linux to user-wide icons installation, so if you have more icons pack installed on your system, they can be used in COSMIC Web Apps.

Profiles directories are now stored also in one place. This is `~/.local/share/cosmic-webapps path`. There will be created directory for browser (firefox/chromium) and there will be stored profiles for your web apps. Unfortunately, this require some manual intervention. Since, flatpak is sandboxed, it's require permissions for everything. We can't control others app permissions, so user must run one command, to allow reading and writing to our directory.

## Example

For Mozilla Firefox, run this command:

`flatpak override --filesystem=~/.local/share/cosmic-webapps org.mozilla.firefox`

If you aren't familiar with cli and terminal, still you can use GUI for this process, just use this awesome tool: [Flatseal](https://github.com/tchx84/Flatseal)


Sorry for this. It's the only way to keep this app compatible with `just` method installation (system-wide) and flatpak and I know, I'm not the only person who use both options. Thank you :)

# Installation

Clone the repository:

`git clone https://github.com/elevenhsoft/WebApps.git`

cd into folder

`cd WebApps`

## Just use, [just](https://github.com/casey/just)

For Pop OS make sure you have [just](https://github.com/casey/just) installed.

`sudo apt install just`

Make sure you are in `WebApps` directory. You should be already.

### Building

**NOTE:** Before installation you should build binary file.

### Build prerequisites

Before compilation please add needed dependencies to your system, or make sure they are installed.

- rustc
- cargo
- pkg-config
- libssl-dev

for Pop OS you can install them via this command:

`sudo apt install rustc cargo pkg-config libssl-dev`

**Run** this command and after it, you will be able to install
app.

`just`

### Installation

`sudo just install`

That's all. You can run `COSMIC Web Apps` from you app launcher.

### Uninstall

`sudo just uninstall`

# License

Code is distributed with [GPL-3.0 license](https://github.com/elevenhsoft/WebApps/blob/master/LICENSE)

# Credits

Special thanks to Linux Mint team for inspiration and solutions in this awesome
app: [webapp-manager](https://github.com/linuxmint/webapp-manager)
