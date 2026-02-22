//! Rust port of the [`cefsimple`](https://github.com/chromiumembedded/cef/tree/master/tests/cefsimple) example.

use cef::*;
use clap::Parser as _;
use webapps::{MOBILE_UA, WebviewArgs};

pub mod simple_app;
pub mod simple_handler;

pub struct Library;

#[allow(dead_code)]
pub fn load_cef() -> Library {
    let library = Library;

    // Initialize the CEF API version.
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);

    library
}

#[allow(dead_code)]
pub fn run_main(main_args: &MainArgs, cmd_line: &CommandLine, sandbox_info: *mut u8) {
    let switch = CefString::from("type");
    let is_browser_process = cmd_line.has_switch(Some(&switch)) != 1;

    let ret = execute_process(Some(main_args), None, sandbox_info);

    if is_browser_process {
        println!("launch browser process");
        assert_eq!(ret, -1, "cannot execute browser process");
    } else {
        let process_type = CefString::from(&cmd_line.switch_value(Some(&switch)));
        println!("launch process {process_type}");
        assert!(ret >= 0, "cannot execute non-browser process");
        // non-browser process does not initialize cef
        return;
    }

    let mut app = simple_app::SimpleApp::new();

    let helper_path = webapps::helper_bin();

    let args = WebviewArgs::parse();

    let Some(browser_config) = crate::Browser::from_appid(&args.id) else {
        return;
    };

    let mobile_ua = browser_config.try_simulate_mobile.unwrap_or(false);
    let Some(root_cache_path) = browser_config.profile else {
        return;
    };

    let path = root_cache_path.join("cache");
    let cache_path = CefString::from(path.display().to_string().as_str());
    let root_cache_path = CefString::from(root_cache_path.display().to_string().as_str());

    let mut settings = Settings {
        no_sandbox: 1,
        browser_subprocess_path: CefString::from(helper_path.as_str()),
        root_cache_path,
        cache_path,
        user_agent: if mobile_ua {
            CefString::from(MOBILE_UA)
        } else {
            Default::default()
        },
        ..Default::default()
    };

    if let Some(res_path) = webapps::cef_resources_path() {
        settings.resources_dir_path = CefString::from(res_path.to_str().unwrap());
        settings.locales_dir_path = CefString::from(res_path.join("locales").to_str().unwrap());
    }

    assert_eq!(
        initialize(
            Some(main_args),
            Some(&settings),
            Some(&mut app),
            sandbox_info,
        ),
        1
    );

    run_message_loop();

    shutdown();
}
