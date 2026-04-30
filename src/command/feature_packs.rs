//! Lists all supported feature packs.

use crate::registry::packs_registry;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Cell, Color, ContentArrangement, Table};

/// Prints a table of all supported feature packs with their Maven coordinates.
pub fn feature_packs_cmd() {
    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Shortcut", "Version", "Group ID", "Artifact ID"]);

    for fp in packs_registry().all() {
        table.add_row(vec![
            Cell::new(&fp.shortcut).fg(Color::DarkMagenta),
            Cell::new(fp.version.to_string()).fg(Color::DarkCyan),
            Cell::new(&fp.group_id).fg(Color::AnsiValue(248)),
            Cell::new(&fp.artifact_id).fg(Color::AnsiValue(248)),
        ]);
    }

    println!("\n{table}");
}
