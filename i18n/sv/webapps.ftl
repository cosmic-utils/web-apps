app=Snabba webbappar
loading=Laddar...
open=Öppna
number={ $number }
git-description = Git commit {$hash} på {$date}
delete=Radera
yes=Ja
no=Nej
confirm-delete=Är du säker på att du vill radera { $app }?
cancel=Avbryt
downloader-canceled=Installationen avbröts.
help=Hjälp
about=Om
support-me=Stöd mig
support-body=Om du tycker att det här programmet är användbart, överväg att stödja utvecklaren genom valfri donation :)
settings=Inställningar
import-theme=Importera tema
imported-themes=Importerade teman
run-app=Kör app
reset-settings=Återställ inställningar
reset=Återställ

# header
main-window={ $app }
view=Visa
create=Skapa
edit=Editera
close=Stäng
create-new-webapp=Skapa ny Webb App
icon-selector=Ikonväljare
icon-installer=Installationsprogram för Papirus ikoner

# common.rs
select-category=Välj kategori
select-browser=Välj webbläsare

# home_screen.rs
installed-header=Du har{ $number ->
        [1] 1 webbapp
        *[other] { $number} webappar
    } installerade:
not-installed-header=Du har ingen webbapp installerad. Vänligen tryck på skapa knappen och skapa en.

# creator.rs
web=Webb
accessories=Tillbehör
education=Utbildning
games=Spel
graphics=Grafik
internet=Internet
office=Kontor
programming=Programmering
sound-and-video=Ljud & Video

browser=Webbläsare

new-webapp-title=Ny Snabb webbapp

title=Titel
url=URL
download-favicon=Ladda ner favicon
non-standard-arguments=Icke-standardiserade argument
# behåll navigeringsfältet, isolerad profil och privat läge litet antal tecken
navbar=Navigeringsfältet
isolated-profile=Isolerad profil
private-mode=Privat läge

# iconpicker.rs
icon-name-to-find=Ikonnamn att hitta
my-icons=Mina ikoner
download=Ladda ner
search=Sök

# icons_installator.rs
icons-installer-header=Please wait. Laddar ner ikoner...
icons-installer-message=Detta program kräver ikoner att jobba med. Om vi ​​inte har tillgång till dina installerade ikoner, installerar vi Papirus ikonpaketet till en lokal katalog så att du kan välja en ikon för din webbapp från detta paket.
icons-installer-finished-waiting=Nedladdningen är klar. Väntar 3 sekunder på att stänga det här fönstret..


# warning.rs
warning=Du uppfyller inte kraven
    .success=Du kan skapa en ny webb app
    .duplicate=  - Webb app ogiltig. Kanske har du redan den här webbappen?
    .wrong-icon =  - Vald ikon är ogiltig. Välj en annan.
    .app-name=  - Appens namn måste vara längre än 3 tecken
    .app-url=  - Du måste ange en giltig webbadress som börjar med http:// eller https://
    .app-icon=  - Du måste välja en ikon för startprogrammet
    .app-browser=  - Var vänlig välj en webbläsare. Se till att minst en är installerad för hela systemet eller via Flatpak
