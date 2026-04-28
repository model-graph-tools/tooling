//! Lists all available Neo4J model DB images with local/in-use status.

use crate::container::{local_image_names, running_neo4j_containers, verify_container_command};
use crate::feature_pack::all_feature_packs;
use crate::neo4j::Neo4JImage;
use crate::source::Source;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use console::style;
use wildfly_container_versions::VERSIONS;

/// A single row in the images table, tracking local availability and usage.
struct ImageEntry {
    display_name: String,
    type_name: &'static str,
    image_tag: String,
    local: bool,
    in_use: bool,
}

/// Lists all available Neo4J model DB images, showing local availability and running status.
pub async fn images(wildfly_only: bool, feature_packs_only: bool) -> anyhow::Result<()> {
    verify_container_command()?;

    let show_both = (!wildfly_only && !feature_packs_only) || (wildfly_only && feature_packs_only);
    let show_wildfly = show_both || wildfly_only;
    let show_feature_packs = show_both || feature_packs_only;

    let mut entries: Vec<ImageEntry> = Vec::new();

    if show_wildfly {
        for wc in VERSIONS.values() {
            let source = Source::WildFly(wc.clone());
            let image = Neo4JImage::new(&source);
            entries.push(ImageEntry {
                display_name: wc.display_version(),
                type_name: "WildFly",
                image_tag: image.image_tag(),
                local: false,
                in_use: false,
            });
        }
    }

    if show_feature_packs {
        for fp in all_feature_packs() {
            let source = Source::FeaturePack(fp.clone());
            let image = Neo4JImage::new(&source);
            entries.push(ImageEntry {
                display_name: fp.display_name(),
                type_name: "Feature Pack",
                image_tag: image.image_tag(),
                local: false,
                in_use: false,
            });
        }
    }

    let local = local_image_names().await?;
    let running = running_neo4j_containers().await?;
    let in_use_tags: std::collections::HashSet<String> = running
        .iter()
        .map(|r| r.container.image.image_tag())
        .collect();

    for entry in &mut entries {
        entry.local = local.contains(&entry.image_tag);
        entry.in_use = in_use_tags.contains(&entry.image_tag);
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Version", "Type", "Image"]);

    for entry in &entries {
        let image_cell = if entry.in_use {
            Cell::new(&entry.image_tag).fg(Color::Green)
        } else if entry.local {
            Cell::new(&entry.image_tag)
        } else {
            Cell::new(&entry.image_tag).fg(Color::AnsiValue(248))
        };
        table.add_row(vec![
            Cell::new(&entry.display_name).fg(Color::DarkMagenta),
            Cell::new(entry.type_name).fg(Color::DarkCyan),
            image_cell,
        ]);
    }

    println!("\n{table}");
    println!(
        "Image legend: {}, local, {}",
        style("in use").green(),
        style("remote").color256(248)
    );
    Ok(())
}
