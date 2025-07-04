app=Quick Web Apps
loading=Carregando...
open=Abrir
number={ $number }
git-description = Git commit {$hash} de {$date}
delete=Excluir
yes=Sim
no=Não
confirm-delete=Tem certeza de que deseja excluir { $app }?
cancel=Cancelar
downloader-canceled=Instalação interrompida.
help=Ajuda
about=Sobre
support-me=Apoie-me
support-body=Se você achar este aplicativo útil, considere apoiar o autor, por meio de uma doação opcional :)
settings=Configurações
import-theme=Importar tema
imported-themes=Temas importador
run-app=Executar aplicativo
reset-settings=Redefinir configurações
reset=Redefinir

# header
main-window={ $app }
view=Exibir
create=Concluir
new-app=Criar novo
edit=Editar
close=Fechar
create-new-webapp=Criar novo Aplicativo Web
icon-selector=Seletor de ícone
icon-installer=Instalador de Ícones Papirus

# common.rs
select-category=Selecionar Categoria
select-browser=Selecionar Navegador

# home_screen.rs
installed-header=Você tem { $number ->
        [1] 1 aplicativo web
        *[other] { $number} aplicativos web
    } instalados:
not-installed-header=Você não tem nenhum aplicativo web instalado. Por favor, pressione o botão de criar para instalar um novo app.

# creator.rs
web=Web
accessories=Acessórios
education=Educação
games=Jogos
graphics=Gráficos
internet=Internet
office=Escritório
programming=Desenvolvimento
sound-and-video=Som e Vídeo

browser=Navegador

new-webapp-title=Novo Aplicativo Web
title=Título
url=URL
download-favicon=Baixar ícone do app
non-standard-arguments=Argumentos não padronizados
# keep navbar, isolated profile nad private mode small count of characters
navbar=Barra de Navegação
isolated-profile=Perfil Isolado
private-mode=Modo Privado

# iconpicker.rs
icon-name-to-find=Nome do ícone de busca
my-icons=Meus ícones
download=Download
search=Procurar

# icons_installator.rs
icons-installer-header=Por favor, aguarde. Baixando ícones...
icons-installer-message=Este aplicativo requer ícones para funcionar. Caso não tenhamos acesso aos seus ícones instalados, estamos instalando o pacote de ícones Papirus no diretório local para que você possa escolher um ícone para o seu aplicativo web deste pacote.
icons-installer-finished-waiting=Download concluído. Aguardando 3 segundos para fechar esta janela...

# warning.rs
warning=Requisitos não atendidos
    .success=Você pode criar um novo aplicativo Web
    .duplicate=  - Aplicativo web inválido. Talvez você já tenha este aplicativo instalado.
    .wrong-icon =  - Ícone selecionado é inválido. Selecione outro ícone.
    .app-name=  - O nome do aplicativo deve ter mais de 3 caracteres
    .app-url=  -  Você deve fornecer uma URL válida começando com http:// ou https://
    .app-icon=  - Você deve selecionar um ícone para seu lançador
    .app-browser=  - Por favor, selecione um aplicativo. Certifique-se de que pelo menos um esteja instalado em todo o sistema ou via Flatpak
