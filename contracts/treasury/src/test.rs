#![cfg(test)]

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, BytesN, Env,
};

fn setup_test() -> (
    Env,
    Address, // Admin
    Address, // Asset/Token Contract Address
    Address, // Treasury Contract Address
    TreasuryContractClient<'static>,
    StellarAssetClient<'static>,
    TokenClient<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_id = token_contract.address();

    let treasury_id = env.register(TreasuryContract, ());
    let treasury_client = TreasuryContractClient::new(&env, &treasury_id);

    let token_admin_client = StellarAssetClient::new(&env, &token_id);
    let token_client = TokenClient::new(&env, &token_id);

    treasury_client.initialize(&admin, &token_id, &symbol_short!("ORG1"));

    (
        env,
        admin,
        token_id,
        treasury_id,
        treasury_client,
        token_admin_client,
        token_client,
    )
}

#[test]
fn test_treasury_initialization_and_double_init_prevention() {
    let (env, admin, token_id, _treasury_id, treasury_client, _, _) = setup_test();

    let config = treasury_client.get_treasury_config();
    assert_eq!(config.admin, admin);
    assert_eq!(config.asset, token_id);
    assert_eq!(config.status, TreasuryStatus::Active);

    // Re-initialization must fail
    let second_admin = Address::generate(&env);
    let res = treasury_client.try_initialize(&second_admin, &token_id, &symbol_short!("ORG2"));
    assert!(res.is_err());
}

#[test]
fn test_member_management() {
    let (env, admin, _, _, treasury_client, _, _) = setup_test();

    let spender = Address::generate(&env);
    let approver = Address::generate(&env);

    treasury_client.add_member(&admin, &spender, &MemberRole::Spender);
    treasury_client.add_member(&admin, &approver, &MemberRole::Approver);

    let member_info = treasury_client.get_member_info(&spender);
    assert_eq!(member_info.role, MemberRole::Spender);
    assert_eq!(member_info.status, MemberStatus::Active);

    // Suspend member
    treasury_client.suspend_member(&admin, &spender);
    let suspended_info = treasury_client.get_member_info(&spender);
    assert_eq!(suspended_info.status, MemberStatus::Suspended);

    // Remove member
    treasury_client.remove_member(&admin, &spender);
    assert!(treasury_client.try_get_member_info(&spender).is_err());
}

#[test]
fn test_policy_validation_and_identifier_increment() {
    let (env, admin, _, _, treasury_client, _, _) = setup_test();
    let spender = Address::generate(&env);
    let viewer = Address::generate(&env);
    treasury_client.add_member(&admin, &spender, &MemberRole::Spender);
    treasury_client.add_member(&admin, &viewer, &MemberRole::Viewer);

    assert!(treasury_client
        .try_set_policy(&admin, &viewer, &500, &SpendingPeriod::Daily, &200, &1, &0)
        .is_err());
    assert!(treasury_client
        .try_set_policy(&admin, &spender, &500, &SpendingPeriod::Daily, &200, &0, &0)
        .is_err());

    treasury_client.set_policy(&admin, &spender, &500, &SpendingPeriod::Daily, &200, &1, &0);
    assert_eq!(treasury_client.get_treasury_config().next_policy_id, 2);
}

#[test]
fn test_deposit_and_balance() {
    let (env, _admin, _token_id, treasury_id, treasury_client, token_admin_client, token_client) =
        setup_test();

    let depositor = Address::generate(&env);
    token_admin_client.mint(&depositor, &1000);

    treasury_client.deposit(&depositor, &500);

    assert_eq!(token_client.balance(&treasury_id), 500);
    assert_eq!(token_client.balance(&depositor), 500);
}

#[test]
fn test_auto_approval_payment_within_threshold() {
    let (env, admin, _token_id, treasury_id, treasury_client, token_admin_client, token_client) =
        setup_test();

    let spender = Address::generate(&env);
    let recipient = Address::generate(&env);

    treasury_client.add_member(&admin, &spender, &MemberRole::Spender);
    treasury_client.set_policy(
        &admin,
        &spender,
        &500, // spending limit
        &SpendingPeriod::Daily,
        &200, // approval threshold (<= 200 is auto-approved)
        &1,   // 1 approval required if > 200
        &0,
    );

    // Deposit funds into treasury
    let depositor = Address::generate(&env);
    token_admin_client.mint(&depositor, &1000);
    treasury_client.deposit(&depositor, &1000);

    let metadata_hash = BytesN::from_array(&env, &[0u8; 32]);

    // Payment of 150 (<= threshold 200) auto-executes immediately
    let req_id = treasury_client.request_payment(&spender, &recipient, &150, &metadata_hash, &0);
    assert_eq!(req_id, 1);

    let req = treasury_client.get_payment_request(&1);
    assert_eq!(req.status, PaymentStatus::Executed);
    assert_eq!(token_client.balance(&recipient), 150);
    assert_eq!(token_client.balance(&treasury_id), 850);
    assert_eq!(treasury_client.get_spent_allowance(&spender), 150);
}

#[test]
fn test_approval_workflow_for_payment_above_threshold() {
    let (env, admin, _token_id, treasury_id, treasury_client, token_admin_client, token_client) =
        setup_test();

    let spender = Address::generate(&env);
    let approver1 = Address::generate(&env);
    let approver2 = Address::generate(&env);
    let recipient = Address::generate(&env);

    treasury_client.add_member(&admin, &spender, &MemberRole::Spender);
    treasury_client.add_member(&admin, &approver1, &MemberRole::Approver);
    treasury_client.add_member(&admin, &approver2, &MemberRole::Approver);

    treasury_client.set_policy(
        &admin,
        &spender,
        &1000, // spending limit
        &SpendingPeriod::Monthly,
        &200, // approval threshold
        &2,   // requires 2 approvals if > 200
        &0,
    );

    token_admin_client.mint(&admin, &2000);
    treasury_client.deposit(&admin, &2000);

    let metadata_hash = BytesN::from_array(&env, &[1u8; 32]);

    // Request 500 (> 200 threshold) -> PendingApproval
    let req_id = treasury_client.request_payment(&spender, &recipient, &500, &metadata_hash, &0);
    let req = treasury_client.get_payment_request(&req_id);
    assert_eq!(req.status, PaymentStatus::PendingApproval);

    // Execution fails before sufficient approvals
    let exec_res = treasury_client.try_execute_payment(&spender, &req_id);
    assert!(exec_res.is_err());

    // Approver 1 approves
    treasury_client.approve_payment(&approver1, &req_id);
    assert_eq!(
        treasury_client.has_approved_request(&req_id, &approver1),
        true
    );

    // Approver 2 approves -> request status becomes Approved
    treasury_client.approve_payment(&approver2, &req_id);
    let req_approved = treasury_client.get_payment_request(&req_id);
    assert_eq!(req_approved.status, PaymentStatus::Approved);

    // Execute payment
    treasury_client.execute_payment(&spender, &req_id);

    let req_executed = treasury_client.get_payment_request(&req_id);
    assert_eq!(req_executed.status, PaymentStatus::Executed);
    assert_eq!(token_client.balance(&recipient), 500);
    assert_eq!(token_client.balance(&treasury_id), 1500);

    // Double execution must fail (Idempotency invariant)
    let double_exec = treasury_client.try_execute_payment(&spender, &req_id);
    assert!(double_exec.is_err());
}

#[test]
fn test_spending_allowance_exceeded_rejection() {
    let (env, admin, _token_id, _treasury_id, treasury_client, token_admin_client, _) =
        setup_test();

    let spender = Address::generate(&env);
    let recipient = Address::generate(&env);

    treasury_client.add_member(&admin, &spender, &MemberRole::Spender);
    treasury_client.set_policy(
        &admin,
        &spender,
        &300, // spending limit = 300
        &SpendingPeriod::Daily,
        &1000, // threshold high so auto-executes if within limit
        &0,
        &0,
    );

    token_admin_client.mint(&admin, &1000);
    treasury_client.deposit(&admin, &1000);

    let metadata_hash = BytesN::from_array(&env, &[0u8; 32]);

    // First spend 200 -> OK
    treasury_client.request_payment(&spender, &recipient, &200, &metadata_hash, &0);
    assert_eq!(treasury_client.get_spent_allowance(&spender), 200);

    // Attempting to spend 150 more (200+150=350 > 300 limit) -> Fails AllowanceExceeded
    let res = treasury_client.try_request_payment(&spender, &recipient, &150, &metadata_hash, &0);
    assert!(res.is_err());
    assert_eq!(treasury_client.get_spent_allowance(&spender), 200); // Allowance unconsumed on fail
}

#[test]
fn test_recurring_period_rollover() {
    let (env, admin, _token_id, _treasury_id, treasury_client, token_admin_client, _) =
        setup_test();

    let spender = Address::generate(&env);
    let recipient = Address::generate(&env);

    treasury_client.add_member(&admin, &spender, &MemberRole::Spender);
    treasury_client.set_policy(
        &admin,
        &spender,
        &500, // limit 500 daily
        &SpendingPeriod::Daily,
        &500,
        &0,
        &0,
    );

    token_admin_client.mint(&admin, &2000);
    treasury_client.deposit(&admin, &2000);

    let metadata_hash = BytesN::from_array(&env, &[0u8; 32]);

    // Spend 400 today
    treasury_client.request_payment(&spender, &recipient, &400, &metadata_hash, &0);
    assert_eq!(treasury_client.get_spent_allowance(&spender), 400);

    // Fast forward ledger time by 1 day (86,400 seconds)
    let current_time = env.ledger().timestamp();
    env.ledger().set_timestamp(current_time + 86400);

    // Spent allowance in new period resets to 0
    assert_eq!(treasury_client.get_spent_allowance(&spender), 0);

    // Can spend 500 again in the new period!
    treasury_client.request_payment(&spender, &recipient, &500, &metadata_hash, &0);
    assert_eq!(treasury_client.get_spent_allowance(&spender), 500);
}

#[test]
fn test_stale_policy_version_invalidation() {
    let (env, admin, _token_id, _treasury_id, treasury_client, token_admin_client, _) =
        setup_test();

    let spender = Address::generate(&env);
    let approver = Address::generate(&env);
    let recipient = Address::generate(&env);

    treasury_client.add_member(&admin, &spender, &MemberRole::Spender);
    treasury_client.add_member(&admin, &approver, &MemberRole::Approver);

    // Version 1 policy
    treasury_client.set_policy(
        &admin,
        &spender,
        &1000,
        &SpendingPeriod::Monthly,
        &100, // >100 requires approval
        &1,
        &0,
    );

    token_admin_client.mint(&admin, &2000);
    treasury_client.deposit(&admin, &2000);

    let metadata_hash = BytesN::from_array(&env, &[0u8; 32]);

    // Request 300 under Policy V1
    let req_id = treasury_client.request_payment(&spender, &recipient, &300, &metadata_hash, &0);

    // Admin updates policy -> creates Version 2 policy
    treasury_client.set_policy(
        &admin,
        &spender,
        &500,
        &SpendingPeriod::Monthly,
        &100,
        &1,
        &0,
    );

    // Approval of request created under V1 when policy is now V2 returns PolicyVersionMismatch!
    let app_res = treasury_client.try_approve_payment(&approver, &req_id);
    assert!(app_res.is_err());
}
