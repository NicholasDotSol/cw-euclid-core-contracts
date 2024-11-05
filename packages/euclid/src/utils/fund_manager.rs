use std::collections::HashMap;

use cosmwasm_std::{ensure, Coin, Uint128};

use crate::error::ContractError;

/// A utility struct for managing and validating funds in a transaction
pub struct FundManager {
    /// Map of denomination to amount of funds
    funds: HashMap<String, Uint128>,
}

impl FundManager {
    /// Creates a new FundManager instance from a slice of Coins
    ///
    /// # Arguments
    /// * `funds` - Slice of Coins to initialize the fund manager with
    ///
    /// # Returns
    /// A new FundManager instance with the provided funds
    pub fn new(funds: &[Coin]) -> Self {
        let mut fund_manager = FundManager {
            funds: HashMap::new(),
        };
        for fund in funds {
            fund_manager.add(fund);
        }
        fund_manager
    }

    /// Gets the amount of funds for a given denomination
    ///
    /// # Arguments
    /// * `denom` - The denomination to get the amount for
    ///
    /// # Returns
    /// The amount of funds for the denomination, or zero if not found
    pub fn get(&self, denom: &str) -> Uint128 {
        self.funds.get(denom).cloned().unwrap_or(Uint128::zero())
    }

    /// Adds funds to the manager
    ///
    /// # Arguments
    /// * `fund` - The Coin to add to the manager
    pub fn add(&mut self, fund: &Coin) {
        *self
            .funds
            .entry(fund.denom.to_string())
            .or_insert(Uint128::zero()) += fund.amount;
    }

    /// Uses (deducts) funds from the manager
    ///
    /// # Arguments
    /// * `amount` - The amount to deduct
    /// * `denom` - The denomination to deduct from
    ///
    /// # Returns
    /// * `Ok(())` if the funds were successfully deducted
    /// * `Err(ContractError)` if:
    ///   - The amount is zero
    ///   - There are insufficient funds
    pub fn use_fund(&mut self, amount: Uint128, denom: &str) -> Result<(), ContractError> {
        ensure!(
            !amount.is_zero(),
            ContractError::new("Amount cannot be zero")
        );
        ensure!(
            self.get(denom).ge(&amount),
            ContractError::InsufficientFunds {}
        );
        *self.funds.get_mut(denom).unwrap() -= amount;
        Ok(())
    }

    /// Validates that all fund amounts are non-zero
    ///
    /// # Returns
    /// * `Ok(())` if all fund amounts are non-zero
    /// * `Err(ContractError)` if any fund amount is zero
    pub fn validate_non_zero_funds(&self) -> Result<(), ContractError> {
        ensure!(
            self.funds.iter().all(|(_, amount)| !amount.is_zero()),
            ContractError::new("Funds cannot be zero")
        );
        Ok(())
    }

    /// Validates that all fund amounts are zero
    ///
    /// This should be called after all fund operations are complete to ensure
    /// no funds remain unaccounted for.
    ///
    /// # Returns
    /// * `Ok(())` if all fund amounts are zero
    /// * `Err(ContractError)` if any fund amount is non-zero
    pub fn validate_funds_are_empty(&self) -> Result<(), ContractError> {
        ensure!(
            self.funds.iter().all(|(_, amount)| amount.is_zero()),
            ContractError::new("Funds should be empty")
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{Coin, Uint128};

    use crate::error::ContractError;

    use super::*;

    #[test]
    fn test_new() {
        let fund_manager = FundManager::new(&[Coin::new(100, "atom")]);
        assert_eq!(fund_manager.get("atom"), Uint128::new(100));
    }

    #[test]
    fn test_duplicate_funds() {
        let fund_manager = FundManager::new(&[Coin::new(100, "atom"), Coin::new(200, "atom")]);
        assert_eq!(fund_manager.get("atom"), Uint128::new(300));
    }

    #[test]
    fn test_use_fund() {
        let mut fund_manager = FundManager::new(&[Coin::new(100, "atom")]);
        assert_eq!(fund_manager.use_fund(Uint128::new(50), "atom"), Ok(()));
        assert_eq!(fund_manager.get("atom"), Uint128::new(50));
    }

    #[test]
    fn test_use_fund_insufficient() {
        let mut fund_manager = FundManager::new(&[Coin::new(100, "atom")]);
        assert_eq!(
            fund_manager.use_fund(Uint128::new(150), "atom"),
            Err(ContractError::InsufficientFunds {})
        );
    }

    #[test]
    fn test_validate_non_zero_funds() {
        let fund_manager = FundManager::new(&[Coin::new(100, "atom")]);
        assert_eq!(fund_manager.validate_non_zero_funds(), Ok(()));
    }

    #[test]
    fn test_validate_non_zero_funds_empty() {
        let fund_manager = FundManager::new(&[Coin::new(0, "atom")]);
        assert_eq!(
            fund_manager.validate_non_zero_funds(),
            Err(ContractError::new("Funds cannot be zero"))
        );
    }

    #[test]
    fn test_validate_funds_are_empty() {
        let fund_manager = FundManager::new(&[]);
        assert_eq!(fund_manager.validate_funds_are_empty(), Ok(()));
    }

    #[test]
    fn test_funds_are_not_empty() {
        let fund_manager = FundManager::new(&[Coin::new(100, "atom")]);
        assert_eq!(
            fund_manager.validate_funds_are_empty(),
            Err(ContractError::new("Funds should be empty"))
        );
    }

    #[test]
    fn test_validate_funds_are_empty_after_use() {
        let mut fund_manager = FundManager::new(&[Coin::new(100, "atom")]);
        fund_manager.use_fund(Uint128::new(100), "atom").unwrap();
        assert_eq!(fund_manager.validate_funds_are_empty(), Ok(()));
    }

    #[test]
    fn test_insufficient_funds() {
        let mut fund_manager = FundManager::new(&[Coin::new(100, "atom")]);
        assert_eq!(
            fund_manager.use_fund(Uint128::new(150), "atom"),
            Err(ContractError::InsufficientFunds {})
        );
    }
}
