use soroban_sdk::{symbol_short, Address, Env, Symbol};

pub fn emit_treasury_created(env: &Env, admin: Address, asset: Address) {
    let topics = (symbol_short!("created"), admin);
    env.events().publish(topics, asset);
}

pub fn emit_treasury_status_changed(env: &Env, status_symbol: Symbol) {
    let topics = (symbol_short!("status"), status_symbol);
    env.events().publish(topics, ());
}

pub fn emit_treasury_deposited(env: &Env, depositor: Address, asset: Address, amount: i128) {
    let topics = (symbol_short!("deposit"), depositor, asset);
    env.events().publish(topics, amount);
}

pub fn emit_member_added(env: &Env, member: Address, role_symbol: Symbol) {
    let topics = (symbol_short!("mem_add"), member);
    env.events().publish(topics, role_symbol);
}

pub fn emit_member_removed(env: &Env, member: Address) {
    let topics = (symbol_short!("mem_rem"), member);
    env.events().publish(topics, ());
}

pub fn emit_member_suspended(env: &Env, member: Address) {
    let topics = (symbol_short!("mem_susp"), member);
    env.events().publish(topics, ());
}

pub fn emit_policy_updated(env: &Env, spender: Address, version: u32, limit: i128) {
    let topics = (symbol_short!("policy"), spender);
    env.events().publish(topics, (version, limit));
}

pub fn emit_policy_disabled(env: &Env, spender: Address) {
    let topics = (symbol_short!("pol_dis"), spender);
    env.events().publish(topics, ());
}

pub fn emit_payment_requested(env: &Env, request_id: u64, spender: Address, amount: i128) {
    let topics = (symbol_short!("req_pay"), request_id, spender);
    env.events().publish(topics, amount);
}

pub fn emit_payment_approved(
    env: &Env,
    request_id: u64,
    approver: Address,
    current_approvals: u32,
) {
    let topics = (symbol_short!("app_pay"), request_id, approver);
    env.events().publish(topics, current_approvals);
}

pub fn emit_approval_revoked(env: &Env, request_id: u64, approver: Address) {
    let topics = (symbol_short!("rev_app"), request_id, approver);
    env.events().publish(topics, ());
}

pub fn emit_payment_executed(
    env: &Env,
    request_id: u64,
    spender: Address,
    recipient: Address,
    amount: i128,
) {
    let topics = (symbol_short!("exec_pay"), request_id, spender);
    env.events().publish(topics, (recipient, amount));
}

pub fn emit_payment_cancelled(env: &Env, request_id: u64, spender: Address) {
    let topics = (symbol_short!("can_pay"), request_id, spender);
    env.events().publish(topics, ());
}
