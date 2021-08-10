use super::{intra_block_state::IntraBlockState, object::Object};
use crate::StateBuffer;
use derive_more::Constructor;
use ethereum_types::Address;
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
