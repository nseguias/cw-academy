use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub commodity: String,
    pub contract_owner: Addr,
    pub commission: u128,
    pub denom: String,
    pub status: BidStatus,
}

#[cw_serde]
pub enum BidStatus {
    Opened,
    Closed,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const BIDS: Map<Addr, Uint128> = Map::new("bids");
pub const WINNER: Item<(Addr, Uint128)> = Item::new("winner");
