#![no_std]

mod allowances;
mod approvals;
mod authorization;
mod errors;
mod events;
mod payments;
mod policies;
mod storage;
mod types;

#[cfg(test)]
mod test;

use crate::authorization::{
    check_active_member, check_active_treasury, check_admin, check_initialized, get_member,
};
use crate::errors::ContractError;
use crate::events::{
    emit_member_added, emit_member_removed, emit_member_suspended, emit_policy_disabled,
    emit_policy_updated, emit_treasury_created, emit_treasury_deposited,
    emit_treasury_status_changed,
};
use crate::policies::calculate_period_id;
use crate::storage::DataKey;
pub use crate::types::{
    MemberInfo, MemberRole, MemberStatus, PaymentRequest, PaymentStatus, SpendingPeriod,
    SpendingPolicy, TreasuryConfig, TreasuryStatus,
};
use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env, Symbol};

#[contract]
pub struct TreasuryContract;

#[contractimpl]
impl TreasuryContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        asset: Address,
        org_id: Symbol,
    ) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Config) {
            return Err(ContractError::AlreadyInitialized);
        }

        admin.require_auth();

        let config = TreasuryConfig {
            org_id,
            admin: admin.clone(),
            asset: asset.clone(),
            status: TreasuryStatus::Active,
            created_at: env.ledger().timestamp(),
            next_request_id: 1,
            next_policy_id: 1,
        };

        env.storage().instance().set(&DataKey::Config, &config);

        // Add initial admin as member
        let admin_info = MemberInfo {
            address: admin.clone(),
            role: MemberRole::Admin,
            status: MemberStatus::Active,
            added_at: env.ledger().timestamp(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::Member(admin.clone()), &admin_info);

        emit_treasury_created(&env, admin, asset);
        Ok(())
    }

    pub fn deposit(env: Env, depositor: Address, amount: i128) -> Result<(), ContractError> {
        depositor.require_auth();
        let config = check_active_treasury(&env)?;

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        let token_client = TokenClient::new(&env, &config.asset);
        token_client.transfer(&depositor, &env.current_contract_address(), &amount);

        emit_treasury_deposited(&env, depositor, config.asset, amount);
        Ok(())
    }

    pub fn add_member(
        env: Env,
        admin: Address,
        new_member: Address,
        role: MemberRole,
    ) -> Result<(), ContractError> {
        check_admin(&env, &admin)?;

        if env
            .storage()
            .persistent()
            .has(&DataKey::Member(new_member.clone()))
        {
            return Err(ContractError::MemberAlreadyExists);
        }

        let member_info = MemberInfo {
            address: new_member.clone(),
            role: role.clone(),
            status: MemberStatus::Active,
            added_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Member(new_member.clone()), &member_info);

        let role_symbol = match role {
            MemberRole::Admin => symbol_short!("admin"),
            MemberRole::Approver => symbol_short!("approver"),
            MemberRole::Spender => symbol_short!("spender"),
            MemberRole::Viewer => symbol_short!("viewer"),
        };

        emit_member_added(&env, new_member, role_symbol);
        Ok(())
    }

    pub fn remove_member(env: Env, admin: Address, member: Address) -> Result<(), ContractError> {
        check_admin(&env, &admin)?;
        let _existing = get_member(&env, &member)?;

        env.storage()
            .persistent()
            .remove(&DataKey::Member(member.clone()));

        // Also disable any active spending policy
        if let Ok(mut policy) = policies::get_policy(&env, &member) {
            policy.active = false;
            env.storage()
                .persistent()
                .set(&DataKey::Policy(member.clone()), &policy);
        }

        emit_member_removed(&env, member);
        Ok(())
    }

    pub fn suspend_member(
        env: Env,
        admin: Address,
        member: Address,
    ) -> Result<(), ContractError> {
        check_admin(&env, &admin)?;
        let mut member_info = get_member(&env, &member)?;
        member_info.status = MemberStatus::Suspended;

        env.storage()
            .persistent()
            .set(&DataKey::Member(member.clone()), &member_info);

        emit_member_suspended(&env, member);
        Ok(())
    }

    pub fn set_policy(
        env: Env,
        admin: Address,
        spender: Address,
        spending_limit: i128,
        period: SpendingPeriod,
        approval_threshold: i128,
        required_approvals: u32,
        expires_at: u64,
    ) -> Result<u32, ContractError> {
        check_admin(&env, &admin)?;
        let config = check_active_treasury(&env)?;
        let _spender_member = check_active_member(&env, &spender)?;

        if spending_limit <= 0 || approval_threshold <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        let existing_policy = policies::get_policy(&env, &spender).ok();
        let new_version = match existing_policy {
            Some(p) => p.version + 1,
            None => 1,
        };

        let policy = SpendingPolicy {
            policy_id: config.next_policy_id,
            spender: spender.clone(),
            asset: config.asset,
            spending_limit,
            period,
            approval_threshold,
            required_approvals,
            version: new_version,
            active: true,
            created_at: env.ledger().timestamp(),
            expires_at,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Policy(spender.clone()), &policy);

        emit_policy_updated(&env, spender, new_version, spending_limit);
        Ok(new_version)
    }

    pub fn disable_policy(env: Env, admin: Address, spender: Address) -> Result<(), ContractError> {
        check_admin(&env, &admin)?;
        let mut policy = policies::get_policy(&env, &spender)?;
        policy.active = false;

        env.storage()
            .persistent()
            .set(&DataKey::Policy(spender.clone()), &policy);

        emit_policy_disabled(&env, spender);
        Ok(())
    }

    pub fn request_payment(
        env: Env,
        spender: Address,
        recipient: Address,
        amount: i128,
        metadata_hash: BytesN<32>,
        expires_in_seconds: u64,
    ) -> Result<u64, ContractError> {
        payments::create_request(
            &env,
            &spender,
            &recipient,
            amount,
            metadata_hash,
            expires_in_seconds,
        )
    }

    pub fn approve_payment(
        env: Env,
        approver: Address,
        request_id: u64,
    ) -> Result<(), ContractError> {
        payments::approve_request(&env, &approver, request_id)
    }

    pub fn revoke_approval(
        env: Env,
        approver: Address,
        request_id: u64,
    ) -> Result<(), ContractError> {
        payments::revoke_approval_request(&env, &approver, request_id)
    }

    pub fn cancel_payment(
        env: Env,
        caller: Address,
        request_id: u64,
    ) -> Result<(), ContractError> {
        payments::cancel_request(&env, &caller, request_id)
    }

    pub fn execute_payment(
        env: Env,
        executor: Address,
        request_id: u64,
    ) -> Result<(), ContractError> {
        payments::execute_payment(&env, &executor, request_id)
    }

    pub fn suspend_treasury(env: Env, admin: Address) -> Result<(), ContractError> {
        check_admin(&env, &admin)?;
        let mut config = check_initialized(&env)?;
        config.status = TreasuryStatus::Suspended;
        env.storage().instance().set(&DataKey::Config, &config);
        emit_treasury_status_changed(&env, symbol_short!("suspended"));
        Ok(())
    }

    pub fn reactivate_treasury(env: Env, admin: Address) -> Result<(), ContractError> {
        check_admin(&env, &admin)?;
        let mut config = check_initialized(&env)?;
        config.status = TreasuryStatus::Active;
        env.storage().instance().set(&DataKey::Config, &config);
        emit_treasury_status_changed(&env, symbol_short!("active"));
        Ok(())
    }

    // Read Queries
    pub fn get_treasury_config(env: Env) -> Result<TreasuryConfig, ContractError> {
        check_initialized(&env)
    }

    pub fn get_member_info(env: Env, member: Address) -> Result<MemberInfo, ContractError> {
        get_member(&env, &member)
    }

    pub fn get_policy(env: Env, spender: Address) -> Result<SpendingPolicy, ContractError> {
        policies::get_policy(&env, &spender)
    }

    pub fn get_spent_allowance(env: Env, spender: Address) -> Result<i128, ContractError> {
        let policy = policies::get_policy(&env, &spender)?;
        let now = env.ledger().timestamp();
        let period_id = calculate_period_id(now, &policy.period);
        Ok(allowances::get_spent_in_period(&env, &spender, period_id))
    }

    pub fn get_payment_request(env: Env, request_id: u64) -> Result<PaymentRequest, ContractError> {
        payments::get_request(&env, request_id)
    }

    pub fn has_approved_request(env: Env, request_id: u64, approver: Address) -> bool {
        approvals::has_approved(&env, request_id, &approver)
    }
}
