use crate::cli::Cli;
use crate::item::ItemManager;
use crate::solver::Solver;
use crate::system::System;
use clap::{Parser, ValueEnum};
use itertools::Itertools;
use serde::Deserialize;

mod cli;
mod error;
mod item;
mod solver;
mod system;

fn main() -> Result<(), error::Error> {
    env_logger::init();

    let cli = Cli::parse();

    let item_manager = ItemManager::new(cli.items)?;
    let system = System::new(cli.system, &item_manager)?;

    let planets = if cli.include_planet.is_empty() {
        system.planets
    } else {
        system
            .planets
            .into_iter()
            .filter(|p| cli.include_planet.contains(&p.label))
            .collect()
    };

    let simulation = Solver::builder()
        .use_factory_planet(!cli.no_factory)
        .max_planets(cli.max_planets)
        .production_max_tier(cli.production_max_tier)
        .factory_max_tier(cli.factory_max_tier)
        .build()
        .solve(&planets, &item_manager);

    if !simulation.factory_solutions.is_empty() {
        let min_tier = cli.factory_min_tier.unwrap_or(Tier::R0);

        for solution in simulation.factory_solutions {
            let products: Vec<_> = solution
                .products
                .into_iter()
                .filter(|p| p.tier >= min_tier)
                .sorted_by_key(|p| p.tier)
                .rev()
                .collect();

            if products.is_empty() {
                continue;
            }

            println!(
                "Using {}",
                solution.planets.iter().map(|s| &s.planet.label).join(", ")
            );

            for product in products {
                if product.tier < min_tier {
                    continue;
                }

                println!("  {product}");
            }

            println!();
        }
    } else {
        for solution in simulation.planet_solutions {
            println!("{}", solution.planet);

            for product in solution.products.iter().sorted_by_key(|p| p.tier) {
                println!("  {product}");
            }

            println!();
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    R0,
    P1,
    P2,
    P3,
    P4,
}
