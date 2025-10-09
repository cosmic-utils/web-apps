use std::env;

use clap::Parser;
use gtk4::ApplicationWindow;
use gtk4::glib::GString;
use gtk4::prelude::*;
use webapps::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, WindowSize, browser::Browser, fl};
use webkit6::DeviceInfoPermissionRequest;
use webkit6::NotificationPermissionRequest;
use webkit6::UserMediaPermissionRequest;
use webkit6::prelude::*;
use webkit6::{Settings, UserContentManager, WebContext, WebView};

fn main() {
    let args = webapps::WebviewArgs::parse();

    unsafe {
        // workaround for webkitgtk sandboxing issues
        env::set_var("WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS", "1");
    }

    gtk4::init().expect("Failed to initialize GTK");

    gtk4::glib::set_program_name(args.id.clone().into());
    gtk4::glib::set_application_name(&args.id);

    let app = gtk4::Application::builder()
        .application_id(args.id.clone())
        .build();

    app.connect_activate(move |app| {
        if let Some(ref browser) = Browser::from_appid(&args.id) {
            let window_size = match browser.window_size {
                Some(ref size) => size,
                None => &WindowSize(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT),
            };

            let window_title = match browser.window_title {
                Some(ref title) => title,
                None => &fl!("app"),
            };

            let window_decorations = browser.window_decorations.unwrap_or(true);

            let window = ApplicationWindow::builder()
                .application(app)
                .title(window_title)
                .default_width(window_size.0 as i32)
                .default_height(window_size.1 as i32)
                .build();

            window.set_decorated(window_decorations);

            let context_builder = WebContext::builder();
            // if let Some(data_directory) = browser.profile.as_ref() {
            //     let data_manager = WebsiteDataManager::builder()
            //         .base_data_directory(data_directory.to_string_lossy())
            //         .build();
            //     if let Some(cookie_manager) = data_manager.cookie_manager() {
            //         cookie_manager.set_persistent_storage(
            //             &data_directory.join("cookies").to_string_lossy(),
            //             CookiePersistentStorage::Text,
            //         );
            //     }
            //     context_builder = context_builder.website_data_manager(&data_manager);
            // }

            let context = context_builder.build();

            // Create WebView with custom settings
            let settings = Settings::new();
            settings.set_enable_javascript(true);

            if let Some(true) = browser.try_simulate_mobile {
                settings.set_user_agent(Some(webapps::MOBILE_UA));
            }

            if let Some(flag) = browser.with_devtools {
                settings.set_enable_developer_extras(flag);
            }

            settings.set_enable_webgl(true);
            settings.set_enable_webaudio(true);

            // Enable clipboard
            settings.set_javascript_can_access_clipboard(true);
            settings.set_enable_write_console_messages_to_stdout(true);

            // Enable App cache
            settings.set_enable_page_cache(true);

            // Devtools
            if browser.with_devtools.unwrap_or(false) {
                settings.set_enable_developer_extras(true);
            };

            let builder = WebView::builder()
                .user_content_manager(&UserContentManager::new())
                .web_context(&context)
                .build();

            builder.set_settings(&settings);

            let Some(url) = browser.url.as_ref() else {
                eprintln!("No URL specified in browser configuration.");
                return;
            };
            builder.load_uri(url);

            builder.connect_permission_request(move |_, request| {
                if let Some(notification_request) =
                    request.downcast_ref::<NotificationPermissionRequest>()
                {
                    println!(
                        "Notification permission requested for: {:?}",
                        notification_request
                    );
                    notification_request.allow();
                    println!("Notification permission granted.");
                    true
                } else if let Some(device_info_request) =
                    request.downcast_ref::<DeviceInfoPermissionRequest>()
                {
                    println!(
                        "Device info permission requested for: {:?}",
                        device_info_request
                    );
                    device_info_request.allow();
                    println!("Device info permission granted.");
                    true
                } else if let Some(user_media_permission) =
                    request.downcast_ref::<UserMediaPermissionRequest>()
                {
                    println!(
                        "User media permission requested for: {:?}",
                        user_media_permission
                    );
                    user_media_permission.allow();
                    println!("User media permission granted.");
                    true
                } else {
                    println!("Unknown permission request: {:?}", request);
                    request.deny();
                    println!("Permission denied for unknown request.");
                    false
                }
            });

            let app_name = browser.window_title.clone().unwrap_or_else(|| fl!("app"));

            builder.connect_show_notification(move |_, webkit_notification| {
                let title = webkit_notification
                    .title()
                    .unwrap_or(GString::from(fl!("app")));
                let body = webkit_notification.body().unwrap_or_default();

                notify_rust::Notification::new()
                    .appname(&app_name)
                    .summary(&title)
                    .body(&body)
                    .timeout(notify_rust::Timeout::Milliseconds(6000))
                    .show()
                    .is_ok()
            });

            builder.connect_web_process_terminated(move |_, reason| {
                eprintln!("Web process terminated: {:?}", reason);
            });

            // Add to window
            window.set_child(Some(&builder));
            window.show();
        } else {
            eprintln!("Failed to parse browser configuration from arguments.");
        }
    });

    let argv: &[&str; 0] = &[];
    app.run_with_args(argv);
}
