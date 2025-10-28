// SPDX-License-Identifier: MIT
// Compatible with OpenZeppelin Stellar Soroban Contracts ^0.4.1

use soroban_sdk::{Address, contract, contractimpl, Env, String, Symbol};
use stellar_access::access_control::{self as access_control, AccessControl};
use stellar_contract_utils::pausable::{self as pausable, Pausable};
use stellar_contract_utils::upgradeable::UpgradeableInternal;
use stellar_macros::{default_impl, only_role, Upgradeable, when_not_paused};
use stellar_tokens::fungible::{
    Base, blocklist::{BlockList, FungibleBlockList}, burnable::FungibleBurnable, FungibleToken
};

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
        Base::set_metadata(e, 18, String::from_str(e, "IDRX"), String::from_str(e, "IDRX"));
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
}

#[default_impl]
#[contractimpl]
impl FungibleToken for IDRX {
    type ContractType = BlockList;

    #[when_not_paused]
    fn transfer(e: &Env, from: Address, to: Address, amount: i128) {
        Self::ContractType::transfer(e, &from, &to, amount);
    }

    #[when_not_paused]
    fn transfer_from(e: &Env, spender: Address, from: Address, to: Address, amount: i128) {
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
