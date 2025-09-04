app=Quick Web Apps
loading=Loading...
open=Open
number={ $number }
git-description = Git commit {$hash} on {$date}
delete=Delete
yes=Yes
no=No
confirm-delete=Are you sure you want to delete { $app }?
cancel=Cancel
downloader-canceled=Installing stopped.
help=Help
about=About
support-me=Support me
support-body=If you find this app useful, please consider to support author, by optional donation :)
settings=Settings
import-theme=Import theme
imported-themes=Imported themes
run-app=Run app
reset-settings=Reset settings
reset=Reset

# header
main-window={ $app }
view=View
create=Done
new-app=Create new
edit=Edit
close=Close
create-new-webapp=Create new Web App
icon-selector=Icon selector
icon-installer=Papirus Icons Installer

# common.rs
select-category=Select Category
select-browser=Select Browser

# home_screen.rs
installed-header=You have { $number ->
        [1] 1 web app
        *[other] { $number} web apps
    } installed:
not-installed-header=You don't have any web app installed. Please, press create button and create one.

# creator.rs
category=Category
web=Web
accessories=Accessories
education=Education
games=Games
graphics=Graphics
internet=Internet
office=Office
programming=Programming
sound-and-video=Sound & Video

browser=Browser

new-webapp-title=New Quick Web App
title=Title
url=URL
download-favicon=Download favicon
non-standard-arguments=Non-standard arguments
# keep navbar, isolated profile nad private mode small count of characters
navbar=Nav Bar
persistent-profile=Persistent Profile
private-mode=Private Mode
window-size=Window Size
decorations=Window Decorations
simulate-mobile=Try to simulate mobile device

# iconpicker.rs
icon-name-to-find=Icon name to find
my-icons=My icons
download=Download
search=Search

# icons_installator.rs
icons-installer-header=Please wait. Downloading icons...
icons-installer-message=This app requires icons to work with. In case we don't have access to your installed icons, we are installing Papirus icon pack to local directory so you can choose one icon for your web app from this pack.
icons-installer-finished-waiting=Downloading finished. Waiting 3 seconds to close this window..

# warning.rs
warning=You don't meet requirements
    .success=You can create new Web App
    .duplicate=  - Web App invalid. Maybe you already have this Web App?
    .wrong-icon =  - Selected icon is invalid. Select another one.
    .app-name=  - App name must be longer than 3 characters
    .app-url=  - You must provide valid URL starting with http:// or https://
    .app-icon=  - You must select an Icon for your launcher
    .app-browser=  - Please select a browser. Make sure at least one is installed system-wide or via Flatpak
