use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use stylus_sdk::{evm, msg, prelude::*};

// Events using sol! macro
sol! {
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
}

// Storage using sol_storage! with Solidity-like syntax
sol_storage! {
    #[entrypoint]
    pub struct ERC20Token {
        mapping(address => uint256) balances;
        mapping(address => mapping(address => uint256)) allowances;
        uint256 total_supply;
        string name;
        string symbol;
        uint8 decimals;
    }
}

#[public]
impl ERC20Token {
    /// Initialize the token
    pub fn init(&mut self, name: String, symbol: String, decimals: u8, initial_supply: U256) {
        let sender = msg::sender();
        
        self.name.set_str(name);
        self.symbol.set_str(symbol);
        self.decimals.set(decimals);
        self.total_supply.set(initial_supply);
        
        // Give initial supply to deployer
        self.balances.setter(sender).set(initial_supply);
        
        evm::log(Transfer {
            from: Address::ZERO,
            to: sender,
            value: initial_supply,
        });
    }

    /// Returns the name of the token
    pub fn name(&self) -> String {
        self.name.get_string()
    }

    /// Returns the symbol of the token
    pub fn symbol(&self) -> String {
        self.symbol.get_string()
    }

    /// Returns the number of decimals
    pub fn decimals(&self) -> u8 {
        self.decimals.get()
    }

    /// Returns the total supply
    pub fn total_supply(&self) -> U256 {
        self.total_supply.get()
    }

    /// Returns the balance of an account
    pub fn balance_of(&self, account: Address) -> U256 {
        self.balances.get(account)
    }

    /// Transfer tokens to another address
    pub fn transfer(&mut self, to: Address, amount: U256) -> bool {
        let sender = msg::sender();
        self._transfer(sender, to, amount);
        true
    }

    /// Approve spender to spend tokens on behalf of the caller
    pub fn approve(&mut self, spender: Address, amount: U256) -> bool {
        let owner = msg::sender();
        self.allowances.setter(owner).setter(spender).set(amount);
        
        evm::log(Approval {
            owner,
            spender,
            value: amount,
        });
        
        true
    }

    /// Returns the allowance of spender for owner's tokens
    pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.allowances.getter(owner).get(spender)
    }

    /// Transfer tokens from one address to another using allowance
    pub fn transfer_from(&mut self, from: Address, to: Address, amount: U256) -> bool {
        let spender = msg::sender();
        
        // Check allowance
        let current_allowance = self.allowances.getter(from).get(spender);
        assert!(current_allowance >= amount, "Insufficient allowance");

        // Update allowance
        let new_allowance = current_allowance - amount;
        self.allowances.setter(from).setter(spender).set(new_allowance);
        
        evm::log(Approval {
            owner: from,
            spender,
            value: new_allowance,
        });

        // Transfer tokens
        self._transfer(from, to, amount);
        true
    }

    /// Optional: Mint function (simple implementation)
    pub fn mint(&mut self, to: Address, amount: U256) -> bool {
        // Update total supply
        let new_total_supply = self.total_supply.get() + amount;
        self.total_supply.set(new_total_supply);

        // Update balance
        let current_balance = self.balances.get(to);
        let new_balance = current_balance + amount;
        self.balances.setter(to).set(new_balance);

        evm::log(Transfer {
            from: Address::ZERO,
            to,
            value: amount,
        });

        true
    }
}

impl ERC20Token {
    /// Internal transfer function
    fn _transfer(&mut self, from: Address, to: Address, amount: U256) {
        let from_balance = self.balances.get(from);
        assert!(from_balance >= amount, "Insufficient balance");

        // Update balances
        let new_from_balance = from_balance - amount;
        self.balances.setter(from).set(new_from_balance);

        let to_balance = self.balances.get(to);
        let new_to_balance = to_balance + amount;
        self.balances.setter(to).set(new_to_balance);

        evm::log(Transfer {
            from,
            to,
            value: amount,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let mut contract = ERC20Token::default();
        let initial_supply = U256::from(1000000);
        
        contract.init("Test Token".to_string(), "TEST".to_string(), 18, initial_supply);

        assert_eq!(contract.name(), "Test Token");
        assert_eq!(contract.symbol(), "TEST");
        assert_eq!(contract.decimals(), 18);
        assert_eq!(contract.total_supply(), initial_supply);
    }

    #[test]
    fn test_transfer() {
        let mut contract = ERC20Token::default();
        let from = Address::from([1u8; 20]);
        let to = Address::from([2u8; 20]);
        let amount = U256::from(100);
        
        // Set up balance
        contract.balances.setter(from).set(U256::from(500));
        
        contract._transfer(from, to, amount);
        
        assert_eq!(contract.balance_of(from), U256::from(400));
        assert_eq!(contract.balance_of(to), amount);
    }

    #[test]
    #[should_panic(expected = "Insufficient balance")]
    fn test_transfer_insufficient_balance() {
        let mut contract = ERC20Token::default();
        let from = Address::from([1u8; 20]);
        let to = Address::from([2u8; 20]);
        
        contract.balances.setter(from).set(U256::from(50));
        contract._transfer(from, to, U256::from(100));
    }

    #[test]
    fn test_allowance() {
        let mut contract = ERC20Token::default();
        let owner = Address::from([1u8; 20]);
        let spender = Address::from([2u8; 20]);
        let amount = U256::from(100);

        contract.allowances.setter(owner).setter(spender).set(amount);
        
        assert_eq!(contract.allowance(owner, spender), amount);
    }
}
