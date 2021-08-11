use super::{intra_block_state::IntraBlockState, object::Object};
use crate::{StateBuffer, Storage};
use derive_more::Constructor;
use ethereum_types::{Address, H256};
use std::fmt::Debug;

/// Delta is a reversible change made to IntraBlockState.
pub trait Delta<'storage, 'r, R: StateBuffer<'storage>>: Debug + Send + Sync {
    fn revert(self, _: &mut IntraBlockState<'storage, 'r, R>);
}

#[derive(Constructor, Debug)]
pub struct CreateDelta {
    address: Address,
}

impl<'storage, 'r, R: StateBuffer<'storage>> Delta<'storage, 'r, R> for CreateDelta {
    fn revert(self, state: &mut IntraBlockState<'storage, 'r, R>) {
        state.objects.remove(&self.address);
    }
}

#[derive(Constructor, Debug)]
pub struct UpdateDelta {
    address: Address,
    previous: Object,
}

impl<'storage, 'r, R: StateBuffer<'storage>> Delta<'storage, 'r, R> for UpdateDelta {
    fn revert(self, state: &mut IntraBlockState<'storage, 'r, R>) {
        state.objects.insert(self.address, self.previous);
    }
}

#[derive(Constructor, Debug)]
pub struct SelfdestructDelta {
    address: Address,
}

impl<'storage, 'r, R: StateBuffer<'storage>> Delta<'storage, 'r, R> for SelfdestructDelta {
    fn revert(self, state: &mut IntraBlockState<'storage, 'r, R>) {
        state.self_destructs.remove(&self.address);
    }
}

#[derive(Constructor, Debug)]
pub struct TouchDelta {
    address: Address,
}

impl<'storage, 'r, R: StateBuffer<'storage>> Delta<'storage, 'r, R> for TouchDelta {
    fn revert(self, state: &mut IntraBlockState<'storage, 'r, R>) {
        state.touched.remove(&self.address);
    }
}

#[derive(Constructor, Debug)]
pub struct StorageChangeDelta {
    address: Address,
    key: H256,
    previous: H256,
}

impl<'storage, 'r, R: StateBuffer<'storage>> Delta<'storage, 'r, R> for StorageChangeDelta {
    fn revert(self, state: &mut IntraBlockState<'storage, 'r, R>) {
        state
            .storage
            .get_mut(&self.address)
            .unwrap()
            .current
            .insert(self.key, self.previous);
    }
}

#[derive(Constructor, Debug)]
pub struct StorageWipeDelta {
    address: Address,
    storage: Storage,
}

impl<'storage, 'r, R: StateBuffer<'storage>> Delta<'storage, 'r, R> for StorageWipeDelta {
    fn revert(self, state: &mut IntraBlockState<'storage, 'r, R>) {
        state.storage.insert(self.address, self.storage);
    }
}

#[derive(Constructor, Debug)]
pub struct StorageCreateDelta {
    address: Address,
}

impl<'storage, 'r, R: StateBuffer<'storage>> Delta<'storage, 'r, R> for StorageCreateDelta {
    fn revert(self, state: &mut IntraBlockState<'storage, 'r, R>) {
        state.storage.remove(&self.address);
    }
}

#[derive(Constructor, Debug)]
pub struct StorageAccessDelta {
    address: Address,
    key: H256,
}

impl<'storage, 'r, R: StateBuffer<'storage>> Delta<'storage, 'r, R> for StorageAccessDelta {
    fn revert(self, state: &mut IntraBlockState<'storage, 'r, R>) {
        state
            .accessed_storage_keys
            .get_mut(&self.address)
            .unwrap()
            .remove(&self.key);
    }
}

#[derive(Constructor, Debug)]
pub struct AccountAccessDelta {
    address: Address,
}

impl<'storage, 'r, R: StateBuffer<'storage>> Delta<'storage, 'r, R> for AccountAccessDelta {
    fn revert(self, state: &mut IntraBlockState<'storage, 'r, R>) {
        state.accessed_addresses.remove(&self.address);
    }
}
