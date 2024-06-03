// SPDX-License-Identifier: GPL-3.0-only
// Source: https://github.com/pop-os/cosmic-term/blob/master/src/icon_cache.rs

use cosmic::widget::icon;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IconCacheKey {
    name: &'static str,
    size: u16,
}

pub struct IconCache {
    cache: HashMap<IconCacheKey, icon::Handle>,
}

impl IconCache {
    pub fn new() -> Self {
        let mut cache = HashMap::new();

        macro_rules! bundle {
            ($name:expr, $size:expr) => {
                let data: &'static [u8] = include_bytes!(concat!("../res/icons/", $name, ".svg"));
                cache.insert(
                    IconCacheKey {
                        name: $name,
                        size: $size,
                    },
                    icon::from_svg_bytes(data).symbolic(true),
                );
            };
        }

        bundle!("folder-download-symbolic", 16);
        bundle!("folder-pictures-symbolic", 16);
        bundle!("application-menu-symbolic", 16);
        bundle!("edit-symbolic", 16);
        bundle!("edit-delete-symbolic", 16);
        bundle!("document-new-symbolic", 16);
        bundle!("go-home-symbolic", 16);

        Self { cache }
    }

    pub fn get(&mut self, name: &'static str, size: u16) -> icon::Icon {
        let handle = self
            .cache
            .entry(IconCacheKey { name, size })
            .or_insert_with(|| icon::from_name(name).size(size).handle())
            .clone();
        icon::icon(handle).size(size)
    }
}
