<!--suppress HtmlDeprecatedAttribute -->
<div align="center">
  <br>
  <img alt="Quick Web Apps" src="https://raw.githubusercontent.com/cosmic-utils/web-apps/master/res/icons/hicolor/256x256/apps/io.github.elevenhsoft.WebApps.png" width="192" />
  <h1>Quick Web Apps</h1>

<h3>Web App Manager for the COSMIC™ desktop written with love and libcosmic. Allow you to simply create web applications
from
given url working inside separate window of your browser of choice.</h3>

  <br>

  <img alt="Quick Web Apps Dark window" src="https://raw.githubusercontent.com/cosmic-utils/web-apps/refs/tags/0.5.4a/res/screenshots/window-light.png" width="192">
  <img alt="Quick Web Apps Dark window" src="https://raw.githubusercontent.com/cosmic-utils/web-apps/refs/tags/0.5.4a/res/screenshots/window-dark.png" width="192">

  <br>

  <img alt="Quick Web Apps Creator" src="https://raw.githubusercontent.com/cosmic-utils/web-apps/refs/tags/0.5.4a/res/screenshots/window-creator.png" width="192">
  <img alt="Quick Web Apps Icon selector" src="https://raw.githubusercontent.com/cosmic-utils/web-apps/refs/tags/0.5.4a/res/screenshots/window-icon-picker.png" width="192">

<br><br><br>

  <a href='https://flathub.org/apps/io.github.elevenhsoft.WebApps'>
    <img width='240' alt='Download on Flathub' src='https://flathub.org/api/badge?locale=en'/>
  </a>
</div>

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

Before compilation please add needed dependencies to your system, or make sure they are installed.

- rustc
- cargo
- pkg-config
- libssl-dev

for Pop OS you can install them via this command:

`sudo apt install rustc cargo pkg-config libssl-dev libxkbcommon-dev`

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

# Credits

Special thanks to Linux Mint team for inspiration and solutions in this awesome
app: [webapp-manager](https://github.com/linuxmint/webapp-manager)
