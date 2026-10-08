use crate::approvals::{has_approved, set_approval};
use crate::authorization::{check_active_member, check_active_treasury};
use crate::errors::ContractError;
use crate::events::{
    emit_approval_revoked, emit_payment_approved, emit_payment_cancelled, emit_payment_executed,
    emit_payment_requested,
};
use crate::policies::{calculate_period_id, get_policy};
use crate::storage::DataKey;
use crate::types::{PaymentRequest, PaymentStatus, TreasuryConfig};
use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, BytesN, Env};

pub fn get_request(env: &Env, request_id: u64) -> Result<PaymentRequest, ContractError> {
    env.storage()
        .persistent()
        .get(&DataKey::Request(request_id))
        .ok_or(ContractError::RequestNotFound)
}

pub fn is_executed(env: &Env, request_id: u64) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::ExecutedRequest(request_id))
        .unwrap_or(false)
}

pub fn create_request(
    env: &Env,
    spender: &Address,
    recipient: &Address,
    amount: i128,
    metadata_hash: BytesN<32>,
    expires_in_seconds: u64,
) -> Result<u64, ContractError> {
    spender.require_auth();
    let mut config = check_active_treasury(env)?;
    let member = check_active_member(env, spender)?;
    if member.role != crate::types::MemberRole::Spender
        && member.role != crate::types::MemberRole::Admin
    {
        return Err(ContractError::Unauthorized);
    }

    if amount <= 0 {
        return Err(ContractError::InvalidAmount);
    }

    let policy = get_policy(env, spender)?;
    if config.asset != policy.asset {
        return Err(ContractError::InvalidAsset);
    }

    let now = env.ledger().timestamp();
    let expires_at = if expires_in_seconds > 0 {
        now.checked_add(expires_in_seconds)
            .ok_or(ContractError::Overflow)?
    } else {
        0
    };

    let request_id = config.next_request_id;
    config.next_request_id = config
        .next_request_id
        .checked_add(1)
        .ok_or(ContractError::Overflow)?;
    env.storage().instance().set(&DataKey::Config, &config);

    let status = PaymentStatus::PendingApproval;

    let request = PaymentRequest {
        request_id,
        spender: spender.clone(),
        recipient: recipient.clone(),
        asset: policy.asset.clone(),
        amount,
        metadata_hash,
        policy_version: policy.version,
        created_at: now,
        expires_at,
        status,
        approval_count: 0,
    };

    env.storage()
        .persistent()
        .set(&DataKey::Request(request_id), &request);

    emit_payment_requested(env, request_id, spender.clone(), amount);

    // If amount is within individual approval threshold, check if auto-executable
    if amount <= policy.approval_threshold {
        // Auto-execution path
        execute_payment_internal(env, &mut config, request_id)?;
    }

    Ok(request_id)
}

pub fn approve_request(
    env: &Env,
    approver: &Address,
    request_id: u64,
) -> Result<(), ContractError> {
    approver.require_auth();
    let _config = check_active_treasury(env)?;
    let approver_member = check_active_member(env, approver)?;
    if approver_member.role != crate::types::MemberRole::Approver
        && approver_member.role != crate::types::MemberRole::Admin
    {
        return Err(ContractError::Unauthorized);
    }

    let mut request = get_request(env, request_id)?;

    if request.status == PaymentStatus::Executed {
        return Err(ContractError::RequestAlreadyExecuted);
    }
    if request.status == PaymentStatus::Cancelled {
        return Err(ContractError::RequestCancelled);
    }

    let now = env.ledger().timestamp();
    if request.expires_at > 0 && now > request.expires_at {
        return Err(ContractError::RequestExpired);
    }

    // Require distinct approver (requester cannot approve own payment unless policy permits)
    if *approver == request.spender {
        return Err(ContractError::SelfApprovalNotAllowed);
    }

    if has_approved(env, request_id, approver) {
        return Err(ContractError::AlreadyApproved);
    }

    set_approval(env, request_id, approver, true);
    request.approval_count += 1;

    let policy = get_policy(env, &request.spender)?;
    if request.policy_version != policy.version {
        return Err(ContractError::PolicyVersionMismatch);
    }

    if request.approval_count >= policy.required_approvals {
        request.status = PaymentStatus::Approved;
    }

    env.storage()
        .persistent()
        .set(&DataKey::Request(request_id), &request);

    emit_payment_approved(env, request_id, approver.clone(), request.approval_count);
    Ok(())
}

pub fn revoke_approval_request(
    env: &Env,
    approver: &Address,
    request_id: u64,
) -> Result<(), ContractError> {
    approver.require_auth();
    let mut request = get_request(env, request_id)?;

    if request.status == PaymentStatus::Executed {
        return Err(ContractError::RequestAlreadyExecuted);
    }

    if !has_approved(env, request_id, approver) {
        return Err(ContractError::Unauthorized);
    }

    set_approval(env, request_id, approver, false);
    if request.approval_count > 0 {
        request.approval_count -= 1;
    }

    let policy = get_policy(env, &request.spender)?;
    if request.approval_count < policy.required_approvals {
        request.status = PaymentStatus::PendingApproval;
    }

    env.storage()
        .persistent()
        .set(&DataKey::Request(request_id), &request);

    emit_approval_revoked(env, request_id, approver.clone());
    Ok(())
}

pub fn cancel_request(
    env: &Env,
    caller: &Address,
    request_id: u64,
) -> Result<(), ContractError> {
    caller.require_auth();
    let mut request = get_request(env, request_id)?;

    if request.status == PaymentStatus::Executed {
        return Err(ContractError::RequestAlreadyExecuted);
    }

    let caller_member = check_active_member(env, caller)?;
    if caller_member.address != request.spender && caller_member.role != crate::types::MemberRole::Admin {
        return Err(ContractError::Unauthorized);
    }

    request.status = PaymentStatus::Cancelled;
    env.storage()
        .persistent()
        .set(&DataKey::Request(request_id), &request);

    emit_payment_cancelled(env, request_id, request.spender.clone());
    Ok(())
}

pub fn execute_payment(
    env: &Env,
    executor: &Address,
    request_id: u64,
) -> Result<(), ContractError> {
    executor.require_auth();
    let mut config = check_active_treasury(env)?;
    execute_payment_internal(env, &mut config, request_id)
}

fn execute_payment_internal(
    env: &Env,
    _config: &mut TreasuryConfig,
    request_id: u64,
) -> Result<(), ContractError> {
    if is_executed(env, request_id) {
        return Err(ContractError::RequestAlreadyExecuted);
    }

    let mut request = get_request(env, request_id)?;

    if request.status == PaymentStatus::Executed {
        return Err(ContractError::RequestAlreadyExecuted);
    }
    if request.status == PaymentStatus::Cancelled {
        return Err(ContractError::RequestCancelled);
    }

    let now = env.ledger().timestamp();
    if request.expires_at > 0 && now > request.expires_at {
        return Err(ContractError::RequestExpired);
    }

    // Verify spender active state
    let _spender_member = check_active_member(env, &request.spender)?;

    // Retrieve spender's active policy & verify versioning
    let policy = get_policy(env, &request.spender)?;
    if request.policy_version != policy.version {
        return Err(ContractError::PolicyVersionMismatch);
    }

    // Check approval threshold requirement
    if request.amount > policy.approval_threshold {
        if request.approval_count < policy.required_approvals {
            return Err(ContractError::ApprovalThresholdNotMet);
        }
    }

    // Check and update recurring allowance
    let period_id = calculate_period_id(now, &policy.period);
    let current_spent = crate::allowances::get_spent_in_period(env, &request.spender, period_id);
    let new_spent = current_spent
        .checked_add(request.amount)
        .ok_or(ContractError::Overflow)?;

    if new_spent > policy.spending_limit {
        return Err(ContractError::AllowanceExceeded);
    }

    // Verify treasury contract balance
    let token_client = TokenClient::new(env, &request.asset);
    let contract_address = env.current_contract_address();
    let treasury_balance = token_client.balance(&contract_address);
    if treasury_balance < request.amount {
        return Err(ContractError::InsufficientTreasuryBalance);
    }

    // Update spending allowance record
    env.storage()
        .persistent()
        .set(&DataKey::Allowance(request.spender.clone(), period_id), &new_spent);

    // Mark as executed in storage (Idempotency map)
    env.storage()
        .persistent()
        .set(&DataKey::ExecutedRequest(request_id), &true);

    request.status = PaymentStatus::Executed;
    env.storage()
        .persistent()
        .set(&DataKey::Request(request_id), &request);

    // Execute on-chain SEP-41 token transfer
    token_client.transfer(&contract_address, &request.recipient, &request.amount);

    emit_payment_executed(
        env,
        request_id,
        request.spender.clone(),
        request.recipient.clone(),
        request.amount,
    );

    Ok(())
}
