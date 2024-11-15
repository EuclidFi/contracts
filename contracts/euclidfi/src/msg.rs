use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128, CosmosMsg, WasmMsg, QueryRequest, StdError, Order,
};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::contract::TokenWeight;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
    pub reward_token: String,
    pub reward_rate: Uint128,
    pub min_lock_period: u64,
    pub compound_frequency: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateBasket {
        name: String,
        tokens: Vec<TokenWeight>,
        min_investment: Uint128,
    },
    UpdateBasket {
        name: String,
        tokens: Vec<TokenWeight>,
        min_investment: Option<Uint128>,
        active: bool,
    },
    Invest {
        basket_name: String,
        amount: Uint128,
        auto_compound: bool,
    },
    Withdraw {
        basket_name: String,
        percentage: u8,
    },
    ClaimRewards {},
    SetAutoCompound {
        basket_name: String,
        enabled: bool,
    },
    Rebalance {
        basket_name: String,
    },
    UpdateConfig {
        reward_rate: Option<Uint128>,
        min_lock_period: Option<u64>,
        compound_frequency: Option<u64>,
    },
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum QueryMsg {
    GetConfig {},
    GetBasket {
        name: String,
    },
    GetPortfolio {
        address: String,
    },
    GetInvestmentHistory {
        address: String,
        from_timestamp: Option<u64>,
        to_timestamp: Option<u64>,
    },
    GetPerformanceMetrics {
        address: String,
        basket_name: Option<String>,
    },
    GetRewards {
        address: String,
    },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetCountResponse {
    pub count: i32,
}


