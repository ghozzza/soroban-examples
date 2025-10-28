#![cfg(test)]
extern crate std;

use crate::{contract::IDRX, IDRXClient};
use soroban_sdk::{
    log, symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Env, IntoVal, Symbol,
};

fn create_token<'a>(
    e: &Env,
    admin: &Address,
    pauser: &Address,
    upgrader: &Address,
    minter: &Address,
    manager: &Address,
) -> IDRXClient<'a> {
    let token_contract = e.register(IDRX, (admin.clone(), pauser.clone(), upgrader.clone(), minter.clone(), manager.clone()));
    IDRXClient::new(e, &token_contract)
}

#[test]
fn test_mint() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let pauser = Address::generate(&e);
    let upgrader = Address::generate(&e);
    let minter = Address::generate(&e);
    let manager = Address::generate(&e);
    let user1 = Address::generate(&e);

    let token = create_token(&e, &admin, &pauser, &upgrader, &minter, &manager);
    
    // Mint tokens to user1 using the minter
    token.mint(&user1, &1000, &minter);
    
    // Verify auth was called
    assert_eq!(
        e.auths(),
        std::vec![(
            minter.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (&user1, 1000_i128, minter.clone()).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    
    // Verify balance
    assert_eq!(token.balance(&user1), 1000);
}

#[test]
fn test_multiple_mints() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let pauser = Address::generate(&e);
    let upgrader = Address::generate(&e);
    let minter = Address::generate(&e);
    let manager = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);

    let token = create_token(&e, &admin, &pauser, &upgrader, &minter, &manager);
    
    // Mint to multiple users
    token.mint(&user1, &1000, &minter);
    token.mint(&user2, &2500, &minter);
    
    assert_eq!(token.balance(&user1), 1000);
    assert_eq!(token.balance(&user2), 2500);
    assert_eq!(token.total_supply(), 3500);
}