use cef::{Rect, *};
use clap::Parser as _;
use std::cell::RefCell;
use webapps::WebviewArgs;

use super::simple_handler::*;

wrap_window_delegate! {
    struct SimpleWindowDelegate {
        browser_view: RefCell<Option<BrowserView>>,
        initial_show_state: ShowState,
    }

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size {
                width: 800,
                height: 600,
            }
        }
    }

    impl PanelDelegate {}

    impl WindowDelegate {
        fn on_window_created(&self, window: Option<&mut Window>) {
            // Add the browser view and show the window.
            let browser_view = self.browser_view.borrow();
            let (Some(window), Some(browser_view)) = (window, browser_view.as_ref()) else {
                return;
            };
            let mut view = View::from(browser_view);
            window.add_child_view(Some(&mut view));

            if self.initial_show_state != ShowState::HIDDEN {
                window.show();
            }
        }

        fn can_resize(&self, _window: Option<&mut Window>) -> i32 {
            1
        }

        fn initial_bounds(&self, window: Option<&mut Window>) -> Rect {
            let args = WebviewArgs::parse();

            if let Some(browser_config) = crate::Browser::from_appid(&args.id) {
                let Some(size) = browser_config.window_size else {
                    return Default::default();
                };


                return Rect {width: size.0 as i32, height: size.1 as i32, ..Default::default()};
            };

            Default::default()
        }

        fn on_window_destroyed(&self, _window: Option<&mut Window>) {
            let mut browser_view = self.browser_view.borrow_mut();
            *browser_view = None;
        }

        fn can_close(&self, _window: Option<&mut Window>) -> i32 {
            // Allow the window to close if the browser says it's OK.
            let browser_view = self.browser_view.borrow();
            let browser_view = browser_view.as_ref().expect("BrowserView is None");
            if let Some(browser) = browser_view.browser() {
                let browser_host = browser.host().expect("BrowserHost is None");
                browser_host.try_close_browser()
            } else {
                1
            }
        }

        fn initial_show_state(&self, _window: Option<&mut Window>) -> ShowState {
            self.initial_show_state
        }

        fn window_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::ALLOY
        }
    }
}

wrap_browser_view_delegate! {
    struct SimpleBrowserViewDelegate {}

    impl ViewDelegate {}

    impl BrowserViewDelegate {
        fn on_popup_browser_view_created(
            &self,
            _browser_view: Option<&mut BrowserView>,
            popup_browser_view: Option<&mut BrowserView>,
            _is_devtools: i32,
        ) -> i32 {
            // Create a new top-level Window for the popup. It will show itself after
            // creation.
            let mut window_delegate = SimpleWindowDelegate::new(
                RefCell::new(popup_browser_view.cloned()),
                ShowState::NORMAL,
            );
            window_create_top_level(Some(&mut window_delegate));

            // We created the Window.
            1
        }

        fn browser_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::ALLOY
        }
    }
}

wrap_app! {
    pub struct SimpleApp;

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(SimpleBrowserProcessHandler::new(RefCell::new(None)))
        }
    }
}

wrap_browser_process_handler! {
    struct SimpleBrowserProcessHandler {
        client: RefCell<Option<Client>>,
    }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            debug_assert_ne!(currently_on(ThreadId::UI), 0);

            let command_line = command_line_get_global().expect("Failed to get command line");

            {
                // SimpleHandler implements browser-level callbacks.
                let mut client = self.client.borrow_mut();
                *client = Some(SimpleHandlerClient::new(SimpleHandler::new()));
            }

            // Specify CEF browser settings here.
            let settings = BrowserSettings::default();

            let args = WebviewArgs::parse();

            let Some(browser_config) = crate::Browser::from_appid(&args.id) else {
                return;
            };

            let Some(url) = browser_config.url else {
                return;
            };

            let url = CefString::from(url.as_str());

            // Create the BrowserView.
            let mut client = self.default_client();
            let mut delegate = SimpleBrowserViewDelegate::new();
            let browser_view = browser_view_create(
                client.as_mut(),
                Some(&url),
                Some(&settings),
                None,
                None,
                Some(&mut delegate),
            );

            // Optionally configure the initial show state.
            let initial_show_state = CefString::from(
                &command_line.switch_value(Some(&CefString::from("initial-show-state"))),
            )
            .to_string();

            let initial_show_state = match initial_show_state.as_str() {
                "minimized" => ShowState::MINIMIZED,
                "maximized" => ShowState::MAXIMIZED,
                _ => ShowState::NORMAL,
            };

            // Create the Window. It will show itself after creation.
            let mut delegate = SimpleWindowDelegate::new(
                RefCell::new(browser_view),
                initial_show_state,
            );
            window_create_top_level(Some(&mut delegate));
        }

        fn default_client(&self) -> Option<Client> {
            self.client.borrow().clone()
        }
    }
}
