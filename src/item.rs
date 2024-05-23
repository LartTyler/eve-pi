use crate::error::Error;
use crate::{error, Tier};
use log::{debug, trace, warn};
use serde::Deserialize;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Item<'a> {
    pub id: &'a str,
    pub label: &'a str,
    pub tier: Tier,
    pub is_p4_input: bool,
    pub production: Option<Production<'a>>,
}

impl Hash for Item<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl PartialEq for Item<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(other.id)
    }
}

impl Eq for Item<'_> {}

impl PartialOrd for Item<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Item<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(other.id)
    }
}

impl<'a> Item<'a> {
    fn from_raw(item_manager: &'a ItemManager, raw_item: &'a RawItem) -> error::Result<Self> {
        Ok(Self {
            id: &raw_item.id,
            label: &raw_item.label,
            tier: raw_item.tier,
            is_p4_input: raw_item.is_p4_input,
            production: raw_item
                .production
                .as_ref()
                .map(|raw| Production::from_raw(item_manager, raw))
                .transpose()?,
        })
    }
}

impl Display for Item<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(production) = &self.production {
            for (index, input) in production.inputs.iter().enumerate() {
                if index > 0 {
                    f.write_str(" + ")?;
                }

                f.write_str(input.item.label)?;
            }

            f.write_str(" → ")?;
        }

        f.write_str(self.label)
    }
}

#[derive(Debug, Clone)]
pub struct Production<'a> {
    pub quantity: u16,
    pub inputs: Vec<Input<'a>>,
}

impl<'a> Production<'a> {
    fn from_raw(
        item_manager: &'a ItemManager,
        raw_production: &'a RawProduction,
    ) -> error::Result<Self> {
        let mut inputs: Vec<Input<'a>> = Vec::new();

        for (item_id, amount) in &raw_production.inputs {
            let Some(item) = item_manager.get(item_id) else {
                return Err(Error::MissingItem(String::from(item_id)));
            };

            inputs.push(Input {
                item,
                amount: *amount,
            })
        }

        Ok(Self {
            quantity: raw_production.quantity,
            inputs,
        })
    }

    pub fn can_be_made_using(&self, possible_inputs: &HashSet<&Item<'a>>) -> bool {
        self.inputs
            .iter()
            .all(|input| possible_inputs.contains(&input.item))
    }
}

#[derive(Debug, Clone)]
pub struct Input<'a> {
    pub item: Item<'a>,
    pub amount: u16,
}

type ItemMap = HashMap<String, RawItem>;
type UsedInMap = HashMap<String, HashSet<String>>;

#[derive(Debug)]
pub struct ItemManager {
    items: ItemMap,
    used_in: UsedInMap,
}

impl ItemManager {
    pub fn new<P>(items_file: P) -> error::Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut items: ItemMap = serde_yaml::from_str(&fs::read_to_string(items_file)?)?;
        let mut used_in = UsedInMap::new();

        for (id, item) in items.iter_mut() {
            item.id = id.clone();

            if let Some(production) = &item.production {
                for input in production.inputs.keys() {
                    used_in
                        .entry(input.to_string())
                        .or_default()
                        .insert(id.to_string());
                }
            }

            trace!("Finished initializing {id}");
        }

        debug!("Item manager initialized with {} item(s)", items.len());

        debug!(
            "Item manager initialized with {} mapped product(s)",
            used_in.len()
        );

        Ok(Self { items, used_in })
    }

    pub fn get<Id>(&self, item_id: Id) -> Option<Item>
    where
        Id: AsRef<str>,
    {
        let Some(raw_item) = self.items.get(item_id.as_ref()) else {
            warn!("Could not find item with ID '{}'", item_id.as_ref());
            return None;
        };

        Item::from_raw(self, raw_item).ok()
    }

    pub fn get_products<'a>(&self, item: &'a Item<'a>) -> Option<Vec<Item>> {
        let Some(products) = self.used_in.get(item.id) else {
            return None;
        };

        Some(products.iter().map(|id| self.get(id).unwrap()).collect())
    }
}

#[derive(Debug, Deserialize)]
struct RawItem {
    #[serde(default)]
    id: String,
    label: String,
    tier: Tier,
    production: Option<RawProduction>,
    #[serde(default)]
    is_p4_input: bool,
}

#[derive(Debug, Deserialize)]
struct RawProduction {
    quantity: u16,
    inputs: HashMap<String, u16>,
}
