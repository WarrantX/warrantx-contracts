use crate::storage::DataKey;
use soroban_sdk::{Address, Env};

pub fn has_approved(env: &Env, request_id: u64, approver: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::Approval(request_id, approver.clone()))
        .unwrap_or(false)
}

pub fn set_approval(env: &Env, request_id: u64, approver: &Address, approved: bool) {
    if approved {
        env.storage()
            .persistent()
            .set(&DataKey::Approval(request_id, approver.clone()), &true);
    } else {
        env.storage()
            .persistent()
            .remove(&DataKey::Approval(request_id, approver.clone()));
    }
}
