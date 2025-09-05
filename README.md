<!--suppress HtmlDeprecatedAttribute -->
<div align="center">
  <br>
  <img alt="Quick Web Apps" src="https://raw.githubusercontent.com/cosmic-utils/web-apps/master/resources/icons/hicolor/256x256/apps/dev.heppen.webapps.png" width="192" />
  <h1>Quick Web Apps</h1>

  <p>Web App Manager for the COSMIC™ desktop written with love and libcosmic. Allow you to simply create web applications from given url working inside separate window. With some customization options.</p>

  <br>

  <img alt="Quick Web Apps" src="https://raw.githubusercontent.com/cosmic-utils/web-apps/refs/heads/master/resources/screenshots/window.png" width="512">

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
so I can work longer on it :) Thanks! :)

# Installation

Clone the repository:

`git clone https://github.com/cosmic-utils/web-apps.git`

cd into folder

`cd web-apps`

Building is simple. Make sure you have configured `flathub` remote as `--user`.

`flatpak remote-add --if-not-exists --user flathub https://dl.flathub.org/repo/flathub.flatpakrepo`

Install `flatpak-builder`.

`flatpak install -y flathub org.flatpak.Builder`

and start the process:

`flatpak run --command=flathub-build org.flatpak.Builder --install dev.heppen.webapps.json`

### Launching

`flatpak run dev.heppen.webapps`

### Uninstall

`flatpak uninstall dev.heppen.webapps`

# License

Code is distributed with [GPL-3.0 license](https://github.com/cosmic-utils/web-apps/blob/master/LICENSE)
