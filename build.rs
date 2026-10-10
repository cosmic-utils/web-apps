use std::fs::create_dir_all;

use dircpy::copy_dir;

const APP_ID: &str = "dev.heppen.webapps";

fn main() {
    // this will copy from data to XDG_STATE/APP_ID dir empty firefox profile
    // as a template for next firefox's webapps
    if let Some(mut state) = dirs::state_dir() {
        state.push(APP_ID);

        if !state.exists() {
            let _ = create_dir_all(&state);
        }

        state.push("firefox");

        if !state.exists() {
            let _ = copy_dir("data/firefox", &state);
        }
    }
}
