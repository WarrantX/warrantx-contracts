use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Config,
    Member(Address),
    Policy(Address),              // Spender -> Policy
    Allowance(Address, u64),      // Spender, PeriodId -> i128 spent amount
    Request(u64),                 // RequestId -> PaymentRequest
    Approval(u64, Address),       // RequestId, Approver -> bool
    ExecutedRequest(u64),         // RequestId -> bool (idempotency check)
}
