use core::marker::PhantomData;

use stylus_sdk::{
    alloy_primitives::{Address, U256},
    alloy_sol_types::sol,
    evm, msg,
    stylus_proc::{external, sol_storage},
};

pub trait SERC721Details {
    const NAME: &'static str;
    const SYMBOL: &'static str;
}

sol_storage! {
    pub struct SERC721<T>{
        mapping (uint256 => address) _owner;
        mapping (address=>uint256) _balance_of;
        mapping (uint256=>address) _token_approval;
        mapping (address=> mapping(address=>bool)) _operator_approval;
        PhantomData<T> phantom_data;
    }
}

sol! {
    event Transfer(address indexed from, address indexed to, uint256 indexed id);
    event Approval(address indexed owner, address indexed spender, uint256 indexed id);
    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);

    error NonexistentToken(uint256 id);
    error NotOwner();
    error NotAuthorized();
    error InvalidRecipient();
    error NotApprovedForAll();

}

pub enum SERC721Error {
    NotOwner,
    NotAuthorized,
    InvalidRecipient,
    NotApprovedForAll,
    NonexistentToken(U256),
    NotMinted,
}

impl From<SERC721Error> for Vec<u8> {
    fn from(value: SERC721Error) -> Vec<u8> {
        match value {
            SERC721Error::NotOwner => "SERC721: Caller is not an owner".to_string().into_bytes(),
            SERC721Error::NotMinted => b"SERC721: Not minted ".to_vec(),
            SERC721Error::NotAuthorized => b"SERC721: Not Authorised".to_vec(),
            SERC721Error::NotApprovedForAll => {
                b"SERC721: Transfer caller is not an owner or is not approved".to_vec()
            }
            SERC721Error::InvalidRecipient => b"SERC721: Invalid receipient".to_vec(),
            SERC721Error::NonexistentToken(e) => format!("SERC721: Token {} doesnt exits", e)
                .to_string()
                .into_bytes(),
        }
    }
}

// internal methods
impl<T> SERC721<T>
where
    T: SERC721Details,
{
    pub fn _mint(&mut self, to: Address, id: U256) -> Result<(), SERC721Error> {
        if self._owner.get(id) != Address::ZERO {
            return Err(SERC721Error::InvalidRecipient.into());
        }
        let old_balance = self._balance_of.get(to);
        let mut to_balance_setter = self._balance_of.setter(to);
        to_balance_setter.set(old_balance + U256::from(1));

        self._owner.setter(id).set(to);
        evm::log(Transfer {
            from: Address::ZERO,
            to,
            id,
        });
        Ok(())
    }
}

// external methods

#[external]
impl<T> SERC721<T>
where
    T: SERC721Details,
{
    pub fn name(&self) -> Result<String, SERC721Error> {
        Ok(T::NAME.into())
    }

    pub fn symbol(&self) -> Result<String, SERC721Error> {
        Ok(T::SYMBOL.into())
    }

    pub fn owner_of(&self, id: U256) -> Result<Address, SERC721Error> {
        if self._owner.get(id) == Address::ZERO {
            return Err(SERC721Error::NonexistentToken(id));
        }
        Ok(self._owner.get(id))
    }

    pub fn balance_of(&self, owner: Address) -> Result<U256, SERC721Error> {
        Ok(self._balance_of.get(owner))
    }

    pub fn get_approved(&self, id: U256) -> Result<Address, SERC721Error> {
        Ok(self._token_approval.get(id))
    }

    pub fn is_approved_for_all(
        &self,
        owner: Address,
        operator: Address,
    ) -> Result<bool, SERC721Error> {
        Ok(self._operator_approval.get(owner).get(operator))
    }

    pub fn approve(&mut self, spender: Address, id: U256) -> Result<(), SERC721Error> {
        let owner = self._owner.get(id);

        if owner != msg::sender() || self.is_approved_for_all(owner, spender)? {
            return Err(SERC721Error::NotOwner);
        }

        let mut approve = self._token_approval.setter(id);
        approve.set(spender);

        evm::log(Approval { owner, spender, id });
        Ok(())
    }

    pub fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), SERC721Error> {
        let caller = msg::sender();
        self._operator_approval
            .setter(caller)
            .setter(operator)
            .set(approved);

        evm::log(ApprovalForAll {
            owner: caller,
            operator,
            approved,
        });
        Ok(())
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
    ) -> Result<(), SERC721Error> {
        if self._owner.get(id) != from {
            return Err(SERC721Error::NotOwner.into());
        }

        if to == Address::ZERO {
            return Err(SERC721Error::InvalidRecipient.into());
        }

        let caller = msg::sender();
        if caller != from
            && !self.is_approved_for_all(from, caller)?
            && self.get_approved(id)? != caller
        {
            return Err(SERC721Error::NotAuthorized.into());
        }

        let old_balance_from = self._balance_of.get(from);
        let mut from_balance_setter = self._balance_of.setter(from);
        from_balance_setter.set(old_balance_from - U256::from(1));

        let old_balance_to = self._balance_of.get(to);
        let mut to_balance_setter = self._balance_of.setter(to);
        to_balance_setter.set(old_balance_to + U256::from(1));

        let mut owner_of_setter = self._owner.setter(id);
        owner_of_setter.set(to);

        let mut approved_setter = self._token_approval.setter(id);
        approved_setter.set(Address::ZERO);
        evm::log(Transfer { from, to, id });
        Ok(())
    }
}
