use std::{
    fs::{self, create_dir_all, remove_dir_all, remove_file, File},
    io::{self, BufRead, Write},
    path::PathBuf,
};

use anyhow::{anyhow, Error};
use rand::{thread_rng, Rng};

use crate::{
    browser,
    common::{self, home_dir},
    launcher,
};

pub fn webapplauncher_is_valid(
    webbrowser: &browser::Browser,
    icon: &str,
    codename: &str,
    name: &str,
    url: &str,
) -> bool {
    let installed = get_webapps();

    for app in installed.iter().flatten() {
        if !common::url_valid(url)
            || !webbrowser.is_installed()
            || (name.is_empty() || app.name == name)
            || icon.is_empty()
            || (codename.is_empty() || app.codename == codename)
            || url.is_empty()
        {
            return false;
        }
    }

    true
}

pub fn get_webapps() -> Vec<anyhow::Result<launcher::WebAppLauncher>> {
    let mut webapps = Vec::new();

    match fs::read_dir(common::desktop_filepath("")) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let entry_fn = entry.file_name();
                        let filename = entry_fn.to_str().unwrap();

                        if filename.starts_with("webapp-") && filename.ends_with(".desktop") {
                            let codename = filename.replace("webapp-", "").replace(".desktop", "");

                            let launcher = WebAppLauncher::read(entry.path(), codename);
                            webapps.push(launcher);
                        }
                    }
                    Err(e) => tracing::error!("Error reading directory: {}", e),
                }
            }
        }
        Err(_) => {
            create_dir_all(common::desktop_filepath(""))
                .expect("Cannot create local applications dir");
        }
    }

    webapps
}

#[derive(Debug, Clone)]
pub struct WebAppLauncher {
    pub path: PathBuf,
    pub codename: String,
    pub web_browser: browser::Browser,
    pub name: String,
    pub icon: String,
    pub exec: String,
    // pub args: Vec<String>,
    pub category: String,
    pub url: String,
    pub custom_parameters: String,
    pub isolate_profile: bool,
    pub navbar: bool,
    pub is_incognito: bool,
}

#[allow(clippy::too_many_arguments)]
impl WebAppLauncher {
    pub fn new(
        name: String,
        codename: Option<String>,
        url: String,
        icon: String,
        category: String,
        browser: browser::Browser,
        custom_parameters: String,
        isolated: bool,
        navbar: bool,
        privatewindow: bool,
    ) -> Self {
        let codename = if let Some(codename) = codename {
            codename
        } else {
            let random_code: u16 = thread_rng().gen_range(1000..10000);
            format!("{}{}", name.replace(' ', ""), random_code)
        };
        let filename = format!("webapp-{}.desktop", codename);
        let path = common::desktop_filepath(&filename);
        let web_browser = browser;
        let exec = match web_browser._binary {
            browser::BinaryLocation::System => web_browser.exec.clone(),
            browser::BinaryLocation::Nix => web_browser.exec.clone(),
            browser::BinaryLocation::Snap => web_browser.exec.clone(),
            browser::BinaryLocation::FlatpakLocal => {
                format!(
                    "{}/.local/share/flatpak/exports/bin/{}",
                    home_dir().to_str().unwrap_or_default(),
                    web_browser.exec.clone()
                )
            }
            browser::BinaryLocation::FlatpakSystem => {
                format!("/var/lib/flatpak/exports/bin/{}", web_browser.exec.clone())
            }
        };
        // let args = Vec::new();
        let isolate_profile = isolated;
        let is_incognito = privatewindow;

        Self {
            path,
            codename,
            web_browser,
            name,
            icon,
            exec,
            // args,
            category,
            url,
            custom_parameters,
            isolate_profile,
            navbar,
            is_incognito,
        }
    }

    pub fn read(path: PathBuf, codename: String) -> Result<WebAppLauncher, Error> {
        let file = File::open(&path)?;
        let mut browser_name = String::new();
        let mut name = String::new();
        let mut icon = String::new();
        let mut exec = String::new();
        let mut args = Vec::new();
        let mut category = String::new();
        let mut url = String::new();
        let mut custom_parameters = String::new();
        let mut isolate_profile = false;
        let mut navbar = false;
        let mut is_incognito = false;

        let reader = io::BufReader::new(file);

        for line_result in reader.lines() {
            match line_result {
                Ok(line) => {
                    if line.contains("Name=") {
                        name = line.replace("Name=", "");
                    };

                    if line.contains("Icon=") {
                        icon = line.replace("Icon=", "");
                    };

                    if line.contains("Exec=") {
                        exec = line.replace("Exec=", "");
                    };

                    if line.contains("Categories=") {
                        category = line
                            .replace("Categories=", "")
                            .replace("GTK;", "")
                            .replace(';', "");
                    };

                    if line.contains("X-WebApp-Browser=") {
                        browser_name = line.replace("X-WebApp-Browser=", "");
                    };

                    if line.contains("X-WebApp-URL=") {
                        url = line.replace("X-WebApp-URL=", "");
                    };

                    if line.contains("X-WebApp-CustomParameters=") {
                        custom_parameters = line.replace("X-WebApp-CustomParameters=", "");
                    };

                    if line.contains("X-WebApp-Isolated=") {
                        isolate_profile = line.replace("X-WebApp-Isolated=", "") == "true"
                    };

                    if line.contains("X-WebApp-Navbar=") {
                        navbar = line.replace("X-WebApp-Navbar=", "") == "true"
                    };

                    if line.contains("X-WebApp-PrivateWindow=") {
                        is_incognito = line.replace("X-WebApp-PrivateWindow=", "") == "true"
                    };
                }
                Err(e) => eprintln!("Error reading line: {}", e),
            }
        }

        let web_browser = browser::Browser::web_browser(browser_name);

        match web_browser {
            Some(web_browser) => {
                exec.split(' ').enumerate().for_each(|(n, arg)| {
                    if n > 0 && !arg.is_empty() {
                        args.push(arg.to_string())
                    }
                });

                Ok(WebAppLauncher {
                    path,
                    codename,
                    web_browser,
                    name,
                    icon,
                    exec,
                    // args,
                    category,
                    url,
                    custom_parameters,
                    isolate_profile,
                    navbar,
                    is_incognito,
                })
            }
            None => Err(anyhow!("Cannot read web app launcher.")),
        }
    }

    fn create_firefox_userjs(
        &self,
        is_zen_browser: bool,
        path: PathBuf,
        create_navbar: bool,
    ) -> bool {
        let mut file = File::create(&path)
            .unwrap_or_else(|_| panic!("failed to create user.js in {:?}", path));

        let navbar_pref = if create_navbar {
            b"user_pref(\"browser.tabs.inTitlebar\", 2);\n"
        } else {
            b"user_pref(\"browser.tabs.inTitlebar\", 0);\n"
        };
        let Ok(_) = file.write_all(navbar_pref) else {
            return false;
        };

        match is_zen_browser {
            true => file
                .write_all(include_bytes!(
                    "../data/runtime/zen-browser/profile/user.js"
                ))
                .is_ok(),
            false => file
                .write_all(include_bytes!("../data/runtime/firefox/profile/user.js"))
                .is_ok(),
        }
    }

    fn create_user_chrome_css(
        &self,
        is_zen_browser: bool,
        path: PathBuf,
        create_navbar: bool,
    ) -> bool {
        let mut file = File::create(&path)
            .unwrap_or_else(|_| panic!("cant create userChrome.css in {:?}", path));

        if create_navbar {
            file.write_all(b"").is_ok()
        } else {
            match is_zen_browser {
                true => file
                    .write_all(include_bytes!(
                        "../data/runtime/zen-browser/profile/chrome/userChrome.css"
                    ))
                    .is_ok(),
                false => file
                    .write_all(include_bytes!(
                        "../data/runtime/firefox/profile/chrome/userChrome.css"
                    ))
                    .is_ok(),
            }
        }
    }

    fn exec_firefox(&self, is_zen_browser: bool) -> String {
        let profile_path = self.web_browser.profile_path.join(&self.codename);
        let user_js_path = profile_path.join("user.js");
        let mut user_chrome_css = profile_path.join("chrome");

        tracing::info!("Creating profile directory in: {:?}", &profile_path);
        create_dir_all(&profile_path)
            .unwrap_or_else(|_| panic!("cant create profile dir in {:?}", &profile_path));
        create_dir_all(&user_chrome_css)
            .unwrap_or_else(|_| panic!("cant create chrome dir in {:?}", &user_chrome_css));

        user_chrome_css = user_chrome_css.join("userChrome.css");

        self.create_firefox_userjs(is_zen_browser, user_js_path, self.navbar);
        self.create_user_chrome_css(is_zen_browser, user_chrome_css, self.navbar);

        let profile_path = profile_path.to_str().unwrap();

        let mut exec_string = format!(
            "{} --class WebApp-{} --name WebApp-{} --profile {} --no-remote ",
            self.exec, self.codename, self.codename, profile_path
        );

        if self.is_incognito {
            exec_string.push_str("--private-window ");
        }

        if !self.custom_parameters.is_empty() {
            exec_string.push_str(&format!("{} ", self.custom_parameters));
        }

        exec_string.push_str(&self.url);

        exec_string
    }

    fn exec_chromium(&self) -> String {
        let mut exec_string = format!(
            "{} --app={} --class=WebApp-{} --name=WebApp-{} ",
            self.exec, self.url, self.codename, self.codename
        );

        if self.isolate_profile {
            let profile_dir = self.web_browser.profile_path.join(&self.codename);

            tracing::info!("Creating profile directory in: {:?}", &profile_dir);
            let _ = create_dir_all(&profile_dir);
            let profile_path = profile_dir.to_str().unwrap();
            exec_string.push_str(&format!("--user-data-dir={} ", profile_path));
        }

        if self.is_incognito {
            if self.web_browser.name.starts_with("Microsoft Edge") {
                exec_string.push_str("--inprivate ");
            } else {
                exec_string.push_str("--incognito ");
            }
        }

        if !self.custom_parameters.is_empty() {
            exec_string.push_str(&format!("{} ", self.custom_parameters));
        }

        exec_string
    }

    fn exec_falkon(&self) -> String {
        let mut exec_string = String::new();

        if self.isolate_profile {
            let profile_dir = self.web_browser.profile_path.join(&self.codename);
            tracing::info!("Creating profile directory in: {:?}", &profile_dir);
            let _ = create_dir_all(&profile_dir);

            let profile_path = profile_dir.to_str().unwrap();

            exec_string = format!(
                "{} --portable --wmclass WebApp-{} --profile {} ",
                self.exec, self.codename, profile_path
            );
        }

        if self.is_incognito {
            exec_string.push_str("--private-browsing ");
        }

        if !self.custom_parameters.is_empty() {
            exec_string.push_str(&format!("{} ", self.custom_parameters));
        }

        exec_string.push_str(&format!("--no-remote --current-tab {}", self.url));

        exec_string
    }

    fn exec_string(&self) -> String {
        use browser::BrowserType::*;
        match self.web_browser._type {
            Firefox | FirefoxFlatpak => self.exec_firefox(false),
            Zen | ZenFlatpak => self.exec_firefox(true),
            Chromium | ChromiumFlatpak => self.exec_chromium(),
            Falkon | FalkonFlatpak => self.exec_falkon(),
            NoBrowser => String::new(),
        }
    }

    pub fn create(&self) -> anyhow::Result<()> {
        let mut output = File::create(&self.path)?;

        writeln!(output, "[Desktop Entry]")?;
        writeln!(output, "Version=1.0")?;
        writeln!(output, "Name={}", self.name)?;
        writeln!(output, "Comment=Web App")?;
        writeln!(output, "Exec={}", self.exec_string())?;
        writeln!(output, "Terminal=false")?;
        writeln!(output, "Type=Application")?;
        writeln!(output, "Icon={}", self.icon)?;
        writeln!(output, "Categories=GTK;{};", self.category)?;
        writeln!(output, "MimeType=text/html;text/xml;application/xhtml_xml;")?;
        writeln!(output, "StartupWMClass=WebApp-{}", self.codename)?;
        writeln!(output, "StartupNotify=true")?;
        writeln!(output, "X-MultipleArgs=false")?;
        writeln!(output, "X-WebApp-Browser={}", self.web_browser.name)?;
        writeln!(output, "X-WebApp-URL={}", self.url)?;
        writeln!(output, "X-WebApp-Navbar={}", self.navbar)?;
        writeln!(output, "X-WebApp-PrivateWindow={}", self.is_incognito)?;
        writeln!(output, "X-WebApp-Isolated={}", self.isolate_profile)?;
        writeln!(
            output,
            "X-WebApp-CustomParameters={}",
            self.custom_parameters
        )?;

        Ok(())
    }

    pub fn remove_desktop_file(&self) -> anyhow::Result<()> {
        let exist = self.path.as_path().exists();

        match exist {
            true => {
                remove_file(&self.path)?;
            }
            false => {
                tracing::error!("file not found");
            }
        }

        Ok(())
    }

    pub fn delete(&self) -> anyhow::Result<()> {
        let exist = self.path.as_path().exists();

        match exist {
            true => {
                remove_file(&self.path)?;
            }
            false => {
                tracing::error!("file not found");
            }
        }

        let profile_path = self.web_browser.profile_path.join(&self.codename);

        if remove_dir_all(&profile_path).is_ok() {
            tracing::info!(
                "Removed profile directory from: {}",
                profile_path.to_str().unwrap()
            );
        };

        Ok(())
    }
}
