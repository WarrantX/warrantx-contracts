use crate::errors::ContractError;
use crate::storage::DataKey;
use crate::types::{MemberInfo, MemberRole, MemberStatus, TreasuryConfig, TreasuryStatus};
use soroban_sdk::{Address, Env};

pub fn check_initialized(env: &Env) -> Result<TreasuryConfig, ContractError> {
    env.storage()
        .instance()
        .get(&DataKey::Config)
        .ok_or(ContractError::NotInitialized)
}

pub fn check_active_treasury(env: &Env) -> Result<TreasuryConfig, ContractError> {
    let config = check_initialized(env)?;
    match config.status {
        TreasuryStatus::Active => Ok(config),
        TreasuryStatus::Suspended => Err(ContractError::TreasurySuspended),
        TreasuryStatus::Closed => Err(ContractError::TreasuryClosed),
    }
}

pub fn get_member(env: &Env, address: &Address) -> Result<MemberInfo, ContractError> {
    env.storage()
        .persistent()
        .get(&DataKey::Member(address.clone()))
        .ok_or(ContractError::MemberNotFound)
}

pub fn check_active_member(env: &Env, address: &Address) -> Result<MemberInfo, ContractError> {
    let member = get_member(env, address)?;
    if member.status == MemberStatus::Suspended {
        return Err(ContractError::MemberSuspended);
    }
    Ok(member)
}

pub fn check_admin(env: &Env, caller: &Address) -> Result<(), ContractError> {
    caller.require_auth();
    let config = check_active_treasury(env)?;
    if config.admin == *caller {
        return Ok(());
    }
    let member = check_active_member(env, caller)?;
    if member.role == MemberRole::Admin {
        Ok(())
    } else {
        Err(ContractError::Unauthorized)
    }
}
