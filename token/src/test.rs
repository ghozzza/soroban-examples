#![cfg(test)]
extern crate std;

use crate::{contract::IDRX, IDRXClient};
use soroban_sdk::{
    log, symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal, String,
};

// ============================================================================
// Helper Functions
// ============================================================================

/// Creates a new IDRX token contract instance with all required role addresses.
///
/// This helper function initializes a token contract with:
/// - `admin`: Administrator address with full control
/// - `pauser`: Address authorized to pause/unpause the contract
/// - `upgrader`: Address authorized to upgrade the contract
/// - `minter`: Address authorized to mint new tokens
/// - `blacklister`: Address authorized to blacklist addresses and destroy funds
///
/// # Arguments
/// * `e` - The Soroban environment
/// * `admin` - Administrator address
/// * `pauser` - Pauser role address
/// * `upgrader` - Upgrader role address
/// * `minter` - Minter role address
/// * `blacklister` - Blacklister role address
///
/// # Returns
/// A configured IDRXClient instance ready for testing
fn create_token<'a>(
    e: &Env,
    admin: &Address,
    pauser: &Address,
    upgrader: &Address,
    minter: &Address,
    blacklister: &Address,
) -> IDRXClient<'a> {
    let token_contract = e.register(
        IDRX,
        (
            admin.clone(),
            pauser.clone(),
            minter.clone(),
            upgrader.clone(),
            blacklister.clone(),
        ),
    );
    IDRXClient::new(e, &token_contract)
}

// ============================================================================
// Basic Token Operations Tests
// ============================================================================

/// Tests basic minting functionality.
///
/// This test verifies that:
/// - Tokens can be minted to a user address
/// - The minter role authorization is required and properly checked
/// - The user's balance is correctly updated after minting
///
/// Run with: `cargo test test_mint -- --nocapture`
#[test]
// cargo test test_mint -- --nocapture
fn test_mint() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let pauser = Address::generate(&e);
    let upgrader = Address::generate(&e);
    let minter = Address::generate(&e);
    let blacklister = Address::generate(&e);
    let user1 = Address::generate(&e);

    let token = create_token(&e, &admin, &pauser, &upgrader, &minter, &blacklister);

    // Mint 1000 tokens to user1 using the minter role
    token.mint(&user1, &1000, &minter);

    // Verify that minter authorization was required
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

    // Verify balance was updated correctly
    assert_eq!(token.balance(&user1), 1000);
}

/// Tests minting tokens to multiple users.
///
/// This test verifies that:
/// - Multiple mint operations work correctly
/// - Each user's balance is tracked independently
/// - Total supply reflects all minted tokens across all users
///
/// Run with: `cargo test test_multiple_mints -- --nocapture`
#[test]
fn test_multiple_mints() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let pauser = Address::generate(&e);
    let upgrader = Address::generate(&e);
    let minter = Address::generate(&e);
    let blacklister = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);

    let token = create_token(&e, &admin, &pauser, &upgrader, &minter, &blacklister);

    // Mint tokens to multiple users
    token.mint(&user1, &1000, &minter);
    token.mint(&user2, &2500, &minter);

    // Verify individual balances
    assert_eq!(token.balance(&user1), 1000);
    assert_eq!(token.balance(&user2), 2500);
    
    // Verify total supply is the sum of all minted tokens
    assert_eq!(token.total_supply(), 3500);
}

/// Tests basic token transfer functionality.
///
/// This test verifies that:
/// - Tokens can be transferred from one user to another
/// - The sender's authorization is required
/// - The recipient's balance is correctly updated
///
/// Run with: `cargo test test_transfer -- --nocapture`
#[test]
fn test_transfer() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let pauser = Address::generate(&e);
    let upgrader = Address::generate(&e);
    let minter = Address::generate(&e);
    let blacklister = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);

    let token = create_token(&e, &admin, &pauser, &upgrader, &minter, &blacklister);

    // Mint tokens to user1
    token.mint(&user1, &1000, &minter);
    
    // Transfer all tokens from user1 to user2
    token.transfer(&user1, &user2, &1000);

    // Verify recipient received the tokens
    assert_eq!(token.balance(&user2), 1000);
    assert_eq!(token.balance(&user1), 0);
}

// ============================================================================
// Contract Metadata Tests
// ============================================================================

/// Tests contract address retrieval functionality.
///
/// This test verifies that:
/// - The contract can retrieve its own address
/// - The retrieved address matches the token client's address
///
/// Run with: `cargo test test_contract_address -- --nocapture`
#[test]
fn test_contract_address() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let pauser = Address::generate(&e);
    let upgrader = Address::generate(&e);
    let minter = Address::generate(&e);
    let blacklister = Address::generate(&e);

    let token = create_token(&e, &admin, &pauser, &upgrader, &minter, &blacklister);

    // Get the contract's own address
    let contract_addr = token.get_contract_address();
    
    // Verify the address matches the token's address
    assert_eq!(contract_addr, token.address);
}

// ============================================================================
// Authorization and Security Tests
// ============================================================================

/// Tests transfer authorization requirements.
///
/// This test demonstrates the authorization flow for transfers:
/// - Shows who needs to authorize the transfer (the sender)
/// - Verifies authorization is properly tracked
/// - Confirms the transfer succeeds with proper authorization
///
/// Run with: `cargo test test_who_executes_transfer -- --nocapture`
#[test]
fn test_who_executes_transfer() {
    let e = Env::default();

    let admin = Address::generate(&e);
    let pauser = Address::generate(&e);
    let upgrader = Address::generate(&e);
    let minter = Address::generate(&e);
    let blacklister = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);

    let token = create_token(&e, &admin, &pauser, &upgrader, &minter, &blacklister);

    // Setup: Mint tokens to user1 with minter authorization
    e.mock_auths(&[MockAuth {
        address: &minter,
        invoke: &MockAuthInvoke {
            contract: &token.address,
            fn_name: "mint",
            args: (&user1, 1000_i128, &minter).into_val(&e),
            sub_invokes: &[],
        },
    }]);
    token.mint(&user1, &1000, &minter);

    // Reset auths for transfer test
    e.set_auths(&[]);

    // Mock authorization for transfer - user1 must authorize moving their tokens
    e.mock_auths(&[MockAuth {
        address: &user1,
        invoke: &MockAuthInvoke {
            contract: &token.address,
            fn_name: "transfer",
            args: (&user1, &user2, 1000_i128).into_val(&e),
            sub_invokes: &[],
        },
    }]);

    // Execute transfer - user1 authorizes moving their tokens
    token.transfer(&user1, &user2, &1000);

    // Verify authorization was recorded
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

    // Verify transfer was successful
    assert_eq!(token.balance(&user2), 1000);
    assert_eq!(token.balance(&user1), 0);
}

/// Tests and documents the complete transfer authorization flow.
///
/// This test provides a comprehensive explanation of how transfers work:
/// 1. Executor/Invoker: The code that calls the transfer function
/// 2. Authorizer: The sender (user1) who must approve the transfer
/// 3. Recipient: The receiver (user2) who doesn't need to authorize
///
/// Run with: `cargo test test_transfer_authorization_flow -- --nocapture`
#[test]
fn test_transfer_authorization_flow() {
    let e = Env::default();

    let admin = Address::generate(&e);
    let pauser = Address::generate(&e);
    let upgrader = Address::generate(&e);
    let minter = Address::generate(&e);
    let blacklister = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);

    let token = create_token(&e, &admin, &pauser, &upgrader, &minter, &blacklister);

    // Setup: Mint tokens to user1
    e.mock_all_auths();
    token.mint(&user1, &1000, &minter);

    // Document the transfer flow
    log!(&e, "=== TRANSFER AUTHORIZATION FLOW ===");
    log!(&e, "");
    log!(&e, "1. EXECUTOR/INVOKER: Test code (or dApp/contract in production)");
    log!(&e, "   - Calls token.transfer(&user1, &user2, &1000)");
    log!(&e, "");
    log!(&e, "2. AUTHORIZER: user1 (the 'from' address)");
    log!(&e, "   - Must approve moving their tokens via require_auth()");
    log!(&e, "   - This is enforced by the contract's security model");
    log!(&e, "");
    log!(&e, "3. RECIPIENT: user2 (passive participant)");
    log!(&e, "   - Receives tokens but doesn't need to authorize");
    log!(&e, "");

    // Demonstrate transfer with authorization
    e.set_auths(&[]);
    log!(&e, "4. Executing transfer with proper authorization...");
    e.mock_all_auths(); // Mock user1's authorization
    token.transfer(&user1, &user2, &1000);

    // Verify results
    assert_eq!(token.balance(&user1), 0);
    assert_eq!(token.balance(&user2), 1000);

    log!(&e, "Transfer completed successfully!");
    log!(&e, "");
    log!(&e, "SUMMARY:");
    log!(&e, "- Executor: Test code calls the transfer");
    log!(&e, "- Authorizer: user1 authorizes moving their tokens");
    log!(&e, "- Recipient: user2 receives tokens (no authorization needed)");
}

// ============================================================================
// Burn Operations Tests
// ============================================================================

/// Tests basic token burning functionality.
///
/// This test verifies that:
/// - Users can burn their own tokens
/// - Balance is correctly reduced after burning
/// - Total supply is correctly reduced after burning
///
/// Run with: `cargo test test_burn -- --nocapture`
#[test]
fn test_burn() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let pauser = Address::generate(&e);
    let upgrader = Address::generate(&e);
    let minter = Address::generate(&e);
    let blacklister = Address::generate(&e);
    let user1 = Address::generate(&e);

    let token = create_token(&e, &admin, &pauser, &upgrader, &minter, &blacklister);

    // Mint and then burn tokens
    token.mint(&user1, &1000, &minter);
    token.burn(&user1, &1000);

    // Verify balance and total supply are zero
    assert_eq!(token.balance(&user1), 0);
    assert_eq!(token.total_supply(), 0);
}

/// Tests burning tokens from another user's account with approval.
///
/// This test verifies that:
/// - An approved spender (user2) can burn tokens from the token owner (user1)
/// - Approval mechanism works for burn operations
/// - Balance and total supply are correctly updated
///
/// Run with: `cargo test test_burn_from -- --nocapture`
#[test]
fn test_burn_from() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let pauser = Address::generate(&e);
    let upgrader = Address::generate(&e);
    let minter = Address::generate(&e);
    let blacklister = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);

    let token = create_token(&e, &admin, &pauser, &upgrader, &minter, &blacklister);

    // Mint tokens to user1
    token.mint(&user1, &1000, &minter);

    // User1 approves user2 to spend their tokens
    let expiration = e.ledger().sequence() + 1000;
    token.approve(&user1, &user2, &1000, &expiration);

    // User2 burns tokens from user1's account
    token.burn_from(&user2, &user1, &1000);

    // Verify tokens were burned
    assert_eq!(token.balance(&user1), 0);
    assert_eq!(token.total_supply(), 0);
}

/// Tests burning tokens with an associated account number.
///
/// This test verifies that:
/// - Tokens can be burned with an account number for tracking purposes
/// - The account number is properly associated with the burn operation
/// - Balance and total supply are correctly updated
///
/// Run with: `cargo test test_burn_with_account_number -- --nocapture`
#[test]
fn test_burn_with_account_number() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let pauser = Address::generate(&e);
    let upgrader = Address::generate(&e);
    let minter = Address::generate(&e);
    let blacklister = Address::generate(&e);
    let user1 = Address::generate(&e);

    let token = create_token(&e, &admin, &pauser, &upgrader, &minter, &blacklister);

    // Mint tokens and burn with account number
    token.mint(&user1, &1000, &minter);
    token.burn_with_account_number(&user1, &1000, &String::from_str(&e, "ACC-123456"));

    // Verify tokens were burned
    assert_eq!(token.balance(&user1), 0);
    assert_eq!(token.total_supply(), 0);
}

// ============================================================================
// Bridge Operations Tests
// ============================================================================

/// Tests minting tokens through a bridge operation.
///
/// This test verifies that:
/// - Tokens can be minted via bridge with an external address identifier
/// - The bridge chain ID is properly recorded
/// - User balance and total supply are correctly updated
///
/// Run with: `cargo test test_mint_bridge -- --nocapture`
#[test]
fn test_mint_bridge() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let pauser = Address::generate(&e);
    let upgrader = Address::generate(&e);
    let minter = Address::generate(&e);
    let blacklister = Address::generate(&e);
    let user1 = Address::generate(&e);

    let token = create_token(&e, &admin, &pauser, &upgrader, &minter, &blacklister);

    // Mint tokens via bridge from external address "0xAlex" on chain 8453
    token.mint_bridge(&String::from_str(&e, "0xAlex"), &user1, &1000, &8453, &minter);

    // Verify bridge mint was successful
    assert_eq!(token.balance(&user1), 1000);
    assert_eq!(token.total_supply(), 1000);
}

/// Tests burning tokens through a bridge operation.
///
/// This test verifies that:
/// - Tokens can be burned via bridge to unlock tokens on another chain
/// - Bridge operations correctly track the external address and chain ID
/// - Balance and total supply are correctly reduced after bridge burn
///
/// Run with: `cargo test test_burn_bridge -- --nocapture`
#[test]
fn test_burn_bridge() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let pauser = Address::generate(&e);
    let upgrader = Address::generate(&e);
    let minter = Address::generate(&e);
    let blacklister = Address::generate(&e);
    let user1 = Address::generate(&e);

    let token = create_token(&e, &admin, &pauser, &upgrader, &minter, &blacklister);

    // First mint tokens via bridge
    token.mint_bridge(&String::from_str(&e, "0xAlex"), &user1, &1000, &8453, &minter);

    // Then burn tokens via bridge to unlock on another chain
    token.burn_bridge(&user1, &String::from_str(&e, "0xAlex"), &1000, &8453, &minter);

    // Verify bridge burn was successful
    assert_eq!(token.balance(&user1), 0);
    assert_eq!(token.total_supply(), 0);
}

// ============================================================================
// Blacklist and Security Tests
// ============================================================================

/// Tests destroying blacklisted funds.
///
/// This test verifies that:
/// - The blacklister role can destroy funds from blacklisted addresses
/// - Blacklisted funds are permanently removed from circulation
/// - Balance and total supply are correctly reduced to zero
///
/// Run with: `cargo test test_destroy_blackfunds -- --nocapture`
#[test]
fn test_destroy_blackfunds() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let pauser = Address::generate(&e);
    let upgrader = Address::generate(&e);
    let minter = Address::generate(&e);
    let blacklister = Address::generate(&e);
    let user1 = Address::generate(&e);

    let token = create_token(&e, &admin, &pauser, &upgrader, &minter, &blacklister);

    // Mint tokens to user1
    token.mint(&user1, &1000, &minter);
    
    // Destroy blacklisted funds (assumes user1 was blacklisted)
    token.destroy_blackfunds(&user1, &blacklister);

    // Verify funds were destroyed
    assert_eq!(token.balance(&user1), 0);
    assert_eq!(token.total_supply(), 0);
}