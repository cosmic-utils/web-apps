use std::fs::create_dir_all;

use dircpy::copy_dir;

const APP_ID: &str = "dev.heppen.webapps";

fn main() {
    // this will copy from data to XDG_DATA_DIR/APP_ID dir empty firefox/zen profile
    // as a template for next webapps
    if let Some(mut data) = dirs::data_dir() {
        data.push(APP_ID);

        if !data.exists() {
            let _ = create_dir_all(&data);
        }

        let firefox = data.join("firefox");

        if !firefox.exists() {
            let _ = create_dir_all(&firefox);
            let _ = copy_dir("data/firefox", &firefox);
        }

        let zen = data.join("zen");

        if !zen.exists() {
            let _ = create_dir_all(&zen);
            let _ = copy_dir("data/zen", &zen);
        }
    }
}
