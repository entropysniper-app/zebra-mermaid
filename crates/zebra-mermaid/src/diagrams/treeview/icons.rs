//! Icons of tree entries. Ported from Pele `src/diagrams/treeview/icons.ts`.

use super::NodeType;
use crate::config::{Config, ConfigMap, ConfigValue};

/// Mermaid's name for its built-in pack. Only `file` and `folder` are built in; every other
/// icon name is offered to the host's icon resolver.
pub const BUILTIN_PACK: &str = "mermaid-treeview";

/// Outlines on a 24 by 24 grid.
pub fn glyph(name: &str) -> Option<&'static str> {
    match name {
        "folder" => Some("M3 6.5A1.5 1.5 0 0 1 4.5 5h4.6l2.4 2.5h8A1.5 1.5 0 0 1 21 9v9a1.5 1.5 0 0 1-1.5 1.5h-15A1.5 1.5 0 0 1 3 18z"),
        "file" => Some("M6.5 3h7L19 8.5v11a1.5 1.5 0 0 1-1.5 1.5h-11A1.5 1.5 0 0 1 5 19.5v-15A1.5 1.5 0 0 1 6.5 3zM13.5 3v5.5H19"),
        _ => None,
    }
}

pub struct IconConfig<'a> {
    pub show_icons: bool,
    pub default_icon_pack: &'a str,
    pub filename_icons: Option<&'a ConfigMap>,
    pub extension_icons: Option<&'a ConfigMap>,
}

/// A string entry of a map, as `own` reads an own property that is a string.
fn own<'a>(map: Option<&'a ConfigMap>, key: &str) -> Option<&'a str> {
    map?.get(key).and_then(ConfigValue::as_str)
}

/// A file's icon from the configured maps. A file name wins over an extension, and extensions
/// match in lower case with or without the leading dot.
pub fn detect_icon<'a>(name: &str, config: &IconConfig<'a>) -> Option<&'a str> {
    if let Some(by_name) = own(config.filename_icons, name).filter(|s| !s.is_empty()) {
        return Some(by_name);
    }
    match name.rfind('.') {
        Some(dot) if dot > 0 => {
            let ext = name[dot..].to_lowercase();
            own(config.extension_icons, &ext).or_else(|| own(config.extension_icons, &ext[1..]))
        }
        _ => None,
    }
}

fn qualify_icon(icon: &str, default_icon_pack: &str) -> String {
    if icon.contains(':') {
        return icon.to_string();
    }
    if glyph(icon).is_some() || default_icon_pack.is_empty() {
        return format!("{}:{}", BUILTIN_PACK, icon);
    }
    format!("{}:{}", default_icon_pack, icon)
}

/// The qualified icon name for a node, or `None` when it shows none. An `icon()` annotation
/// always wins; otherwise icons appear only with `showIcons`.
pub fn get_node_icon(icon: Option<&str>, name: &str, node_type: NodeType, config: &IconConfig) -> Option<String> {
    if icon == Some("none") {
        return None;
    }
    if let Some(icon) = icon.filter(|i| !i.is_empty()) {
        return Some(qualify_icon(icon, config.default_icon_pack));
    }
    if !config.show_icons {
        return None;
    }
    if node_type == NodeType::File {
        let detected = detect_icon(name, config);
        if detected == Some("none") {
            return None;
        }
        if let Some(detected) = detected.filter(|d| !d.is_empty()) {
            return Some(qualify_icon(detected, config.default_icon_pack));
        }
    }
    Some(format!("{}:{}", BUILTIN_PACK, if node_type == NodeType::Directory { "folder" } else { "file" }))
}

pub fn icon_config(config: &Config) -> IconConfig<'_> {
    let c = config.section("treeView");
    let get = |key: &str| c.and_then(|c| c.get(key));
    IconConfig {
        show_icons: get("showIcons").is_some_and(ConfigValue::truthy),
        default_icon_pack: get("defaultIconPack").and_then(ConfigValue::as_str).unwrap_or(""),
        filename_icons: get("filenameIcons").and_then(ConfigValue::as_object),
        extension_icons: get("extensionIcons").and_then(ConfigValue::as_object),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualifies() {
        let c = IconConfig { show_icons: true, default_icon_pack: "", filename_icons: None, extension_icons: None };
        assert_eq!(get_node_icon(Some("none"), "a", NodeType::File, &c), None);
        assert_eq!(get_node_icon(Some("fa:x"), "a", NodeType::File, &c).as_deref(), Some("fa:x"));
        assert_eq!(get_node_icon(Some("x"), "a", NodeType::File, &c).as_deref(), Some("mermaid-treeview:x"));
        assert_eq!(get_node_icon(None, "a", NodeType::Directory, &c).as_deref(), Some("mermaid-treeview:folder"));
        let c = IconConfig { show_icons: false, default_icon_pack: "p", filename_icons: None, extension_icons: None };
        assert_eq!(get_node_icon(Some("x"), "a", NodeType::File, &c).as_deref(), Some("p:x"));
        assert_eq!(get_node_icon(Some("file"), "a", NodeType::File, &c).as_deref(), Some("mermaid-treeview:file"));
        assert_eq!(get_node_icon(None, "a", NodeType::File, &c), None);
    }
}
