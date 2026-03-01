use cef::*;

fn window_from_browser(browser: Option<&mut Browser>) -> Option<WindowHandle> {
    let window = browser?.host()?.window_handle();
    if window == 0 { None } else { Some(window) }
}

pub fn platform_title_change(browser: Option<&mut Browser>, _title: Option<&CefString>) {
    // Retrieve the X11 display shared with Chromium.
    let display = get_xdisplay();
    if display.is_null() {
        return;
    }

    // Retrieve the X11 window handle for the browser.
    let Some(_window) = window_from_browser(browser) else {
        return;
    };

    #[cfg(feature = "linux-x11")]
    unsafe {
        use std::ffi::{CString, c_char};
        use x11_dl::xlib::*;

        // Load the Xlib library dynamically.
        let Ok(xlib) = Xlib::open() else {
            return;
        };

        // Retrieve the atoms required by the below XChangeProperty call.
        let Ok(names) = ["_NET_WM_NAME", "UTF8_STRING"]
            .into_iter()
            .map(CString::new)
            .collect::<Result<Vec<_>, _>>()
        else {
            return;
        };
        let mut names: Vec<_> = names
            .iter()
            .map(|name| name.as_ptr() as *mut c_char)
            .collect();
        let mut atoms = [0; 2];
        let result = (xlib.XInternAtoms)(
            display as *mut _,
            names.as_mut_ptr(),
            2,
            0,
            atoms.as_mut_ptr(),
        );
        if result == 0 {
            return;
        }

        // Set the window title.
        let Ok(title) = CString::new(_title.map(CefString::to_string).unwrap_or_default()) else {
            return;
        };
        let title = title.as_c_str();
        (xlib.XChangeProperty)(
            display as *mut _,
            _window,
            atoms[0],
            atoms[1],
            8,
            PropModeReplace,
            title.as_ptr() as *const _,
            title.count_bytes() as i32,
        );

        // TODO(erg): This is technically wrong. So XStoreName and friends expect
        // this in Host Portable Character Encoding instead of UTF-8, which I believe
        // is Compound Text. This shouldn't matter 90% of the time since this is the
        // fallback to the UTF8 property above.
        (xlib.XStoreName)(display as *mut _, _window, title.as_ptr());
    }
}
