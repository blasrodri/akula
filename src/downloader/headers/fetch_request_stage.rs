use crate::downloader::{
    block_id,
    headers::{
        header_slices,
        header_slices::{HeaderSliceStatus, HeaderSlices},
    },
    messages::{GetBlockHeadersMessage, GetBlockHeadersMessageParams, Message},
    sentry_client::PeerFilter,
    sentry_client_reactor::{SendMessageError, SentryClientReactor},
};
use parking_lot::{lock_api::RwLockUpgradableReadGuard, RwLock};
use std::{cell::Cell, sync::Arc};

pub struct FetchRequestStage {
    header_slices: Arc<HeaderSlices>,
    sentry: Arc<RwLock<SentryClientReactor>>,
    last_request_id: Cell<u64>,
}

impl FetchRequestStage {
    pub fn new(header_slices: Arc<HeaderSlices>, sentry: Arc<RwLock<SentryClientReactor>>) -> Self {
        Self {
            header_slices,
            sentry,
            last_request_id: Cell::new(0),
        }
    }

    pub async fn execute(&self) -> anyhow::Result<()> {
        self.request_pending()?;
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        Ok(())
    }

    fn request_pending(&self) -> anyhow::Result<()> {
        for slice_lock in self.header_slices.iter() {
            let slice = slice_lock.upgradable_read();
            match slice.status {
                HeaderSliceStatus::Empty | HeaderSliceStatus::Waiting => {
                    let mut request_id = self.last_request_id.get();
                    request_id += 1;
                    self.last_request_id.set(request_id);

                    let block_num = slice.start_block_num;
                    let limit = header_slices::HEADER_SLICE_SIZE as u64 + 1;

                    let result = self.request(request_id, block_num, limit);
                    match result {
                        Err(error) => match error.downcast_ref::<SendMessageError>() {
                            Some(SendMessageError::SendQueueFull) => break,
                            Some(SendMessageError::ReactorStopped) => return Err(error),
                            None => return Err(error),
                        },
                        Ok(_) => {
                            let mut slice = RwLockUpgradableReadGuard::upgrade(slice);
                            slice.status = HeaderSliceStatus::Waiting;
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn request(&self, request_id: u64, block_num: u64, limit: u64) -> anyhow::Result<()> {
        let message = GetBlockHeadersMessage {
            request_id,
            params: GetBlockHeadersMessageParams {
                start_block: block_id::BlockId::Number(block_num),
                limit,
                skip: 0,
                reverse: 0,
            },
        };
        self.sentry
            .read()
            .try_send_message(Message::GetBlockHeaders(message), PeerFilter::Random(1))
    }
}
