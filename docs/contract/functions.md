# Contract functions

Treasury writes: `initialize`, `deposit`, `add_member`, `remove_member`, `suspend_member`, `set_policy`, `disable_policy`, `request_payment`, `approve_payment`, `revoke_approval`, `cancel_payment`, `execute_payment`, `suspend_treasury`, and `reactivate_treasury`.

Treasury reads: `get_treasury_config`, `get_member_info`, `get_policy`, `get_spent_allowance`, `get_payment_request`, and `has_approved_request`.

Factory functions: `initialize`, `set_wasm_hash`, `deploy_treasury`, and `get_treasuries`. Refer to the Rust signatures for exact Soroban types.

