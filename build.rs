use std::{env, fs::create_dir_all};

use dircpy::copy_dir;
use xdg::BaseDirectories;

fn main() {
    if let Ok(base_dir) = BaseDirectories::new() {
        let local_share = base_dir.get_data_home();
        let wam_rust_path = local_share.join("wam-rust");

        create_dir_all(&wam_rust_path).expect("cannot create wam-rust directory in $XDG_DATA_HOME");

        match env::current_dir() {
            Ok(cwd) => {
                let assets_dir = cwd.join("assets");
                let firefox_dir = cwd.join("firefox");

                copy_dir(assets_dir, &wam_rust_path).expect("cannot copy assets directory");
                copy_dir(firefox_dir, &wam_rust_path).expect("cannot copy firefox data");
            }
            Err(_) => todo!(),
        }
    }
}
