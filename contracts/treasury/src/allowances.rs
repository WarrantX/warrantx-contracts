use crate::errors::ContractError;
use crate::policies::calculate_period_id;
use crate::storage::DataKey;
use crate::types::SpendingPolicy;
use soroban_sdk::{Address, Env};

pub fn get_spent_in_period(env: &Env, spender: &Address, period_id: u64) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Allowance(spender.clone(), period_id))
        .unwrap_or(0)
}

pub fn check_and_update_allowance(
    env: &Env,
    spender: &Address,
    policy: &SpendingPolicy,
    amount: i128,
) -> Result<(), ContractError> {
    if amount <= 0 {
        return Err(ContractError::InvalidAmount);
    }

    let now = env.ledger().timestamp();
    let period_id = calculate_period_id(now, &policy.period);
    let current_spent = get_spent_in_period(env, spender, period_id);

    let new_spent = current_spent
        .checked_add(amount)
        .ok_or(ContractError::Overflow)?;

    if new_spent > policy.spending_limit {
        return Err(ContractError::AllowanceExceeded);
    }

    // Persist updated spending allowance
    env.storage()
        .persistent()
        .set(&DataKey::Allowance(spender.clone(), period_id), &new_spent);

    Ok(())
}
