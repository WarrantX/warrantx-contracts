use crate::errors::ContractError;
use crate::storage::DataKey;
use crate::types::{SpendingPeriod, SpendingPolicy};
use soroban_sdk::{Address, Env};

pub fn calculate_period_id(timestamp: u64, period: &SpendingPeriod) -> u64 {
    match period {
        SpendingPeriod::Daily => timestamp / 86400,
        SpendingPeriod::Weekly => timestamp / 604800,
        SpendingPeriod::Monthly => {
            // Howard Hinnant O(1) UTC year/month algorithm
            let days = (timestamp / 86400) as i64;
            let z = days + 719468;
            let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
            let doe = (z - era * 146097) as u64;
            let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
            let y = yoe as i64 + era * 400;
            let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
            let mp = (5 * doy + 2) / 153;
            let month = if mp < 10 { mp + 3 } else { mp - 9 };
            let year = if month <= 2 { y + 1 } else { y };
            (year as u64) * 12 + month
        }
    }
}

pub fn get_policy(env: &Env, spender: &Address) -> Result<SpendingPolicy, ContractError> {
    let policy: SpendingPolicy = env
        .storage()
        .persistent()
        .get(&DataKey::Policy(spender.clone()))
        .ok_or(ContractError::PolicyNotFound)?;

    if !policy.active {
        return Err(ContractError::PolicyDisabled);
    }

    if policy.expires_at > 0 && env.ledger().timestamp() > policy.expires_at {
        return Err(ContractError::PolicyDisabled);
    }

    Ok(policy)
}
