use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    TreasurySuspended = 4,
    TreasuryClosed = 5,
    MemberAlreadyExists = 6,
    MemberNotFound = 7,
    MemberSuspended = 8,
    InvalidRole = 9,
    PolicyNotFound = 10,
    PolicyDisabled = 11,
    PolicyVersionMismatch = 12,
    InvalidAmount = 13,
    AllowanceExceeded = 14,
    ApprovalThresholdNotMet = 15,
    AlreadyApproved = 16,
    RequestNotFound = 17,
    RequestExpired = 18,
    RequestCancelled = 19,
    RequestAlreadyExecuted = 20,
    SelfApprovalNotAllowed = 21,
    InsufficientTreasuryBalance = 22,
    InvalidPeriod = 23,
    Overflow = 24,
    InvalidAsset = 25,
}
