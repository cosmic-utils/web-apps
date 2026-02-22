use cef::*;
use std::sync::{Arc, Mutex, OnceLock, Weak};

fn get_data_uri(data: &[u8], mime_type: &str) -> String {
    let data = CefString::from(&base64_encode(Some(data)));
    let uri = CefString::from(&uriencode(Some(&data), 0)).to_string();
    format!("data:{mime_type};base64,{uri}")
}

mod linux;
use linux::*;

fn platform_show_window(_browser: Option<&mut Browser>) {
    todo!("Implement platform_show_window for non-macOS platforms");
}

static SIMPLE_HANDLER_INSTANCE: OnceLock<Weak<Mutex<SimpleHandler>>> = OnceLock::new();

pub struct SimpleHandler {
    browser_list: Vec<Browser>,
    is_closing: bool,
    weak_self: Weak<Mutex<Self>>,
}

impl SimpleHandler {
    pub fn instance() -> Option<Arc<Mutex<Self>>> {
        SIMPLE_HANDLER_INSTANCE
            .get()
            .and_then(|weak| weak.upgrade())
    }

    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new_cyclic(|weak| {
            if let Err(instance) = SIMPLE_HANDLER_INSTANCE.set(weak.clone()) {
                assert_eq!(instance.strong_count(), 0, "Replacing a viable instance");
            }

            Mutex::new(Self {
                browser_list: Vec::new(),
                is_closing: false,
                weak_self: weak.clone(),
            })
        })
    }

    fn on_title_change(&mut self, browser: Option<&mut Browser>, title: Option<&CefString>) {
        debug_assert_ne!(currently_on(ThreadId::UI), 0);

        let mut browser = browser.cloned();
        if let Some(browser_view) = browser_view_get_for_browser(browser.as_mut()) {
            if let Some(window) = browser_view.window() {
                window.set_title(title);
            }
        }

        platform_title_change(browser.as_mut(), title);
    }

    fn on_after_created(&mut self, browser: Option<&mut Browser>) {
        debug_assert_ne!(currently_on(ThreadId::UI), 0);

        let browser = browser.cloned().expect("Browser is None");

        // Sanity-check the configured runtime style.
        // assert_eq!(
        //     browser.host().expect("BrowserHost is None").runtime_style(),
        //     RuntimeStyle::ALLOY
        // );

        // Add to the list of existing browsers.
        self.browser_list.push(browser);
    }

    fn do_close(&mut self, _browser: Option<&mut Browser>) -> bool {
        debug_assert_ne!(currently_on(ThreadId::UI), 0);

        // Closing the main window requires special handling. See the DoClose()
        // documentation in the CEF header for a detailed destription of this
        // process.
        if self.browser_list.len() == 1 {
            // Set a flag to indicate that the window close should be allowed.
            self.is_closing = true;
        }

        // Allow the close. For windowed browsers this will result in the OS close
        // event being sent.
        false
    }

    fn on_before_close(&mut self, browser: Option<&mut Browser>) {
        debug_assert_ne!(currently_on(ThreadId::UI), 0);

        // Remove from the list of existing browsers.
        let mut browser = browser.cloned().expect("Browser is None");
        if let Some(index) = self
            .browser_list
            .iter()
            .position(move |elem| elem.is_same(Some(&mut browser)) != 0)
        {
            self.browser_list.remove(index);
        }

        if self.browser_list.is_empty() {
            // All browser windows have closed. Quit the application message loop.
            quit_message_loop();
        }
    }

    fn on_load_error(
        &mut self,
        _browser: Option<&mut Browser>,
        frame: Option<&mut Frame>,
        error_code: Errorcode,
        error_text: Option<&CefString>,
        failed_url: Option<&CefString>,
    ) {
        debug_assert_ne!(currently_on(ThreadId::UI), 0);

        // Don't display an error for downloaded files.
        let error_code = sys::cef_errorcode_t::from(error_code);
        if error_code == sys::cef_errorcode_t::ERR_ABORTED {
            return;
        }
        let error_code = error_code as i32;

        let frame = frame.expect("Frame is None");

        // Display a load error message using a data: URI.
        let error_text = error_text.map(CefString::to_string).unwrap_or_default();
        let failed_url = failed_url.map(CefString::to_string).unwrap_or_default();
        let data = format!(
            r#"
            <html>
                <body bgcolor="white">
                    <h2>Failed to load URL {failed_url} with error {error_text} ({error_code}).</h2>
                </body>
            </html>
            "#
        );

        let uri = get_data_uri(data.as_bytes(), "text/html");
        let uri = CefString::from(uri.as_str());
        frame.load_url(Some(&uri));
    }

    pub fn show_main_window(&mut self) {
        let thread_id = ThreadId::UI;
        if currently_on(thread_id) == 0 {
            // Execute on the UI thread.
            let this = self
                .weak_self
                .upgrade()
                .expect("Weak reference to SimpleHandler is None");
            let mut task = ShowMainWindow::new(this);
            post_task(thread_id, Some(&mut task));
            return;
        }

        let Some(mut main_browser) = self.browser_list.first().cloned() else {
            return;
        };

        if let Some(browser_view) = browser_view_get_for_browser(Some(&mut main_browser)) {
            // Show the window using the Views framework.
            if let Some(window) = browser_view.window() {
                window.show();
            }
        }
        platform_show_window(Some(&mut main_browser));
    }

    pub fn close_all_browsers(&mut self, force_close: bool) {
        let thread_id = ThreadId::UI;
        if currently_on(thread_id) == 0 {
            // Execute on the UI thread.
            let this = self
                .weak_self
                .upgrade()
                .expect("Weak reference to SimpleHandler is None");
            let mut task = CloseAllBrowsers::new(this, force_close);
            post_task(thread_id, Some(&mut task));
            return;
        }

        for browser in self.browser_list.iter() {
            let browser_host = browser.host().expect("BrowserHost is None");
            browser_host.close_browser(force_close.into());
        }
    }

    pub fn is_closing(&self) -> bool {
        self.is_closing
    }
}

wrap_client! {
    pub struct SimpleHandlerClient {
        inner: Arc<Mutex<SimpleHandler>>,
    }

    impl Client {
        fn display_handler(&self) -> Option<DisplayHandler> {
            Some(SimpleHandlerDisplayHandler::new(self.inner.clone()))
        }

        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(SimpleHandlerLifeSpanHandler::new(self.inner.clone()))
        }

        fn load_handler(&self) -> Option<LoadHandler> {
            Some(SimpleHandlerLoadHandler::new(self.inner.clone()))
        }
    }
}

wrap_display_handler! {
    struct SimpleHandlerDisplayHandler {
        inner: Arc<Mutex<SimpleHandler>>,
    }

    impl DisplayHandler {
        fn on_title_change(&self, browser: Option<&mut Browser>, title: Option<&CefString>) {
            let mut inner = self.inner.lock().expect("Failed to lock inner");
            inner.on_title_change(browser, title);
        }
    }
}

wrap_life_span_handler! {
    struct SimpleHandlerLifeSpanHandler {
        inner: Arc<Mutex<SimpleHandler>>,
    }

    impl LifeSpanHandler {
        fn on_after_created(&self, browser: Option<&mut Browser>) {
            let mut inner = self.inner.lock().expect("Failed to lock inner");
            inner.on_after_created(browser);
        }

        fn do_close(&self, browser: Option<&mut Browser>) -> i32 {
            let mut inner = self.inner.lock().expect("Failed to lock inner");
            inner.do_close(browser).into()
        }

        fn on_before_close(&self, browser: Option<&mut Browser>) {
            let mut inner = self.inner.lock().expect("Failed to lock inner");
            inner.on_before_close(browser);
        }
    }
}

wrap_load_handler! {
    struct SimpleHandlerLoadHandler {
        inner: Arc<Mutex<SimpleHandler>>,
    }

    impl LoadHandler {
        fn on_load_error(
            &self,
            browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            error_code: Errorcode,
            error_text: Option<&CefString>,
            failed_url: Option<&CefString>,
        ) {
            let mut inner = self.inner.lock().expect("Failed to lock inner");
            inner.on_load_error(browser, frame, error_code, error_text, failed_url);
        }
    }
}

wrap_task! {
    struct ShowMainWindow {
        inner: Arc<Mutex<SimpleHandler>>,
    }

    impl Task {
        fn execute(&self) {
            debug_assert_ne!(currently_on(ThreadId::UI), 0);

            let mut inner = self.inner.lock().expect("Failed to lock inner");
            inner.show_main_window();
        }
    }
}

wrap_task! {
    struct CloseAllBrowsers {
        inner: Arc<Mutex<SimpleHandler>>,
        force_close: bool,
    }

    impl Task {
        fn execute(&self) {
            debug_assert_ne!(currently_on(ThreadId::UI), 0);

            let mut inner = self.inner.lock().expect("Failed to lock inner");
            inner.close_all_browsers(self.force_close);
        }
    }
}
