use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: String,
    pub legacy_token: String,
    pub target_token: String,
    pub swap_enabled: bool,
}

pub const CONFIG: Item<Config> = Item::new("config");
