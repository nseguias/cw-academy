use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub commodity: String,
    pub contract_owner: Addr,
    pub commission: Decimal,
    pub highest_bidder: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const BIDS: Map<Addr, Coin> = Map::new("bids");
