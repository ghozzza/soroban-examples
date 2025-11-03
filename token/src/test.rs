#![cfg(test)]
extern crate std;

use crate::{contract::IDRX, IDRXClient};
use soroban_sdk::{
    log, symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal, String, Symbol,
};

fn create_token<'a>(
    e: &Env,
    admin: &Address,
    pauser: &Address,
    upgrader: &Address,
    minter: &Address,
    manager: &Address,
) -> IDRXClient<'a> {
    let token_contract = e.register(
        IDRX,
        (
            admin.clone(),
            pauser.clone(),
            upgrader.clone(),
            minter.clone(),
            manager.clone(),
        ),
    );
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

#[test]
// cargo test test_contract_address -- --nocapture
fn test_contract_address() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let pauser = Address::generate(&e);
    let upgrader = Address::generate(&e);
    let minter = Address::generate(&e);
    let manager = Address::generate(&e);

    let token = create_token(&e, &admin, &pauser, &upgrader, &minter, &manager);

    // Get the contract's own address
    let contract_addr = token.get_contract_address();
    log!(&e, "Contract address: {}", contract_addr);

    // The contract address should match the token's address
    assert_eq!(contract_addr, token.address);
}

#[test]
// cargo test test_contract_functionality -- --nocapture
fn test_contract_functionality() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let pauser = Address::generate(&e);
    let upgrader = Address::generate(&e);
    let minter = Address::generate(&e);
    let manager = Address::generate(&e);
    let user1 = Address::generate(&e);

    let token = create_token(&e, &admin, &pauser, &upgrader, &minter, &manager);

    // Test that get_contract_address works
    let contract_addr = token.get_contract_address();
    log!(&e, "Contract address in test: {}", contract_addr);

    // Test minting still works
    token.mint(&user1, &1000, &minter);
    log!(&e, "token.balance(&user1): {}", token.balance(&user1));
    assert_eq!(token.balance(&user1), 1000);
}

#[test]
// cargo test test_transfer -- --nocapture
fn test_transfer() {
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

    // Test that transfer works
    token.mint(&user1, &1000, &minter);
    token.transfer(&user1, &user2, &1000);

    log!(&e, "token.balance(&user2): {}", token.balance(&user2));
    assert_eq!(token.balance(&user2), 1000);
}

#[test]
// cargo test test_who_executes_transfer -- --nocapture
fn test_who_executes_transfer() {
    let e = Env::default();
    // Don't mock auths initially to see authorization requirements

    let admin = Address::generate(&e);
    let pauser = Address::generate(&e);
    let upgrader = Address::generate(&e);
    let minter = Address::generate(&e);
    let manager = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);

    let token = create_token(&e, &admin, &pauser, &upgrader, &minter, &manager);

    // First mint with mocked auth
    // e.mock_all_auths();
    // token.mint(&user1, &1000, &minter);

    // Now reset auths and do transfer to see who needs to authorize
    // e.set_auths(&[]);

    e.mock_auths(&[MockAuth {
        address: &minter,
        invoke: &MockAuthInvoke {
            contract: &token.address,
            fn_name: "mint",
            args: (&user1, 1000_i128, &minter).into_val(&e),
            sub_invokes: &[],
        },
    }]);
    
    // Actually call mint with the mocked authorization
    token.mint(&user1, &1000, &minter);
    
    log!(&e, "After mint - user1 balance: {}", token.balance(&user1));

    // Now reset auths and mock transfer authorization
    e.set_auths(&[]);

    // Note: In testing, we need to mock the auth for user1
    // e.mock_all_auths();
    
    // Mock auth for transfer
    e.mock_auths(&[MockAuth {
        address: &user1,
        invoke: &MockAuthInvoke {
            contract: &token.address,
            fn_name: "transfer", 
            args: (&user1, &user2, 1000_i128).into_val(&e),
            sub_invokes: &[],
        },
    }]);
    
    // Execute transfer
    token.transfer(&user1, &user2, &1000);

    // Check auths to see who was required to authorize
    let auths = e.auths();

    if !auths.is_empty() {
        log!(&e, "Authorization required from: {}", auths[0].0);
        match &auths[0].1.function {
            AuthorizedFunction::Contract((contract_addr, symbol, _args)) => {
                log!(&e, "Contract: {}", contract_addr);
                log!(&e, "Function: {}", symbol);
            }
            _ => {}
        }
    }
    assert_eq!(token.balance(&user2), 1000);
}

#[test]
// cargo test test_transfer_authorization_flow -- --nocapture
fn test_transfer_authorization_flow() {
    let e = Env::default();

    let admin = Address::generate(&e);
    let pauser = Address::generate(&e);
    let upgrader = Address::generate(&e);
    let minter = Address::generate(&e);
    let manager = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);

    let token = create_token(&e, &admin, &pauser, &upgrader, &minter, &manager);

    // Mint tokens with mocked auth
    e.mock_all_auths();
    token.mint(&user1, &1000, &minter);

    log!(&e, "=== UNDERSTANDING TRANSFER EXECUTION ===");
    log!(&e, "In your test: token.transfer(&user1, &user2, &1000)");
    log!(&e, "");
    log!(&e, "1. EXECUTOR/INVOKER: Your test code");
    log!(&e, "   - The test calls the token contract");
    log!(
        &e,
        "   - In real world: could be a dApp, another contract, etc"
    );
    log!(&e, "");
    log!(&e, "2. AUTHORIZER: user1 (the 'from' address)");
    log!(&e, "   - user1 must approve moving their tokens");
    log!(
        &e,
        "   - This is enforced by from.require_auth() in the contract"
    );
    log!(&e, "");
    log!(&e, "3. RECIPIENT: user2 (passive)");
    log!(&e, "   - user2 receives tokens but doesn't need to sign");
    log!(&e, "");

    // Show the actual transfer with auth tracking
    e.set_auths(&[]);

    log!(&e, "4. ATTEMPTING TRANSFER WITHOUT AUTHORIZATION:");
    // This would fail without authorization (commented out to avoid panic)
    // token.transfer(&user1, &user2, &1000); // Would throw: "Unauthorized function call"

    log!(&e, "5. NOW WITH PROPER AUTHORIZATION:");
    e.mock_all_auths(); // Mock user1's authorization
    token.transfer(&user1, &user2, &1000);

    assert_eq!(token.balance(&user1), 0);
    assert_eq!(token.balance(&user2), 1000);

    log!(&e, "Transfer completed successfully!");
    log!(&e, "");
    log!(&e, "SUMMARY:");
    log!(&e, "- Test code EXECUTES the transfer");
    log!(&e, "- user1 AUTHORIZES moving their tokens");
    log!(&e, "- user2 RECEIVES tokens (no auth needed)");
}

#[test]
// cargo test test_events -- --nocapture
fn test_events() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let pauser = Address::generate(&e);
    let upgrader = Address::generate(&e);
    let minter = Address::generate(&e);
    let manager = Address::generate(&e);
    let user1 = Address::generate(&e);

    let token = create_token(&e, &admin, &pauser, &upgrader, &minter, &manager);

    log!(&e, "=== TESTING SOROBAN EVENTS ===");
    
    // Test custom mint with event
    token.mint(&user1, &500, &minter);
    log!(&e, "Custom mint event emitted");
    
    // Test custom burn with account number
    token.burn_with_account_number(&user1, &200, &String::from_str(&e, "ACC-123456"));
    log!(&e, "Custom burn event emitted");
    
    // Verify balances
    assert_eq!(token.balance(&user1), 300); // 500 minted - 200 burned
    
    log!(&e, "Final balance: {}", token.balance(&user1));
    log!(&e, "Events successfully emitted! Check the event log above.");
}
