app=Quick Web Apps
loading=読み込み中...
open=開く
number={ $number }
git-description = {$date}のGitコミット（{$hash}）
delete=削除
yes=はい
no=いいえ
confirm-delete={ $app }を削除してもよろしいですか？
cancel=キャンセル
downloader-canceled=インストールを中止しました。
about=このアプリについて
menu-about=Quick Web Appsについて...
menu-settings=設定...
support-me=作者を支援
support-body=このアプリがお役に立ちましたら、任意の寄付で作者を支援することをご検討ください。:)
settings=設定
import-theme=テーマをインポート
imported-themes=インポート済みのテーマ
run-app=アプリを起動
reset-settings=設定をリセット
reset=リセット
generate-icon=アイコンを生成
reset-icon=アイコンをリセット

repository = リポジトリ
support = サポート
comment = COSMIC™デスクトップ用ウェブアプリマネージャー

# header
main-window={ $app }
view=表示
create=完了
new-app=新規作成
quick-web-app=クイック起動
edit=編集
close=閉じる
create-new-webapp=新しいウェブアプリを作成
icon-selector=アイコンの選択
icon-installer=Papirusアイコンインストーラー

# common.rs
select-category=カテゴリーを選択
select-browser=ブラウザーを選択

# home_screen.rs
installed-header=ウェブアプリが{ $number ->
        [1] 1個
        *[other] { $number }個
    }インストールされています:
not-installed-header=インストールされているウェブアプリはありません。作成ボタンを押して、1つ作成してください。

# creator.rs
category=カテゴリー
web=ウェブ
accessories=アクセサリ
education=教育
games=ゲーム
graphics=グラフィックス
internet=インターネット
office=オフィス
programming=プログラミング
sound-and-video=サウンドとビデオ

browser=ブラウザー

new-webapp-title=新しいQuick Web App
title=タイトル
url=URL
download-favicon=ファビコンをダウンロード
non-standard-arguments=標準以外の引数
# keep navbar, isolated profile nad private mode small count of characters
navbar=ナビバー
persistent-profile=永続プロファイル
isolated-profile=分離プロファイル
private-mode=プライベートモード
window-size=ウィンドウサイズ
decorations=ウィンドウの装飾
simulate-mobile=モバイルデバイスのシミュレーションを試みる

# iconpicker.rs
icon-name-to-find=検索するアイコン名
my-icons=マイアイコン
download=ダウンロード
search=検索

# icons_installator.rs
icons-installer-header=しばらくお待ちください。アイコンをダウンロードしています...
icons-installer-message=このアプリの動作にはアイコンが必要です。インストール済みのアイコンにアクセスできない場合に備えて、Papirusアイコンパックをローカルフォルダーにインストールしています。このパックの中から、ウェブアプリ用のアイコンを1つ選択できます。
icons-installer-finished-waiting=ダウンロードが完了しました。3秒後にこのウィンドウを閉じます。

# warning.rs
warning=要件を満たしていません
    .success=新しいウェブアプリを作成できます。
    .duplicate=  - ウェブアプリが無効です。すでに同じウェブアプリを作成していませんか？
    .wrong-icon =  - 選択したアイコンが無効です。別のアイコンを選択してください。
    .app-name=  - アプリ名は3文字より長くする必要があります。
    .app-url=  - http://またはhttps://で始まる有効なURLを指定する必要があります。
    .app-icon=  - ランチャーのアイコンを選択する必要があります。
    .app-browser=  - ブラウザーを選択してください。システム全体またはFlatpak経由で、少なくとも1つインストールされていることを確認してください。

# mod.rs
permissions-warning=権限に関する警告
permissions-body=サンドボックス環境で、分離プロファイルを使ったウェブアプリを作成しようとしています。場合によっては、読み取り専用と思われるこのパスにファイルを作成する必要があります。続行する場合は、このパス[ { $path } ]への書き込みを許可してください。Flatpak用の次のいずれかのコマンドで許可できます。その後、アプリを再起動する必要があります。