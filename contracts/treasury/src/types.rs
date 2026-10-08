use soroban_sdk::{contracttype, Address, BytesN, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TreasuryStatus {
    Active,
    Suspended,
    Closed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MemberRole {
    Admin,
    Approver,
    Spender,
    Viewer,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MemberStatus {
    Active,
    Suspended,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemberInfo {
    pub address: Address,
    pub role: MemberRole,
    pub status: MemberStatus,
    pub added_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpendingPeriod {
    Daily,   // 86400s boundaries
    Weekly,  // 604800s boundaries
    Monthly, // ~30 days (2592000s) deterministic ledger boundary
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpendingPolicy {
    pub policy_id: u64,
    pub spender: Address,
    pub asset: Address,
    pub spending_limit: i128,
    pub period: SpendingPeriod,
    pub approval_threshold: i128,
    pub required_approvals: u32,
    pub version: u32,
    pub active: bool,
    pub created_at: u64,
    pub expires_at: u64, // 0 = no expiration
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PaymentStatus {
    PendingApproval,
    Approved,
    Executed,
    Cancelled,
    Rejected,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentRequest {
    pub request_id: u64,
    pub spender: Address,
    pub recipient: Address,
    pub asset: Address,
    pub amount: i128,
    pub metadata_hash: BytesN<32>,
    pub policy_version: u32,
    pub created_at: u64,
    pub expires_at: u64,
    pub status: PaymentStatus,
    pub approval_count: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryConfig {
    pub org_id: Symbol,
    pub admin: Address,
    pub asset: Address,
    pub status: TreasuryStatus,
    pub created_at: u64,
    pub next_request_id: u64,
    pub next_policy_id: u64,
}
