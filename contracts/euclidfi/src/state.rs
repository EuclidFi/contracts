use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");

pub const PORTFOLIOS: Map<&str, UserPortfolio> = Map::new("portfolios");
pub const REWARD_CONFIG: Item<RewardConfig> = Item::new("reward_config");
pub const PRICE_FEEDS: Map<&str, Uint128> = Map::new("price_feeds");


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InvestmentPosition {
    user: String,
    basket_name: String,
    initial_investment: Uint128,
    current_value: Uint128,
    token_amounts: HashMap<String, Uint128>,
    entry_price: HashMap<String, Uint128>,
    last_updated: u64,
    pnl: i64,
    performance: i64, // Percentage
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserPortfolio {
    total_invested: Uint128,
    total_current_value: Uint128,
    total_pnl: i64,
    positions: Vec<InvestmentPosition>,
    investment_history: Vec<InvestmentHistory>,
    rewards_earned: Uint128,
    last_claim: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InvestmentHistory {
    timestamp: u64,
    action: InvestmentAction,
    amount: Uint128,
    basket_name: String,
    token_prices: HashMap<String, Uint128>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum InvestmentAction {
    Deposit,
    Withdraw,
    Rebalance,
    Reinvest,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardConfig {
    reward_token: String,
    reward_rate: Uint128,
    min_lock_period: u64,
    compound_frequency: u64,
}