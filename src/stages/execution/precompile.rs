use ethereum_types::Address;
use evmodin::{Message, Output};
use std::collections::HashMap;

pub type Precompile = fn(Message) -> Output;

#[derive(Clone, Debug, Default)]
pub struct PrecompileSet {
    set: HashMap<Address, Precompile>,
}

impl PrecompileSet {
    pub fn get_precompile(&self, addr: Address) -> Option<Precompile> {
        None
    }
}
