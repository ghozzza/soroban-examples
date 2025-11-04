// SPDX-License-Identifier: MIT
// Compatible with OpenZeppelin Stellar Soroban Contracts ^0.4.1

//! # IDRX Token Contract
//!
//! A comprehensive ERC-20-like fungible token contract for the Stellar Soroban blockchain
//! with advanced features including:
//!
//! - **Role-based access control**: Admin, Minter, Pauser, Upgrader, and Blacklister roles
//! - **Pausable functionality**: Ability to pause/unpause contract operations
//! - **Burnable tokens**: Users can burn their own tokens or authorized burners can burn from others
//! - **Blocklist support**: Ability to block/unblock addresses and destroy blacklisted funds
//! - **Bridge operations**: Support for cross-chain bridge minting and burning
//! - **Custom events**: Comprehensive event emission for all operations
//!
//! ## Roles
//!
//! - **Admin**: Full administrative control, can grant/revoke roles
//! - **Minter**: Authorized to mint new tokens
//! - **Pauser**: Authorized to pause/unpause contract operations
//! - **Upgrader**: Authorized to upgrade the contract
//! - **Blacklister**: Authorized to block/unblock users and destroy blacklisted funds

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

// ============================================================================
// Helper Functions
// ============================================================================

/// Validates that an amount is non-negative.
///
/// # Arguments
/// * `amount` - The amount to validate
///
/// # Panics
/// Panics if the amount is negative
fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}

// ============================================================================
// Contract Definition
// ============================================================================

/// IDRX Token Contract
///
/// An upgradeable, pausable, burnable fungible token with blocklist support
/// and bridge functionality. This contract implements multiple trait interfaces
/// to provide comprehensive token functionality.
#[derive(Upgradeable)]
#[contract]
pub struct IDRX;

// ============================================================================
// Contract Implementation - Core Functions
// ============================================================================

#[contractimpl]
impl IDRX {
    /// Initializes the IDRX token contract with role-based access control.
    ///
    /// This constructor sets up:
    /// - Token metadata (decimals, name, symbol)
    /// - Access control system with admin role
    /// - Role assignments for pauser, minter, upgrader, and blacklister
    ///
    /// # Arguments
    /// * `e` - The Soroban environment
    /// * `admin` - Address with full administrative control
    /// * `pauser` - Address authorized to pause/unpause the contract
    /// * `minter` - Address authorized to mint new tokens
    /// * `upgrader` - Address authorized to upgrade the contract
    /// * `blacklister` - Address authorized to block/unblock users and destroy funds
    ///
    /// # Security
    /// All roles are granted during initialization without requiring authorization,
    /// as this is the constructor and no previous state exists.
    pub fn __constructor(
        e: &Env,
        admin: Address,
        pauser: Address,
        minter: Address,
        upgrader: Address,
        blacklister: Address,
    ) {
        // Set token metadata: decimals=2, name="IDRX", symbol="IDRX"
        Base::set_metadata(
            e,
            2,
            String::from_str(e, "IDRX"),
            String::from_str(e, "IDRX"),
        );

        // Initialize access control with admin
        access_control::set_admin(e, &admin);

        // Grant roles to specified addresses
        access_control::grant_role_no_auth(e, &admin, &pauser, &Symbol::new(e, "pauser"));
        access_control::grant_role_no_auth(e, &admin, &minter, &Symbol::new(e, "minter"));
        access_control::grant_role_no_auth(e, &admin, &upgrader, &Symbol::new(e, "upgrader"));
        access_control::grant_role_no_auth(e, &admin, &blacklister, &Symbol::new(e, "blacklister"));
    }

    /// Mints new tokens to a specified account.
    ///
    /// Only addresses with the "minter" role can call this function.
    /// This function requires the contract to not be paused.
    ///
    /// # Arguments
    /// * `e` - The Soroban environment
    /// * `account` - Address to receive the minted tokens
    /// * `amount` - Amount of tokens to mint (must be non-negative)
    /// * `caller` - Address of the caller (must have minter role)
    ///
    /// # Events
    /// Emits a "mint" event with topics (event_name, minter) and data (account, amount)
    ///
    /// # Requirements
    /// - Caller must have the "minter" role
    /// - Contract must not be paused
    /// - Amount must be non-negative
    #[only_role(caller, "minter")]
    #[when_not_paused]
    pub fn mint(e: &Env, account: Address, amount: i128, caller: Address) {
        // Mint tokens to the specified account
        Base::mint(e, &account, amount);

        // Emit mint event with minter information in topics
        e.events().publish(
            (Symbol::new(e, "mint"), caller.clone()), // Topics: event name + minter address
            (account, amount),                        // Data: recipient address + amount
        );
    }

    /// Returns the contract's own address.
    ///
    /// This is useful for contracts that need to reference themselves
    /// or for external systems to verify contract identity.
    ///
    /// # Arguments
    /// * `e` - The Soroban environment
    ///
    /// # Returns
    /// The address of this contract instance
    pub fn get_contract_address(e: &Env) -> Address {
        e.current_contract_address()
    }

    /// Burns tokens from an account with an associated account number for tracking.
    ///
    /// The account number allows for external system tracking and auditing.
    /// The sender must authorize this operation (handled by Base::burn).
    /// This function requires the contract to not be paused.
    ///
    /// # Arguments
    /// * `e` - The Soroban environment
    /// * `from` - Address to burn tokens from (must authorize)
    /// * `amount` - Amount of tokens to burn (must be non-negative)
    /// * `account_number` - Account number string for external system tracking
    ///
    /// # Events
    /// Emits a "burn_with_account_number" event with data (from, amount, account_number)
    ///
    /// # Requirements
    /// - `from` must authorize the burn operation
    /// - Contract must not be paused
    /// - Amount must be non-negative
    #[when_not_paused]
    pub fn burn_with_account_number(e: &Env, from: Address, amount: i128, account_number: String) {
        // Validate amount is non-negative
        check_nonnegative_amount(amount);

        // Perform the burn (Base::burn handles authorization requirement)
        Base::burn(e, &from, amount);

        // Emit custom burn event with account number for tracking
        e.events().publish(
            (Symbol::new(e, "burn_with_account_number"),),
            (from, amount, account_number),
        );
    }

    /// Mints tokens through a bridge operation from another blockchain.
    ///
    /// This function is used when tokens are locked on another chain and need to be
    /// minted on this chain. Only addresses with the "minter" role can call this.
    /// This function requires the contract to not be paused.
    ///
    /// # Arguments
    /// * `e` - The Soroban environment
    /// * `from` - External address string on the source chain (e.g., "0x...")
    /// * `to` - Address to receive the minted tokens on this chain
    /// * `amount` - Amount of tokens to mint
    /// * `from_chain` - Chain ID of the source blockchain
    /// * `caller` - Address of the caller (must have minter role)
    ///
    /// # Events
    /// Emits a "mint_bridge" event with topics (event_name, minter) and
    /// data (from, to, amount, from_chain)
    ///
    /// # Requirements
    /// - Caller must have the "minter" role
    /// - Contract must not be paused
    #[only_role(caller, "minter")]
    #[when_not_paused]
    pub fn mint_bridge(
        e: &Env,
        from: String,
        to: Address,
        amount: i128,
        from_chain: i128,
        caller: Address,
    ) {
        // Mint tokens to the recipient address
        Base::mint(e, &to, amount);

        // Emit bridge mint event with source chain information
        e.events().publish(
            (Symbol::new(e, "mint_bridge"), caller.clone()), // Topics: event name + minter
            (from, to, amount, from_chain),                  // Data: source address, recipient, amount, chain ID
        );
    }

    /// Burns tokens through a bridge operation to unlock tokens on another blockchain.
    ///
    /// This function is used when tokens need to be burned on this chain to unlock
    /// them on another chain. The sender must authorize this operation.
    /// This function requires the contract to not be paused.
    ///
    /// # Arguments
    /// * `e` - The Soroban environment
    /// * `from` - Address to burn tokens from (must authorize)
    /// * `to` - External address string on the destination chain (e.g., "0x...")
    /// * `amount` - Amount of tokens to burn
    /// * `to_chain` - Chain ID of the destination blockchain
    /// * `caller` - Address of the caller (authorization context)
    ///
    /// # Events
    /// Emits a "burn_bridge" event with topics (event_name, caller) and
    /// data (from, to, amount, to_chain)
    ///
    /// # Requirements
    /// - `from` must authorize the burn operation
    /// - Contract must not be paused
    #[when_not_paused]
    pub fn burn_bridge(
        e: &Env,
        from: Address,
        to: String,
        amount: i128,
        to_chain: i128,
        caller: Address,
    ) {
        // Perform the burn (Base::burn handles authorization requirement)
        Base::burn(e, &from, amount);

        // Emit bridge burn event with destination chain information
        e.events().publish(
            (Symbol::new(e, "burn_bridge"), caller.clone()), // Topics: event name + caller
            (from, to, amount, to_chain),                    // Data: sender, destination address, amount, chain ID
        );
    }

    /// Destroys all funds belonging to a blacklisted user.
    ///
    /// This function permanently removes all tokens from a blacklisted address,
    /// effectively destroying them. Only addresses with the "blacklister" role can call this.
    /// This function requires the contract to not be paused.
    ///
    /// # Arguments
    /// * `e` - The Soroban environment
    /// * `blacklisted_user` - Address of the blacklisted user whose funds will be destroyed
    /// * `caller` - Address of the caller (must have blacklister role)
    ///
    /// # Events
    /// Emits a "destroy_blackfunds" event with topics (event_name, blacklister) and
    /// data (blacklisted_user, amount_destroyed)
    ///
    /// # Requirements
    /// - Caller must have the "blacklister" role
    /// - Contract must not be paused
    /// - User should be blacklisted (enforced by business logic)
    #[only_role(caller, "blacklister")]
    #[when_not_paused]
    pub fn destroy_blackfunds(e: &Env, blacklisted_user: Address, caller: Address) {
        // Get the balance of the blacklisted user
        let balance = Base::balance(e, &blacklisted_user);

        // Burn all tokens from the blacklisted user
        Base::burn(e, &blacklisted_user, balance);

        // Emit event for destroyed funds tracking
        e.events().publish(
            (Symbol::new(e, "destroy_blackfunds"), caller.clone()), // Topics: event name + blacklister
            (blacklisted_user.clone(), balance),                    // Data: blacklisted user + destroyed amount
        );
    }
}

// ============================================================================
// FungibleToken Implementation - Transfer Operations
// ============================================================================

/// Implementation of the FungibleToken trait with blocklist support.
///
/// This implementation provides standard ERC-20-like transfer functionality
/// while ensuring blocked addresses cannot send or receive tokens.
#[default_impl]
#[contractimpl]
impl FungibleToken for IDRX {
    type ContractType = BlockList;

    /// Transfers tokens from one address to another.
    ///
    /// The sender (`from`) must authorize this operation. Both sender and recipient
    /// must not be blocked. This function requires the contract to not be paused.
    ///
    /// # Arguments
    /// * `e` - The Soroban environment
    /// * `from` - Address to transfer tokens from (must authorize)
    /// * `to` - Address to transfer tokens to
    /// * `amount` - Amount of tokens to transfer (must be non-negative)
    ///
    /// # Events
    /// Emits a "transfer" event with data (from, to, amount)
    ///
    /// # Requirements
    /// - `from` must authorize the transfer
    /// - Both `from` and `to` must not be blocked
    /// - Contract must not be paused
    /// - Amount must be non-negative
    #[when_not_paused]
    fn transfer(e: &Env, from: Address, to: Address, amount: i128) {
        // Validate amount is non-negative
        check_nonnegative_amount(amount);

        // Perform transfer with blocklist checks
        Self::ContractType::transfer(e, &from, &to, amount);

        // Emit transfer event
        e.events()
            .publish((Symbol::new(e, "transfer"),), (from, to, amount));
    }

    /// Transfers tokens from one address to another using an allowance.
    ///
    /// The spender must have been approved by the owner to spend the specified amount.
    /// All parties (spender, owner, recipient) must not be blocked.
    /// This function requires the contract to not be paused.
    ///
    /// # Arguments
    /// * `e` - The Soroban environment
    /// * `spender` - Address authorized to spend tokens (must authorize)
    /// * `from` - Address to transfer tokens from (token owner)
    /// * `to` - Address to transfer tokens to
    /// * `amount` - Amount of tokens to transfer (must be non-negative)
    ///
    /// # Events
    /// Emits a "transfer_from" event with data (spender, from, to, amount)
    ///
    /// # Requirements
    /// - `spender` must have sufficient allowance from `from`
    /// - `spender` must authorize the operation
    /// - All parties must not be blocked
    /// - Contract must not be paused
    /// - Amount must be non-negative
    #[when_not_paused]
    fn transfer_from(e: &Env, spender: Address, from: Address, to: Address, amount: i128) {
        // Validate amount is non-negative
        check_nonnegative_amount(amount);

        // Perform transfer_from with blocklist checks
        Self::ContractType::transfer_from(e, &spender, &from, &to, amount);

        // Emit transfer_from event
        e.events().publish(
            (Symbol::new(e, "transfer_from"),),
            (spender, from, to, amount),
        );
    }
}

// ============================================================================
// FungibleBlockList Implementation - Blocklist Management
// ============================================================================

/// Implementation of blocklist functionality for the token contract.
///
/// Provides the ability to block and unblock addresses, preventing them
/// from sending or receiving tokens.
#[contractimpl]
impl FungibleBlockList for IDRX {
    /// Checks if an account is blocked.
    ///
    /// # Arguments
    /// * `e` - The Soroban environment
    /// * `account` - Address to check
    ///
    /// # Returns
    /// `true` if the account is blocked, `false` otherwise
    fn blocked(e: &Env, account: Address) -> bool {
        BlockList::blocked(e, &account)
    }

    /// Blocks a user address, preventing them from sending or receiving tokens.
    ///
    /// Only addresses with the "blacklister" role can call this function.
    ///
    /// # Arguments
    /// * `e` - The Soroban environment
    /// * `user` - Address to block
    /// * `operator` - Address of the caller (must have blacklister role)
    ///
    /// # Events
    /// Emits a "block_user" event with data (user, operator)
    ///
    /// # Requirements
    /// - `operator` must have the "blacklister" role
    #[only_role(operator, "blacklister")]
    fn block_user(e: &Env, user: Address, operator: Address) {
        // Block the user address
        BlockList::block_user(e, &user);

        // Emit block event for tracking
        e.events().publish(
            (Symbol::new(e, "block_user"),),
            (user, operator),
        );
    }

    /// Unblocks a user address, allowing them to send and receive tokens again.
    ///
    /// Only addresses with the "blacklister" role can call this function.
    ///
    /// # Arguments
    /// * `e` - The Soroban environment
    /// * `user` - Address to unblock
    /// * `operator` - Address of the caller (must have blacklister role)
    ///
    /// # Events
    /// Emits an "unblock_user" event with data (user, operator)
    ///
    /// # Requirements
    /// - `operator` must have the "blacklister" role
    #[only_role(operator, "blacklister")]
    fn unblock_user(e: &Env, user: Address, operator: Address) {
        // Unblock the user address
        BlockList::unblock_user(e, &user);

        // Emit unblock event for tracking
        e.events().publish(
            (Symbol::new(e, "unblock_user"),),
            (user, operator),
        );
    }
}

// ============================================================================
// FungibleBurnable Implementation - Burn Operations
// ============================================================================

/// Implementation of burnable token functionality.
///
/// Provides standard burn operations that allow users to permanently destroy
/// their own tokens or for authorized spenders to burn tokens from other accounts.
#[contractimpl]
impl FungibleBurnable for IDRX {
    /// Burns tokens from the caller's own account.
    ///
    /// The sender (`from`) must authorize this operation.
    /// This function requires the contract to not be paused.
    ///
    /// # Arguments
    /// * `e` - The Soroban environment
    /// * `from` - Address to burn tokens from (must authorize)
    /// * `amount` - Amount of tokens to burn
    ///
    /// # Events
    /// Emits a "burn" event with data (from, amount)
    ///
    /// # Requirements
    /// - `from` must authorize the burn
    /// - `from` must have sufficient balance
    /// - Contract must not be paused
    #[when_not_paused]
    fn burn(e: &Env, from: Address, amount: i128) {
        // Burn tokens from the specified address
        Base::burn(e, &from, amount);

        // Emit burn event
        e.events().publish(
            (Symbol::new(e, "burn"),),
            (from, amount),
        );
    }

    /// Burns tokens from another account using an allowance.
    ///
    /// The spender must have been approved by the owner to burn the specified amount.
    /// The spender must authorize this operation.
    /// This function requires the contract to not be paused.
    ///
    /// # Arguments
    /// * `e` - The Soroban environment
    /// * `spender` - Address authorized to burn tokens (must authorize)
    /// * `from` - Address to burn tokens from (token owner)
    /// * `amount` - Amount of tokens to burn
    ///
    /// # Events
    /// Emits a "burn_from" event with data (spender, from, amount)
    ///
    /// # Requirements
    /// - `spender` must have sufficient allowance from `from`
    /// - `spender` must authorize the operation
    /// - `from` must have sufficient balance
    /// - Contract must not be paused
    #[when_not_paused]
    fn burn_from(e: &Env, spender: Address, from: Address, amount: i128) {
        // Burn tokens from the owner's account using spender's allowance
        Base::burn_from(e, &spender, &from, amount);

        // Emit burn_from event
        e.events().publish(
            (Symbol::new(e, "burn_from"),),
            (spender, from, amount),
        );
    }
}

// ============================================================================
// Upgradeable Implementation - Contract Upgrade Functionality
// ============================================================================

/// Internal implementation for contract upgradeability.
///
/// Ensures that only addresses with the "upgrader" role can upgrade the contract.
impl UpgradeableInternal for IDRX {
    /// Requires authorization from an operator with the "upgrader" role.
    ///
    /// This function is called internally during upgrade operations to ensure
    /// proper authorization.
    ///
    /// # Arguments
    /// * `e` - The Soroban environment
    /// * `operator` - Address attempting to upgrade (must have upgrader role)
    ///
    /// # Panics
    /// Panics if the operator doesn't have the "upgrader" role or doesn't authorize
    fn _require_auth(e: &Env, operator: &Address) {
        // Ensure operator has the upgrader role
        access_control::ensure_role(e, operator, &Symbol::new(e, "upgrader"));

        // Require operator authorization
        operator.require_auth();
    }
}

// ============================================================================
// Pausable Implementation - Pause/Unpause Functionality
// ============================================================================

/// Implementation of pausable contract functionality.
///
/// Allows the contract to be paused and unpaused by authorized addresses,
/// which can halt certain operations during emergencies or maintenance.
#[contractimpl]
impl Pausable for IDRX {
    /// Checks if the contract is currently paused.
    ///
    /// # Arguments
    /// * `e` - The Soroban environment
    ///
    /// # Returns
    /// `true` if the contract is paused, `false` otherwise
    fn paused(e: &Env) -> bool {
        pausable::paused(e)
    }

    /// Pauses the contract, halting operations that require `when_not_paused`.
    ///
    /// Only addresses with the "pauser" role can call this function.
    ///
    /// # Arguments
    /// * `e` - The Soroban environment
    /// * `caller` - Address of the caller (must have pauser role)
    ///
    /// # Requirements
    /// - `caller` must have the "pauser" role
    #[only_role(caller, "pauser")]
    fn pause(e: &Env, caller: Address) {
        pausable::pause(e);
    }

    /// Unpauses the contract, resuming normal operations.
    ///
    /// Only addresses with the "pauser" role can call this function.
    ///
    /// # Arguments
    /// * `e` - The Soroban environment
    /// * `caller` - Address of the caller (must have pauser role)
    ///
    /// # Requirements
    /// - `caller` must have the "pauser" role
    #[only_role(caller, "pauser")]
    fn unpause(e: &Env, caller: Address) {
        pausable::unpause(e);
    }
}

// ============================================================================
// AccessControl Implementation - Role Management
// ============================================================================

/// Default implementation of access control functionality.
///
/// Provides role-based access control including:
/// - Role granting and revocation
/// - Role checking
/// - Admin role management
#[default_impl]
#[contractimpl]
impl AccessControl for IDRX {}
