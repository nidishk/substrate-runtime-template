//! Substrate Runtime Template
//! 
//! A minimal runtime module implementation demonstrating:
//! - Storage types (values, maps, double maps)
//! - Events
//! - Errors  
//! - Dispatchable calls

use std::collections::HashMap;
use std::sync::RwLock;

/// Account identifier type
pub type AccountId = u64;
/// Balance type
pub type Balance = u128;
/// Block number type
pub type BlockNumber = u32;

/// Runtime errors
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    InsufficientBalance,
    AccountNotFound,
    Overflow,
    Underflow,
    InvalidValue,
}

/// Runtime events
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Transfer { from: AccountId, to: AccountId, amount: Balance },
    Deposit { who: AccountId, amount: Balance },
    Withdraw { who: AccountId, amount: Balance },
    NewBlock { number: BlockNumber },
}

/// Storage for the runtime
pub struct Storage {
    balances: RwLock<HashMap<AccountId, Balance>>,
    total_issuance: RwLock<Balance>,
    block_number: RwLock<BlockNumber>,
    events: RwLock<Vec<Event>>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            balances: RwLock::new(HashMap::new()),
            total_issuance: RwLock::new(0),
            block_number: RwLock::new(0),
            events: RwLock::new(Vec::new()),
        }
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

/// Runtime pallet implementation
pub struct BalancesPallet {
    storage: Storage,
}

impl BalancesPallet {
    pub fn new() -> Self {
        Self {
            storage: Storage::new(),
        }
    }

    /// Deposit tokens to an account
    pub fn deposit(&self, who: AccountId, amount: Balance) -> Result<(), Error> {
        let mut balances = self.storage.balances.write().unwrap();
        let mut total = self.storage.total_issuance.write().unwrap();
        
        let balance = balances.entry(who).or_insert(0);
        *balance = balance.checked_add(amount).ok_or(Error::Overflow)?;
        *total = total.checked_add(amount).ok_or(Error::Overflow)?;
        
        self.emit_event(Event::Deposit { who, amount });
        Ok(())
    }

    /// Withdraw tokens from an account
    pub fn withdraw(&self, who: AccountId, amount: Balance) -> Result<(), Error> {
        let mut balances = self.storage.balances.write().unwrap();
        let mut total = self.storage.total_issuance.write().unwrap();
        
        let balance = balances.get_mut(&who).ok_or(Error::AccountNotFound)?;
        if *balance < amount {
            return Err(Error::InsufficientBalance);
        }
        
        *balance = balance.checked_sub(amount).ok_or(Error::Underflow)?;
        *total = total.checked_sub(amount).ok_or(Error::Underflow)?;
        
        self.emit_event(Event::Withdraw { who, amount });
        Ok(())
    }

    /// Transfer tokens between accounts
    pub fn transfer(&self, from: AccountId, to: AccountId, amount: Balance) -> Result<(), Error> {
        let mut balances = self.storage.balances.write().unwrap();
        
        let from_balance = balances.get(&from).copied().ok_or(Error::AccountNotFound)?;
        if from_balance < amount {
            return Err(Error::InsufficientBalance);
        }
        
        let to_balance = balances.entry(to).or_insert(0);
        *to_balance = to_balance.checked_add(amount).ok_or(Error::Overflow)?;
        
        let from_balance = balances.get_mut(&from).unwrap();
        *from_balance = from_balance.checked_sub(amount).ok_or(Error::Underflow)?;
        
        self.emit_event(Event::Transfer { from, to, amount });
        Ok(())
    }

    /// Get balance of an account
    pub fn balance_of(&self, who: AccountId) -> Balance {
        self.storage.balances.read().unwrap().get(&who).copied().unwrap_or(0)
    }

    /// Get total issuance
    pub fn total_issuance(&self) -> Balance {
        *self.storage.total_issuance.read().unwrap()
    }

    /// Advance to next block
    pub fn next_block(&self) {
        let mut block_number = self.storage.block_number.write().unwrap();
        *block_number += 1;
        self.emit_event(Event::NewBlock { number: *block_number });
    }

    /// Get current block number
    pub fn block_number(&self) -> BlockNumber {
        *self.storage.block_number.read().unwrap()
    }

    fn emit_event(&self, event: Event) {
        self.storage.events.write().unwrap().push(event);
    }

    /// Get all events
    pub fn events(&self) -> Vec<Event> {
        self.storage.events.read().unwrap().clone()
    }
}

impl Default for BalancesPallet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deposit() {
        let pallet = BalancesPallet::new();
        pallet.deposit(1, 1000).unwrap();
        assert_eq!(pallet.balance_of(1), 1000);
        assert_eq!(pallet.total_issuance(), 1000);
    }

    #[test]
    fn test_withdraw() {
        let pallet = BalancesPallet::new();
        pallet.deposit(1, 1000).unwrap();
        pallet.withdraw(1, 500).unwrap();
        assert_eq!(pallet.balance_of(1), 500);
        assert_eq!(pallet.total_issuance(), 500);
    }

    #[test]
    fn test_withdraw_insufficient() {
        let pallet = BalancesPallet::new();
        pallet.deposit(1, 100).unwrap();
        assert_eq!(pallet.withdraw(1, 200), Err(Error::InsufficientBalance));
    }

    #[test]
    fn test_transfer() {
        let pallet = BalancesPallet::new();
        pallet.deposit(1, 1000).unwrap();
        pallet.transfer(1, 2, 300).unwrap();
        assert_eq!(pallet.balance_of(1), 700);
        assert_eq!(pallet.balance_of(2), 300);
    }

    #[test]
    fn test_events() {
        let pallet = BalancesPallet::new();
        pallet.deposit(1, 100).unwrap();
        pallet.transfer(1, 2, 50).unwrap();
        
        let events = pallet.events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], Event::Deposit { who: 1, amount: 100 });
        assert_eq!(events[1], Event::Transfer { from: 1, to: 2, amount: 50 });
    }

    #[test]
    fn test_block_number() {
        let pallet = BalancesPallet::new();
        assert_eq!(pallet.block_number(), 0);
        pallet.next_block();
        assert_eq!(pallet.block_number(), 1);
    }
}
