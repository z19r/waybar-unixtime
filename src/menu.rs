use crate::config::Custom;
use crate::formats::{Group, SPECS};

fn xml_escape(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// GtkBuilder XML for waybar's `menu-file`, one item per format.
/// Item ids match format keys and are wired via `menu-actions`.
pub fn xml(customs: &[Custom]) -> String {
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
    for (index, custom) in customs.iter().enumerate() {
        if index == 0 {
            out.push_str(
                "    <child>\n\
                 \x20     <object class=\"GtkSeparatorMenuItem\"/>\n\
                 \x20   </child>\n",
            );
        }
        out.push_str(&format!(
            "    <child>\n\
             \x20     <object class=\"GtkMenuItem\" \
             id=\"custom-{index}\">\n\
             \x20       <property name=\"label\">copy {label}\
             </property>\n\
             \x20     </object>\n\
             \x20   </child>\n",
            index = index,
            label = xml_escape(&custom.name),
        ));
    }
    out.push_str("  </object>\n</interface>\n");
    out
}

/// Recommended waybar module config, ready to paste. `binary` should
/// be an absolute path so waybar finds it regardless of environment.
pub fn snippet(binary: &str, customs: &[Custom]) -> String {
    let mut actions = String::new();
    for spec in SPECS {
        actions.push_str(&format!(
            "    \"{key}\": \"{binary} copy {key} | wl-copy\",\n",
            key = spec.key,
            binary = binary,
        ));
    }
    for (index, custom) in customs.iter().enumerate() {
        actions.push_str(&format!(
            "    \"custom-{index}\": \
             \"{binary} copy 'custom:{pattern}' | wl-copy\",\n",
            index = index,
            pattern = custom.format,
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
         \x20 \"on-click-right\": \"{binary} picker\",\n\
         \x20 \"on-click-middle\":\n\
         \x20   \"{binary} toggle && pkill -RTMIN+8 waybar\",\n\
         \x20 \"tooltip\": true\n\
         }}\n",
        binary = binary,
        actions = actions,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_customs() -> Vec<Custom> {
        vec![Custom {
            name: String::from("Logs & Tags"),
            format: String::from("%Y%m%d-%H%M"),
        }]
    }

    #[test]
    fn xml_contains_an_item_per_format_plus_customs() {
        let out = xml(&sample_customs());
        for spec in SPECS {
            assert!(
                out.contains(&format!("id=\"{}\"", spec.key)),
                "missing menu item {}",
                spec.key,
            );
        }
        assert!(out.contains("GtkMenu"));
        assert!(out.contains("GtkSeparatorMenuItem"));
        assert!(out.contains("id=\"custom-0\""));
        assert!(out.contains("Logs &amp; Tags"));
    }

    #[test]
    fn snippet_wires_every_menu_action() {
        let out = snippet("/usr/bin/waybar-unixtime", &sample_customs());
        for spec in SPECS {
            assert!(out.contains(&format!(
                "\"{key}\": \"/usr/bin/waybar-unixtime copy {key} \
                 | wl-copy\"",
                key = spec.key,
            )));
        }
        assert!(out.contains("\"menu\": \"on-click\""));
        assert!(out.contains("\"interval\": 1"));
        assert!(out.contains(
            "\"custom-0\": \"/usr/bin/waybar-unixtime copy \
             'custom:%Y%m%d-%H%M' | wl-copy\"",
        ));
        assert!(out.contains("picker"));
    }
}
