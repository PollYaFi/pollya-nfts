use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Option<Addr>,
    pub admin_pub_key: Option<String>,
    pub transition_admin_pub_key: Option<String>
}

#[cw_serde]
#[derive(Default)]
pub struct PollyaAttributes {
    pub total_points: u128,
    pub poll_creation_points: u128,
    pub poll_vote_points: u128
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const POLLYA_LOYALTY_POINTS: Map<&str, PollyaAttributes> = Map::new("pollya_loyalty");
