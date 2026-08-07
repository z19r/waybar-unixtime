use crate::formats::{Group, SPECS};

/// GtkBuilder XML for waybar's `menu-file`, one item per format.
/// Item ids match format keys and are wired via `menu-actions`.
pub fn xml() -> String {
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <interface>\n\
         \x20 <object class=\"GtkMenu\" id=\"menu\">\n",
    );
    let mut group = None;
    for spec in SPECS {
        if group != Some(spec.group) {
            if group.is_some() {
                out.push_str(
                    "    <child>\n\
                     \x20     <object class=\"GtkSeparatorMenuItem\"/>\n\
                     \x20   </child>\n",
                );
            }
            group = Some(spec.group);
        }
        let title = match spec.group {
            Group::Timestamp => "copy",
            Group::Date => "copy",
        };
        out.push_str(&format!(
            "    <child>\n\
             \x20     <object class=\"GtkMenuItem\" id=\"{key}\">\n\
             \x20       <property name=\"label\">{title} {label}\
             </property>\n\
             \x20     </object>\n\
             \x20   </child>\n",
            key = spec.key,
            label = spec.label,
            title = title,
        ));
    }
    out.push_str("  </object>\n</interface>\n");
    out
}

/// Recommended waybar module config, ready to paste. `binary` should
/// be an absolute path so waybar finds it regardless of environment.
pub fn snippet(binary: &str) -> String {
    let mut actions = String::new();
    for spec in SPECS {
        actions.push_str(&format!(
            "    \"{key}\": \"{binary} copy {key} | wl-copy\",\n",
            key = spec.key,
            binary = binary,
        ));
    }
    format!(
        "\"custom/unixtime\": {{\n\
         \x20 \"exec\": \"{binary} once\",\n\
         \x20 \"interval\": 1,\n\
         \x20 \"signal\": 8,\n\
         \x20 \"return-type\": \"json\",\n\
         \x20 \"menu\": \"on-click\",\n\
         \x20 \"menu-file\": \"$HOME/.config/waybar/unixtime-menu.xml\",\n\
         \x20 \"menu-actions\": {{\n\
         {actions}\
         \x20 }},\n\
         \x20 \"on-click-right\":\n\
         \x20   \"{binary} toggle && pkill -RTMIN+8 waybar\",\n\
         \x20 \"on-click-middle\": \"{binary} copy | wl-copy\",\n\
         \x20 \"tooltip\": true\n\
         }}\n",
        binary = binary,
        actions = actions,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_contains_an_item_per_format() {
        let out = xml();
        for spec in SPECS {
            assert!(
                out.contains(&format!("id=\"{}\"", spec.key)),
                "missing menu item {}",
                spec.key,
            );
        }
        assert!(out.contains("GtkMenu"));
        assert!(out.contains("GtkSeparatorMenuItem"));
    }

    #[test]
    fn snippet_wires_every_menu_action() {
        let out = snippet("/usr/bin/waybar-unixtime");
        for spec in SPECS {
            assert!(out.contains(&format!(
                "\"{key}\": \"/usr/bin/waybar-unixtime copy {key} \
                 | wl-copy\"",
                key = spec.key,
            )));
        }
        assert!(out.contains("\"menu\": \"on-click\""));
        assert!(out.contains("\"interval\": 1"));
    }
}
