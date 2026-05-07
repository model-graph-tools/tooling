//! Lists all supported WildFly versions.

use crate::registry::images_registry;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Cell, Color, ContentArrangement, Table};

/// Prints a table of all supported WildFly versions with their metadata.
pub fn versions(json: bool) -> anyhow::Result<()> {
    if json {
        let images: Vec<_> = images_registry()?.all();
        println!("{}", serde_json::to_string(&images)?);
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            "Version",
            "WildFly Version",
            "WildFly Core",
            "Repository",
        ]);

    for img in images_registry()?.all() {
        table.add_row(vec![
            Cell::new(img.short_name()).fg(Color::DarkMagenta),
            Cell::new(&img.release_version),
            Cell::new(&img.core_release_version),
            Cell::new(&img.repository).fg(Color::AnsiValue(248)),
        ]);
    }

    println!("\n{table}");
    Ok(())
}
