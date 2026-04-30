//! Lists all supported WildFly versions.

use crate::registry::images_registry;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Cell, Color, ContentArrangement, Table};

/// Prints a table of all supported WildFly versions with their metadata.
pub fn versions() {
    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            "Version",
            "Full Version",
            "WildFly Core",
            "Repository",
        ]);

    for img in images_registry().all() {
        table.add_row(vec![
            Cell::new(img.short_name()).fg(Color::DarkMagenta),
            Cell::new(format!("{}{}", img.version, suffix_display(&img.suffix))),
            Cell::new(format!(
                "{}{}",
                img.core_version,
                suffix_display(&img.suffix)
            )),
            Cell::new(&img.repository).fg(Color::AnsiValue(248)),
        ]);
    }

    println!("\n{table}");
}

/// Formats a version suffix (e.g. `Final`) with a leading dot, or empty for no suffix.
fn suffix_display(suffix: &str) -> String {
    if suffix.is_empty() {
        String::new()
    } else {
        format!(".{suffix}")
    }
}
