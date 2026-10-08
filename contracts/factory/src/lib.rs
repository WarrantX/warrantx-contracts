#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, BytesN, Env, IntoVal, Symbol, Vec};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FactoryError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    TreasuryWasmHash,
    Treasuries,
}

#[contract]
pub struct FactoryContract;

#[contractimpl]
impl FactoryContract {
    pub fn initialize(env: Env, admin: Address, treasury_wasm_hash: BytesN<32>) -> Result<(), FactoryError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(FactoryError::AlreadyInitialized);
        }
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TreasuryWasmHash, &treasury_wasm_hash);

        let treasuries: Vec<Address> = Vec::new(&env);
        env.storage().instance().set(&DataKey::Treasuries, &treasuries);

        Ok(())
    }

    pub fn set_wasm_hash(env: Env, admin: Address, new_wasm_hash: BytesN<32>) -> Result<(), FactoryError> {
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).ok_or(FactoryError::NotInitialized)?;
        if stored_admin != admin {
            return Err(FactoryError::Unauthorized);
        }
        admin.require_auth();

        env.storage().instance().set(&DataKey::TreasuryWasmHash, &new_wasm_hash);
        Ok(())
    }

    pub fn deploy_treasury(
        env: Env,
        deployer: Address,
        salt: BytesN<32>,
        admin: Address,
        asset: Address,
        org_id: Symbol,
    ) -> Result<Address, FactoryError> {
        deployer.require_auth();
        let wasm_hash: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::TreasuryWasmHash)
            .ok_or(FactoryError::NotInitialized)?;

        let treasury_address = env.deployer().with_current_contract(salt).deploy_v2(wasm_hash, ());

        // Initialize deployed treasury
        let init_fn = Symbol::new(&env, "initialize");
        let mut init_args = Vec::new(&env);
        init_args.push_back(admin.into_val(&env));
        init_args.push_back(asset.into_val(&env));
        init_args.push_back(org_id.into_val(&env));

        let _: () = env.invoke_contract(&treasury_address, &init_fn, init_args);

        let mut treasuries: Vec<Address> = env.storage().instance().get(&DataKey::Treasuries).unwrap();
        treasuries.push_back(treasury_address.clone());
        env.storage().instance().set(&DataKey::Treasuries, &treasuries);

        Ok(treasury_address)
    }

    pub fn get_treasuries(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::Treasuries)
            .unwrap_or(Vec::new(&env))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_factory_initialization() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let dummy_hash = BytesN::from_array(&env, &[0u8; 32]);

        let factory_id = env.register(FactoryContract, ());
        let client = FactoryContractClient::new(&env, &factory_id);

        client.initialize(&admin, &dummy_hash);
        assert_eq!(client.get_treasuries().len(), 0);
    }
}
