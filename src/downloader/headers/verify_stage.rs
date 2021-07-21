use crate::downloader::headers::header_slices::{HeaderSlice, HeaderSliceStatus, HeaderSlices};
use parking_lot::lock_api::RwLockUpgradableReadGuard;
use std::sync::Arc;

pub struct VerifyStage {
    header_slices: Arc<HeaderSlices>,
}

impl VerifyStage {
    pub fn new(header_slices: Arc<HeaderSlices>) -> Self {
        Self { header_slices }
    }

    pub fn execute(&self) -> anyhow::Result<()> {
        for slice_lock in self.header_slices.iter() {
            let slice = slice_lock.upgradable_read();
            if slice.status == HeaderSliceStatus::Downloaded {
                let is_verified = self.verify_slice(&slice);

                let mut slice = RwLockUpgradableReadGuard::upgrade(slice);
                if is_verified {
                    slice.status = HeaderSliceStatus::Verified;
                } else {
                    slice.status = HeaderSliceStatus::Empty;
                    slice.headers = None;
                    // TODO: penalize peer?
                }
            }
        }
        Ok(())
    }

    fn verify_slice(&self, _slice: &HeaderSlice) -> bool {
        // TODO: verify hashes properly
        rand::random::<u8>() < 224
    }
}
