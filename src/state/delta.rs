use super::{intra_block_state::IntraBlockState, object::Object};
use crate::{StateBuffer, Storage};
use derive_more::Constructor;
use ethereum_types::{Address, H256};
use std::fmt::Debug;

/// Reversible change made to `IntraBlockState`.
#[derive(Debug)]
pub enum Delta {
    Create {
        address: Address,
    },
    Update {
        address: Address,
        previous: Object,
    },
    Selfdestruct {
        address: Address,
    },
    Touch {
        address: Address,
    },
    StorageChange {
        address: Address,
        key: H256,
        previous: H256,
    },
    StorageWipe {
        address: Address,
        storage: Storage,
    },
    StorageCreate {
        address: Address,
    },

    StorageAccess {
        address: Address,
        key: H256,
    },
    AccountAccess {
        address: Address,
    },
}

impl Delta {
    pub fn revert<'storage, 'r, R>(self, state: &mut IntraBlockState<'storage, 'r, R>)
    where
        R: StateBuffer<'storage>,
    {
        match self {
            Delta::Create { address } => {
                state.objects.remove(&address);
            }
            Delta::Update { address, previous } => {
                state.objects.insert(address, previous);
            }
            Delta::Selfdestruct { address } => {
                state.self_destructs.remove(&address);
            }
            Delta::Touch { address } => {
                state.touched.remove(&address);
            }
            Delta::StorageChange {
                address,
                key,
                previous,
            } => {
                state
                    .storage
                    .get_mut(&address)
                    .unwrap()
                    .current
                    .insert(key, previous);
            }
            Delta::StorageWipe { address, storage } => {
                state.storage.insert(address, storage);
            }
            Delta::StorageCreate { address } => {
                state.storage.remove(&address);
            }
            Delta::StorageAccess { address, key } => {
                state
                    .accessed_storage_keys
                    .get_mut(&address)
                    .unwrap()
                    .remove(&key);
            }
            Delta::AccountAccess { address } => {
                state.accessed_addresses.remove(&address);
            }
        }
    }
}
