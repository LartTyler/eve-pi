use crate::Tier;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    /// Path to the system definition file
    #[arg(value_name = "SYSTEM_FILE")]
    pub system: PathBuf,

    /// Path to the item definition file
    #[arg(long, default_value = "./examples/items.yaml")]
    pub items: PathBuf,

    #[arg(short, long)]
    pub no_factory: bool,

    #[arg(long)]
    pub max_planets: Option<usize>,

    #[arg(short, long, value_name = "TIER")]
    pub production_max_tier: Option<Tier>,

    #[arg(short, long, value_name = "TIER")]
    pub factory_max_tier: Option<Tier>,

    #[arg(long, value_name = "TIER")]
    pub factory_min_tier: Option<Tier>,

    #[arg(short, long, value_name = "PLANET")]
    pub include_planet: Vec<String>,
}
