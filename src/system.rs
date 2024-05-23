use crate::error::{Error, Result};
use crate::item::{Item, ItemManager};
use log::debug;
use serde::Deserialize;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

#[derive(Debug)]
pub struct System<'a> {
    pub label: String,
    pub planets: Vec<Planet<'a>>,
}

impl<'a> System<'a> {
    pub fn new<P>(system_path: P, item_manager: &'a ItemManager) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let raw: RawSystem = serde_yaml::from_str(&fs::read_to_string(system_path)?)?;
        let system = Self {
            label: raw.label,
            planets: raw
                .planets
                .into_iter()
                .map(|raw| Planet::from_raw(raw, item_manager))
                .collect::<Result<_>>()?,
        };

        debug!(
            "System {} initialized with {} planet(s)",
            system.label,
            system.planets.len()
        );

        Ok(system)
    }
}

impl Display for System<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({} planets)", self.label, self.planets.len())
    }
}

#[derive(Debug)]
pub struct Planet<'a> {
    pub label: String,
    pub resources: Vec<Resource<'a>>,
}

impl Display for Planet<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.label)?;

        for resource in &self.resources {
            writeln!(
                f,
                "  {}: {:.0}%",
                resource.item.label,
                resource.density * 100.0
            )?;
        }

        Ok(())
    }
}

impl<'a> Planet<'a> {
    fn from_raw(raw_planet: RawPlanet, item_manager: &'a ItemManager) -> Result<Self> {
        let mut resources: Vec<Resource<'a>> = Vec::new();

        for (item_id, density) in raw_planet.resources {
            resources.push(Resource {
                density,
                item: match item_manager.get(&item_id) {
                    Some(item) => item,
                    None => return Err(Error::create_missing_item(item_id)),
                },
            })
        }

        Ok(Self {
            label: raw_planet.label,
            resources,
        })
    }

    pub fn collect_resources(&self) -> HashSet<&Item<'a>> {
        self.resources.iter().map(|res| &res.item).collect()
    }
}

impl Hash for Planet<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.label.hash(state)
    }
}

impl PartialEq for Planet<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

impl Eq for Planet<'_> {}

impl Ord for Planet<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.label.cmp(&other.label)
    }
}

impl PartialOrd for Planet<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub struct Resource<'a> {
    pub item: Item<'a>,
    pub density: f32,
}

#[derive(Debug, Deserialize)]
struct RawSystem {
    label: String,
    planets: Vec<RawPlanet>,
}

#[derive(Debug, Deserialize)]
struct RawPlanet {
    label: String,
    resources: HashMap<String, f32>,
}

pub trait IterPlanets {
    fn iter_planets(&self) -> impl Iterator<Item = &Planet>;
}

impl IterPlanets for System<'_> {
    fn iter_planets(&self) -> impl Iterator<Item = &Planet> {
        self.planets.iter()
    }
}

impl IterPlanets for Vec<Planet<'_>> {
    fn iter_planets(&self) -> impl Iterator<Item = &Planet> {
        self.iter()
    }
}
