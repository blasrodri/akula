use crate::downloader::headers::header_slices::{HeaderSlice, HeaderSliceStatus, HeaderSlices};
use parking_lot::lock_api::RwLockUpgradableReadGuard;
use std::sync::Arc;

pub struct SaveStage {
    header_slices: Arc<HeaderSlices>,
}

impl SaveStage {
    pub fn new(header_slices: Arc<HeaderSlices>) -> Self {
        Self { header_slices }
    }

    pub async fn execute(&self) -> anyhow::Result<()> {
        for slice_lock in self.header_slices.iter() {
            let slice = slice_lock.upgradable_read();
            if slice.status == HeaderSliceStatus::Verified {
                // TODO: this await blocks while holding the read mutex
                self.save_slice(&slice).await?;

                let mut slice = RwLockUpgradableReadGuard::upgrade(slice);
                slice.status = HeaderSliceStatus::Saved;
            }
        }
        Ok(())
    }

    async fn save_slice(&self, _slice: &HeaderSlice) -> anyhow::Result<()> {
        // TODO: save verified headers to the DB
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        Ok(())
    }
}
