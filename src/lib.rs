extern crate alloc;

use stylus_sdk::{
    alloy_primitives::{Address, U256, Uint},
    prelude::*,
};
use alloy_sol_types::sol;

sol! {
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
}

sol_storage! {
    #[entrypoint]
    pub struct ERC20Token {
        mapping(address => uint256) balances;
        mapping(address => mapping(address => uint256)) allowances;
        string name;
        string symbol;
        uint8 decimals;
        uint256 total_supply;
    }
}

#[public]
impl ERC20Token {
    pub fn init(&mut self, name: String, symbol: String, decimals: u8, initial_supply: U256) {
        let sender = self.vm().msg_sender();
        
        self.name.set_str(name);
        self.symbol.set_str(symbol);
        self.decimals.set(Uint::from(decimals));
        self.total_supply.set(initial_supply);
        
        self.balances.setter(sender).set(initial_supply);
        
        log(self.vm(), Transfer {
            from: Address::ZERO,
            to: sender,
            value: initial_supply,
        });
    }

    pub fn name(&self) -> String {
        self.name.get_string()
    }

    pub fn symbol(&self) -> String {
        self.symbol.get_string()
    }

    pub fn decimals(&self) -> u8 {
        self.decimals.get().to::<u8>()
    }

    pub fn total_supply(&self) -> U256 {
        self.total_supply.get()
    }

    pub fn balance_of(&self, account: Address) -> U256 {
        self.balances.get(account)
    }

    pub fn transfer(&mut self, to: Address, amount: U256) -> bool {
        let sender = self.vm().msg_sender();
        self._transfer(sender, to, amount);
        true
    }

    pub fn approve(&mut self, spender: Address, amount: U256) -> bool {
        let owner = self.vm().msg_sender();
        self.allowances.setter(owner).setter(spender).set(amount);
        
        log(self.vm(), Approval {
            owner,
            spender,
            value: amount,
        });
        
        true
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.allowances.getter(owner).get(spender)
    }

    pub fn transfer_from(&mut self, from: Address, to: Address, amount: U256) -> bool {
        let spender = self.vm().msg_sender();
        
        let current_allowance = self.allowances.getter(from).get(spender);
        assert!(current_allowance >= amount, "Insufficient allowance");

        let new_allowance = current_allowance - amount;
        self.allowances.setter(from).setter(spender).set(new_allowance);
        
        log(self.vm(), Approval {
            owner: from,
            spender,
            value: new_allowance,
        });

        self._transfer(from, to, amount);
        true
    }

    pub fn mint(&mut self, to: Address, amount: U256) -> bool {
        let new_total_supply = self.total_supply.get() + amount;
        self.total_supply.set(new_total_supply);

        let current_balance = self.balances.get(to);
        let new_balance = current_balance + amount;
        self.balances.setter(to).set(new_balance);

        log(self.vm(), Transfer {
            from: Address::ZERO,
            to,
            value: amount,
        });

        true
    }
}


impl ERC20Token {
    fn _transfer(&mut self, from: Address, to: Address, amount: U256) {
        let from_balance = self.balances.get(from);
        assert!(from_balance >= amount, "Insufficient balance");

        let new_from_balance = from_balance - amount;
        self.balances.setter(from).set(new_from_balance);

        let to_balance = self.balances.get(to);
        let new_to_balance = to_balance + amount;
        self.balances.setter(to).set(new_to_balance);

        log(self.vm(), Transfer {
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
    fn test_transfer() {
        let mut contract = unsafe { ERC20Token::new(U256::ZERO, 0) };
        let from = Address::from([1u8; 20]);
        let to = Address::from([2u8; 20]);
        let amount = U256::from(100);
        
        contract.balances.setter(from).set(U256::from(500));
        
        contract._transfer(from, to, amount);
        
        assert_eq!(contract.balance_of(from), U256::from(400));
        assert_eq!(contract.balance_of(to), amount);
    }

    #[test]
    #[should_panic(expected = "Insufficient balance")]
    fn test_transfer_insufficient_balance() {
        let mut contract = unsafe { ERC20Token::new(U256::ZERO, 0) };
        let from = Address::from([1u8; 20]);
        let to = Address::from([2u8; 20]);
        
        contract.balances.setter(from).set(U256::from(50));
        contract._transfer(from, to, U256::from(100));
    }

    #[test]
    fn test_allowance() {
        let mut contract = unsafe { ERC20Token::new(U256::ZERO, 0) };
        let owner = Address::from([1u8; 20]);
        let spender = Address::from([2u8; 20]);
        let amount = U256::from(100);

        contract.allowances.setter(owner).setter(spender).set(amount);
        
        assert_eq!(contract.allowance(owner, spender), amount);
    }
}
