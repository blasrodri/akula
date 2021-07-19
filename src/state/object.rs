use crate::models::Account;
use ethereum_types::H256;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct Object {
    pub initial: Option<Account>,
    pub current: Option<Account>,
}

#[derive(Debug)]
pub struct CommittedValue {
    /// value at the begining of the block
    pub initial: H256,
    // value at the begining of the transaction; see EIP-2200
    pub original: H256,
}

#[derive(Debug)]
pub struct Storage {
    pub committed: HashMap<H256, CommittedValue>,
    pub current: HashMap<H256, H256>,
}
