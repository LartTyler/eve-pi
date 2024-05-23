use crate::item::{Item, ItemManager};
use crate::system::{IterPlanets, Planet};
use crate::Tier;
use itertools::Itertools;
use log::trace;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default)]
pub struct Builder {
    use_factory_planet: Option<bool>,
    max_planets: Option<usize>,
    production_max_tier: Option<Tier>,
    factory_max_tier: Option<Tier>,
}

impl Builder {
    pub fn use_factory_planet<V>(mut self, value: V) -> Self
    where
        V: Into<Option<bool>>,
    {
        self.use_factory_planet = value.into();
        self
    }

    pub fn max_planets<V>(mut self, value: V) -> Self
    where
        V: Into<Option<usize>>,
    {
        self.max_planets = value.into();
        self
    }

    pub fn production_max_tier<V>(mut self, value: V) -> Self
    where
        V: Into<Option<Tier>>,
    {
        self.production_max_tier = value.into();
        self
    }

    pub fn factory_max_tier<V>(mut self, value: V) -> Self
    where
        V: Into<Option<Tier>>,
    {
        self.factory_max_tier = value.into();
        self
    }

    pub fn build(self) -> Solver {
        let use_factory_planet = self.use_factory_planet.unwrap_or(true);

        let max_planets = self.max_planets.unwrap_or(6);
        let max_planets = if use_factory_planet {
            max_planets - 1
        } else {
            max_planets
        };

        let production_max_tier = self.production_max_tier.unwrap_or(if use_factory_planet {
            Tier::P1
        } else {
            Tier::P4
        });

        Solver {
            factory_max_tier: self.factory_max_tier.unwrap_or(Tier::P4),
            production_max_tier,
            use_factory_planet,
            max_planets,
        }
    }
}

#[derive(Debug)]
pub struct Solver {
    production_max_tier: Tier,
    factory_max_tier: Tier,
    use_factory_planet: bool,
    max_planets: usize,
}

impl Solver {
    pub fn builder() -> Builder {
        Builder::default()
    }

    pub fn solve<'a, P>(&self, planets: &'a P, item_manager: &'a ItemManager) -> Simulation<'a>
    where
        P: IterPlanets,
    {
        let mut simulation = Simulation::default();

        for planet in planets.iter_planets() {
            let products = self.solve_cycles(
                &planet.collect_resources(),
                item_manager,
                self.production_max_tier,
            );

            simulation
                .planet_solutions
                .push(Solution { planet, products });
        }

        if self.use_factory_planet {
            for planet_set in simulation
                .planet_solutions
                .clone()
                .into_iter()
                .combinations(self.max_planets)
            {
                let inputs = planet_set
                    .iter()
                    .flat_map(|solution| &solution.products)
                    .collect();

                let products = self.solve_cycles(&inputs, item_manager, self.factory_max_tier);

                simulation.factory_solutions.push(FactorySolution {
                    planets: planet_set,
                    products,
                })
            }
        }

        simulation
    }

    fn solve_cycles<'a>(
        &self,
        initial_inputs: &HashSet<&Item<'a>>,
        item_manager: &'a ItemManager,
        max_tier: Tier,
    ) -> HashSet<Item<'a>> {
        let mut products = HashSet::new();
        let mut next_cycle = self.solve_cycle(initial_inputs, item_manager, max_tier);

        loop {
            let mut inserted = 0;

            for output in next_cycle.outputs {
                let is_tier_allowed = output.tier > Tier::R0 && output.tier <= max_tier;

                if is_tier_allowed && products.insert(output) {
                    inserted += 1;
                }
            }

            if inserted == 0 {
                break;
            }

            next_cycle = self.solve_cycle(&products.iter().collect(), item_manager, max_tier);
        }

        products
    }

    fn solve_cycle<'a>(
        &self,
        inputs: &HashSet<&Item<'a>>,
        item_manager: &'a ItemManager,
        max_tier: Tier,
    ) -> Cycle<'a> {
        let mut cycle = Cycle::default();

        for input in inputs {
            for product in item_manager.get_products(input).unwrap_or_default() {
                if product.tier > max_tier {
                    continue;
                }

                trace!("Checking if cycle can produce {}", product.id);

                // Unwrap is safe here because an item cannot be returned from
                // `ItemManager::get_products()` if it has no production information.
                let production = product.production.as_ref().unwrap();

                if production.can_be_made_using(inputs) {
                    cycle.outputs.insert(product.clone());
                    trace!("Cycle can produce {}", product.id);
                } else {
                    trace!("Cycle cannot produce {}", product.id);
                }
            }
        }

        cycle
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
struct Cycle<'a> {
    outputs: HashSet<Item<'a>>,
}

#[derive(Debug, Default, Clone)]
pub struct Simulation<'a> {
    pub planet_solutions: Vec<Solution<'a>>,
    pub factory_solutions: Vec<FactorySolution<'a>>,
}

#[derive(Debug, Clone)]
pub struct Solution<'a> {
    pub planet: &'a Planet<'a>,
    pub products: HashSet<Item<'a>>,
}

impl Display for Solution<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", &self.planet.label)?;

        for product in self.products.iter().sorted_by_key(|product| product.tier) {
            if product.tier < Tier::P1 {
                continue;
            }

            write!(f, "  {product}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FactorySolution<'a> {
    pub planets: Vec<Solution<'a>>,
    pub products: HashSet<Item<'a>>,
}
