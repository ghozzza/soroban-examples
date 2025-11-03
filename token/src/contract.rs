// SPDX-License-Identifier: MIT
// Compatible with OpenZeppelin Stellar Soroban Contracts ^0.4.1

use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol};
use stellar_access::access_control::{self as access_control, AccessControl};
use stellar_contract_utils::pausable::{self as pausable, Pausable};
use stellar_contract_utils::upgradeable::UpgradeableInternal;
use stellar_macros::{default_impl, only_role, when_not_paused, Upgradeable};
use stellar_tokens::fungible::{
    blocklist::{BlockList, FungibleBlockList},
    burnable::FungibleBurnable,
    Base, FungibleToken,
};

fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}

// Events are defined using symbols for topics

#[derive(Upgradeable)]
#[contract]
pub struct IDRX;

#[contractimpl]
impl IDRX {
    pub fn __constructor(
        e: &Env,
        admin: Address,
        pauser: Address,
        upgrader: Address,
        minter: Address,
        manager: Address,
    ) {
        Base::set_metadata(
            e,
            2,
            String::from_str(e, "IDRX"),
            String::from_str(e, "IDRX"),
        );
        access_control::set_admin(e, &admin);
        access_control::grant_role_no_auth(e, &admin, &pauser, &Symbol::new(e, "pauser"));
        access_control::grant_role_no_auth(e, &admin, &upgrader, &Symbol::new(e, "upgrader"));
        access_control::grant_role_no_auth(e, &admin, &minter, &Symbol::new(e, "minter"));
        access_control::grant_role_no_auth(e, &admin, &manager, &Symbol::new(e, "manager"));
    }

    #[only_role(caller, "minter")]
    #[when_not_paused]
    pub fn mint(e: &Env, account: Address, amount: i128, caller: Address) {
        Base::mint(e, &account, amount);
    }

    // Note: invoker() method may not be available in soroban-sdk v22.0.8
    // This function demonstrates how to get the current contract address instead
    pub fn get_contract_address(e: &Env) -> Address {
        e.current_contract_address()
    }

    #[when_not_paused]
    pub fn burn_with_account_number(e: &Env, from: Address, amount: i128, account_number: String) {
        // Check non-negative amount
        check_nonnegative_amount(amount);
        
        // Perform the burn (Base::burn already handles authorization)
        Base::burn(e, &from, amount);

        // Emit custom event using topics and data
        e.events().publish(
            (Symbol::new(e, "burn_with_account_number"),),  // Topics (indexed)
            (from, amount, account_number)           // Data (non-indexed)
        );
    }
    
    // #[only_role(caller, "minter")]
    // #[when_not_paused]
    // pub fn mint_with_event(e: &Env, account: Address, amount: i128, caller: Address) {
    //     // Perform the mint
    //     Base::mint(e, &account, amount);
        
    //     // Emit custom mint event
    //     e.events().publish(
    //         (Symbol::new(e, "custom_mint"), caller.clone()),  // Topics: event name + minter
    //         (account, amount)                                 // Data: recipient + amount  
    //     );
    // }
}

#[default_impl]
#[contractimpl]
impl FungibleToken for IDRX {
    type ContractType = BlockList;

    #[when_not_paused]
    fn transfer(e: &Env, from: Address, to: Address, amount: i128) {
        check_nonnegative_amount(amount);
        Self::ContractType::transfer(e, &from, &to, amount);
    }

    #[when_not_paused]
    fn transfer_from(e: &Env, spender: Address, from: Address, to: Address, amount: i128) {
        check_nonnegative_amount(amount);
        Self::ContractType::transfer_from(e, &spender, &from, &to, amount);
    }
}

//
// Extensions
//

#[contractimpl]
impl FungibleBlockList for IDRX {
    fn blocked(e: &Env, account: Address) -> bool {
        BlockList::blocked(e, &account)
    }

    #[only_role(operator, "manager")]
    fn block_user(e: &Env, user: Address, operator: Address) {
        BlockList::block_user(e, &user);
    }

    #[only_role(operator, "manager")]
    fn unblock_user(e: &Env, user: Address, operator: Address) {
        BlockList::unblock_user(e, &user);
    }
}

#[contractimpl]
impl FungibleBurnable for IDRX {
    #[when_not_paused]
    fn burn(e: &Env, from: Address, amount: i128) {
        Base::burn(e, &from, amount);
    }

    #[when_not_paused]
    fn burn_from(e: &Env, spender: Address, from: Address, amount: i128) {
        Base::burn_from(e, &spender, &from, amount);
    }
}

//
// Utils
//

impl UpgradeableInternal for IDRX {
    fn _require_auth(e: &Env, operator: &Address) {
        access_control::ensure_role(e, operator, &Symbol::new(e, "upgrader"));
        operator.require_auth();
    }
}

#[contractimpl]
impl Pausable for IDRX {
    fn paused(e: &Env) -> bool {
        pausable::paused(e)
    }

    #[only_role(caller, "pauser")]
    fn pause(e: &Env, caller: Address) {
        pausable::pause(e);
    }

    #[only_role(caller, "pauser")]
    fn unpause(e: &Env, caller: Address) {
        pausable::unpause(e);
    }
}

#[default_impl]
#[contractimpl]
impl AccessControl for IDRX {}
