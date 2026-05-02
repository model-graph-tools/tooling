//! Resolves user input to canonical identifiers without any container operations.

use crate::json::ResolveResult;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use wildfly_meta::MetaItem;

pub fn resolve(items: &[MetaItem], json: bool) {
    let results: Vec<ResolveResult> = items
        .iter()
        .map(|item| ResolveResult {
            identifier: item.expression(),
            source_type: item.kind().to_string(),
            name: item.full_name(),
        })
        .collect();

    if json {
        println!("{}", serde_json::to_string(&results).unwrap());
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Identifier", "Type", "Name"]);

    for result in &results {
        table.add_row(vec![
            Cell::new(&result.identifier).fg(Color::DarkMagenta),
            Cell::new(&result.source_type),
            Cell::new(&result.name),
        ]);
    }

    println!("\n{table}");
}
