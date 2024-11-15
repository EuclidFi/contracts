#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cw_storage_plus::Item;
use cosmwasm_std::{
    Uint128, CosmosMsg, WasmMsg, QueryRequest, StdError, Order,
};
use cw_storage_plus::{ Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetCountResponse, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:euclidfi";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CONFIG: Item<Config> = Item::new("config");
pub const BASKETS: Map<&str, BasketConfig> = Map::new("baskets");
pub const PORTFOLIOS: Map<&str, UserPortfolio> = Map::new("portfolios");
pub const PRICE_FEEDS: Map<&str, Uint128> = Map::new("price_feeds");


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: String,
    pub reward_token: String,
    pub reward_rate: Uint128,
    pub min_lock_period: u64,
    pub compound_frequency: u64,
    pub total_value_locked: Uint128,
    pub total_users: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BasketConfig {
    name: String,
    tokens: Vec<TokenWeight>,
    min_investment: Uint128,
    total_value_locked: Uint128,
    active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ChainType {
    Cosmos,
    Ethereum,
    Polygon,
    Binance,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Token {
    address: String,
    chain: ChainType,
    symbol: String,
    decimals: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenWeight {
    token: Token,
    weight: u8,
}

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
    performance: i64,
    auto_compound: bool,
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


#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        admin: msg.admin,
        reward_token: msg.reward_token,
        reward_rate: msg.reward_rate,
        min_lock_period: msg.min_lock_period,
        compound_frequency: msg.compound_frequency,
        total_value_locked: Uint128::zero(),
        total_users: 0,
    };
    
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", info.sender))
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateBasket { name, tokens, min_investment } => {
            execute_create_basket(deps, _env, info, name, tokens, min_investment)
        },
        ExecuteMsg::UpdateBasket { name, tokens, min_investment, active } => {
            execute_update_basket(deps, info, name, tokens, min_investment, active)
        },
        ExecuteMsg::Invest { basket_name, amount, auto_compound } => {
            execute_invest(deps, env, info, basket_name, amount, auto_compound)
        },
        ExecuteMsg::Withdraw { basket_name, percentage } => {
            execute_withdraw(deps, env, info, basket_name, percentage)
        },
        ExecuteMsg::ClaimRewards {} => {
            execute_claim_rewards(deps, env, info)
        },
        ExecuteMsg::SetAutoCompound { basket_name, enabled } => {
            execute_set_auto_compound(deps, info, basket_name, enabled)
        },
        ExecuteMsg::Rebalance { basket_name } => {
            execute_rebalance(deps, env, info, basket_name)
        },
        ExecuteMsg::UpdateConfig { reward_rate, min_lock_period, compound_frequency } => {
            execute_update_config(deps, info, reward_rate, min_lock_period, compound_frequency)
        },
    }
}







// #[entry_point]
// pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
//     match msg {
//         QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
//         QueryMsg::GetBasket { name } => to_binary(&query_basket(deps, name)?),
//         QueryMsg::GetPortfolio { address } => to_binary(&query_portfolio(deps, address)?),
//         QueryMsg::GetInvestmentHistory { address, from_timestamp, to_timestamp } => 
//             to_binary(&query_investment_history(deps, address, from_timestamp, to_timestamp)?),
//         QueryMsg::GetPerformanceMetrics { address, basket_name } =>
//             to_binary(&query_performance_metrics(deps, address, basket_name)?),
//         QueryMsg::GetRewards { address } => to_binary(&query_rewards(deps, address)?),
//     }
// }



// Execute functions implementation...
pub fn execute_create_basket(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    name: String,
    tokens: Vec<TokenWeight>,
    min_investment: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender.to_string() != config.admin {
        return Err(ContractError::Unauthorized {  })
    }

    let basket = BasketConfig {
        name: name.clone(),
        tokens,
        min_investment,
        total_value_locked: Uint128::zero(),
        active: true,
    };

    BASKETS.save(deps.storage, &name, &basket)?;

    Ok(Response::new()
        .add_attribute("action", "create_basket")
        .add_attribute("name", name))
}


pub fn execute_update_basket(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    tokens: Vec<TokenWeight>,
    min_investment: Option<Uint128>,
    active: bool,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender.to_string() != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    let mut basket = BASKETS.load(deps.storage, &name)?;

    let total_weight: u8 = tokens.iter().map(|t| t.weight).sum();
    if total_weight != 100 {
        return Err(ContractError::Unauthorized {});
        
    }

    basket.tokens = tokens;
    if let Some(min_inv) = min_investment {
        basket.min_investment = min_inv;
    }
    basket.active = active;

    BASKETS.save(deps.storage, &name, &basket)?;

    Ok(Response::new()
        .add_attribute("action", "update_basket")
        .add_attribute("name", name))
}




pub mod execute {
    use super::*;

    pub fn increment(deps: DepsMut) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.count += 1;
            Ok(state)
        })?;

        Ok(Response::new().add_attribute("action", "increment"))
    }

    pub fn reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            if info.sender != state.owner {
                return Err(ContractError::Unauthorized {});
            }
            state.count = count;
            Ok(state)
        })?;
        Ok(Response::new().add_attribute("action", "reset"))
    }
}

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
//     match msg {
//         QueryMsg::GetCount {} => to_binary(&query::count(deps)?),
//     }
// }

pub mod query {
    use super::*;

    pub fn count(deps: Deps) -> StdResult<GetCountResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetCountResponse { count: state.count })
    }
}


pub fn execute_invest(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    basket_name: String,
    amount: Uint128,
    auto_compound: bool,
) -> StdResult<Response> {
    let basket = BASKETS.load(deps.storage, &basket_name)?;
    if !basket.active {
        return Err(StdError::generic_err("Basket is not active"));
    }
    if amount < basket.min_investment {
        return Err(StdError::generic_err("Investment below minimum"));
    }

    let mut portfolio = PORTFOLIOS
        .may_load(deps.storage, info.sender.as_str())?
        .unwrap_or(UserPortfolio {
            total_invested: Uint128::zero(),
            total_current_value: Uint128::zero(),
            total_pnl: 0,
            positions: vec![],
            investment_history: vec![],
            rewards_earned: Uint128::zero(),
            last_claim: env.block.time.seconds(),
        });

    let token_prices = get_token_prices(deps.as_ref())?;
    let token_amounts = calculate_token_amounts(&basket.tokens, amount, &token_prices)?;

    let position = InvestmentPosition {
        user: info.sender.to_string(),
        basket_name: basket_name.clone(),
        initial_investment: amount,
        current_value: amount,
        token_amounts: token_amounts.clone(),
        entry_price: token_prices.clone(),
        last_updated: env.block.time.seconds(),
        pnl: 0,
        performance: 0,
        auto_compound,
    };

    portfolio.positions.push(position);
    portfolio.total_invested += amount;
    portfolio.total_current_value += amount;

    portfolio.investment_history.push(InvestmentHistory {
        timestamp: env.block.time.seconds(),
        action: InvestmentAction::Deposit,
        amount,
        basket_name: basket_name.clone(),
        token_prices: token_prices.clone(),
    });

    PORTFOLIOS.save(deps.storage, info.sender.as_str(), &portfolio)?;

    let mut basket = BASKETS.load(deps.storage, &basket_name)?;
    basket.total_value_locked += amount;
    BASKETS.save(deps.storage, &basket_name, &basket)?;

    let mut config = CONFIG.load(deps.storage)?;
    config.total_value_locked += amount;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "invest")
        .add_attribute("basket", basket_name)
        .add_attribute("amount", amount.to_string()))
}

pub fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    basket_name: String,
    percentage: u8,
) -> StdResult<Response> {
    if percentage == 0 || percentage > 100 {
        return Err(StdError::generic_err("Invalid withdrawal percentage"));
    }

    let mut portfolio = PORTFOLIOS.load(deps.storage, info.sender.as_str())?;
    let position_idx = portfolio.positions
        .iter()
        .position(|p| p.basket_name == basket_name)
        .ok_or_else(|| StdError::generic_err("Position not found"))?;

    let position = &mut portfolio.positions[position_idx];
    let withdraw_amount = position.current_value.multiply_ratio(percentage as u128, 100u128);

    let token_prices = get_token_prices(deps.as_ref())?;
    let messages = generate_withdraw_messages(position, &token_prices, percentage)?;

    position.current_value -= withdraw_amount;
    portfolio.total_current_value -= withdraw_amount;

    if percentage == 100 {
        portfolio.positions.remove(position_idx);
    }

    portfolio.investment_history.push(InvestmentHistory {
        timestamp: env.block.time.seconds(),
        action: InvestmentAction::Withdraw,
        amount: withdraw_amount,
        basket_name,
        token_prices,
    });

    PORTFOLIOS.save(deps.storage, info.sender.as_str(), &portfolio)?;

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "withdraw")
        .add_attribute("amount", withdraw_amount.to_string()))
}


fn generate_withdraw_messages(
    position: &InvestmentPosition,
    token_prices: &HashMap<String, Uint128>,
    percentage: u8,
) -> StdResult<Vec<CosmosMsg>> {
    let mut messages: Vec<CosmosMsg> = vec![];

    for (token_symbol, amount) in &position.token_amounts {
        let withdraw_amount = amount.multiply_ratio(percentage as u128, 100u128);
        
        // Get basket configuration to access token information
        let basket = BASKETS.load(deps.storage, &position.basket_name)?;
        let token = basket.tokens.iter()
            .find(|t| t.token.symbol == *token_symbol)
            .ok_or_else(|| StdError::generic_err("Token not found"))?;

        match token.token.chain {
            ChainType::Cosmos => {
                messages.push(CosmosMsg::Bank(BankMsg::Send {
                    to_address: position.user.clone(),
                    amount: vec![Coin {
                        denom: token_symbol.clone(),
                        amount: withdraw_amount,
                    }],
                }));
            },
            ChainType::Ethereum => {
                messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: "eth_bridge".to_string(),
                    msg: to_binary(&"bridge_to_eth")?,
                    funds: vec![Coin {
                        denom: token_symbol.clone(),
                        amount: withdraw_amount,
                    }],
                }));
            },
            _ => return Err(StdError::generic_err("Unsupported chain")),
        }
    }

    Ok(messages)
}
pub fn execute_claim_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let mut portfolio = PORTFOLIOS.load(deps.storage, info.sender.as_str())?;

    let rewards = calculate_rewards(&portfolio, &config, env.block.time.seconds())?;
    if rewards == Uint128::zero() {
        return Err(StdError::generic_err("No rewards to claim"));
    }

    portfolio.rewards_earned += rewards;
    portfolio.last_claim = env.block.time.seconds();
    
    PORTFOLIOS.save(deps.storage, info.sender.as_str(), &portfolio)?;

    let transfer_msg = create_reward_transfer_msg(info.sender.to_string(), rewards, &config)?;

    Ok(Response::new()
        .add_message(transfer_msg)
        .add_attribute("action", "claim_rewards")
        .add_attribute("amount", rewards.to_string()))
}

pub fn execute_set_auto_compound(
    deps: DepsMut,
    info: MessageInfo,
    basket_name: String,
    enabled: bool,
) -> StdResult<Response> {
    let mut portfolio = PORTFOLIOS.load(deps.storage, info.sender.as_str())?;
    let position = portfolio.positions
        .iter_mut()
        .find(|p| p.basket_name == basket_name)
        .ok_or_else(|| StdError::generic_err("Position not found"))?;

    position.auto_compound = enabled;
    PORTFOLIOS.save(deps.storage, info.sender.as_str(), &portfolio)?;

    Ok(Response::new()
        .add_attribute("action", "set_auto_compound")
        .add_attribute("basket", basket_name)
        .add_attribute("enabled", enabled.to_string()))
}


pub fn execute_rebalance(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    basket_name: String,
) -> StdResult<Response> {
    let basket = BASKETS.load(deps.storage, &basket_name)?;
    let mut portfolio = PORTFOLIOS.load(deps.storage, info.sender.as_str())?;
    let position = portfolio.positions
        .iter_mut()
        .find(|p| p.basket_name == basket_name)
        .ok_or_else(|| StdError::generic_err("Position not found"))?;

    let token_prices = get_token_prices(deps.as_ref())?;
    let messages = generate_rebalance_messages(position, &basket.tokens, &token_prices)?;

    position.last_updated = env.block.time.seconds();
    portfolio.investment_history.push(InvestmentHistory {
        timestamp: env.block.time.seconds(),
        action: InvestmentAction::Rebalance,
        amount: position.current_value,
        basket_name,
        token_prices,
    });

    PORTFOLIOS.save(deps.storage, info.sender.as_str(), &portfolio)?;

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "rebalance")
        .add_attribute("basket", basket_name))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    reward_rate: Option<Uint128>,
    min_lock_period: Option<u64>,
    compound_frequency: Option<u64>,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender.to_string() != config.admin {
        return Err(StdError::generic_err("Unauthorized"));
    }

    if let Some(rate) = reward_rate {
        config.reward_rate = rate;
    }
    if let Some(period) = min_lock_period {
        config.min_lock_period = period;
    }
    if let Some(frequency) = compound_frequency {
        config.compound_frequency = frequency;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

fn calculate_token_amounts(
    tokens: &[TokenWeight],
    amount: Uint128,
    prices: &HashMap<String, Uint128>,
) -> StdResult<HashMap<String, Uint128>> {
    let mut amounts = HashMap::new();
    let total_weight: u8 = tokens.iter().map(|t| t.weight).sum();

    for token in tokens {
        let token_amount = amount.multiply_ratio(token.weight as u128, total_weight as u128);
        amounts.insert(token.token.symbol.clone(), token_amount);
    }

    Ok(amounts)
}

fn get_token_prices(deps: Deps) -> StdResult<HashMap<String, Uint128>> {
    let mut prices = HashMap::new();
    for item in PRICE_FEEDS.range(deps.storage, None, None, Order::Ascending) {
        let (token, price) = item?;
        prices.insert(token.to_string(), price);
    }
    Ok(prices)
}

fn calculate_rewards(
    portfolio: &UserPortfolio,
    config: &Config,
    current_time: u64,
) -> StdResult<Uint128> {
    let time_elapsed = current_time.saturating_sub(portfolio.last_claim);
    if time_elapsed < config.min_lock_period {
        return Ok(Uint128::zero());
    }

    let reward = portfolio.total_current_value
        .multiply_ratio(config.reward_rate * time_elapsed as u128, 365 * 24 * 60 * 60 * 100u128);
    
    Ok(reward)
}

// Query implementations
fn query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

fn query_basket(deps: Deps, name: String) -> StdResult<BasketConfig> {
    BASKETS.load(deps.storage, &name)
}

fn query_portfolio(deps: Deps, address: String) -> StdResult<UserPortfolio> {
    PORTFOLIOS.load(deps.storage, &address)
}

fn query_investment_history(
    deps: Deps,
    address: String,
    from_timestamp: Option<u64>,
    to_timestamp: Option<u64>,
) -> StdResult<Vec<InvestmentHistory>> {
    let portfolio = PORTFOLIOS.load(deps.storage, &address)?;
    Ok(portfolio.investment_history
        .into_iter()
        .filter(|h| {
            from_timestamp.map_or(true, |from| h.timestamp >= from) &&
            to_timestamp.map_or(true, |to| h.timestamp <= to)
        })
        .collect())
}

fn query_performance_metrics(
    deps: Deps,
    address: String,
    basket_name: Option<String>,
) -> StdResult<Vec<(String, i64)>> {
    let portfolio = PORTFOLIOS.load(deps.storage, &address)?;
    let metrics: Vec<(String, i64)> = portfolio.positions
        .into_iter()
        .filter(|p| basket_name.as_ref().map_or(true, |b| p.basket_name == *b))
        .map(|p| (p.basket_name, p.performance))
        .collect();
    Ok(metrics)
}

fn query_rewards(deps: Deps, address: String) -> StdResult<Uint128> {
    let portfolio = PORTFOLIOS.load(deps.storage, &address)?;
    Ok(portfolio.rewards_earned)
}


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    // #[test]
    // fn proper_initialization() {
    //     let mut deps = mock_dependencies();

    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(1000, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     // it worked, let's query the state
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: GetCountResponse = from_binary(&res).unwrap();
    //     assert_eq!(17, value.count);
    // }

    // #[test]
    // fn increment() {
    //     let mut deps = mock_dependencies();

    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Increment {};
    //     let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // should increase counter by 1
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: GetCountResponse = from_binary(&res).unwrap();
    //     assert_eq!(18, value.count);
    // }

    // #[test]
    // fn reset() {
    //     let mut deps = mock_dependencies();

    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let unauth_info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
    //     match res {
    //         Err(ContractError::Unauthorized {}) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }

    //     // only the original creator can reset the counter
    //     let auth_info = mock_info("creator", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    //     // should now be 5
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: GetCountResponse = from_binary(&res).unwrap();
    //     assert_eq!(5, value.count);
    // }
}
