use crate::downloader::{
    headers::{
        header_slices,
        header_slices::{HeaderSliceStatus, HeaderSlices},
    },
    messages::{BlockHeadersMessage, EthMessageId, Message},
    sentry_client_reactor::SentryClientReactor,
};
use ethereum::Header;
use futures_core::Stream;
use parking_lot::RwLock;
use std::{cell::Cell, pin::Pin, sync::Arc};
use tokio_stream::StreamExt;

pub struct FetchReceiveStage {
    header_slices: Arc<HeaderSlices>,
    sentry: Arc<RwLock<SentryClientReactor>>,
    is_over: Cell<bool>,
}

impl FetchReceiveStage {
    pub fn new(header_slices: Arc<HeaderSlices>, sentry: Arc<RwLock<SentryClientReactor>>) -> Self {
        Self {
            header_slices,
            sentry,
            is_over: Cell::new(false),
        }
    }

    pub async fn execute(&self) -> anyhow::Result<()> {
        let mut message_stream = self.receive_headers()?;
        while let Some(message) = message_stream.next().await {
            self.on_headers(message.headers);
        }
        self.is_over.set(true);
        Ok(())
    }

    pub fn can_proceed(&self) -> bool {
        let request_statuses = &[HeaderSliceStatus::Empty, HeaderSliceStatus::Waiting];
        let cant_receive_more =
            self.is_over.get() && self.header_slices.has_one_of_statuses(request_statuses);
        !cant_receive_more
    }

    fn on_headers(&self, headers: Vec<Header>) {
        if headers.len() < header_slices::HEADER_SLICE_SIZE {
            tracing::warn!(
                "FetchReceiveStage got a headers slice of a smaller size: {}",
                headers.len()
            );
            return;
        }
        let start_block_num = headers[0].number.as_u64();

        let slice_lock_opt = self.header_slices.find(start_block_num);

        match slice_lock_opt {
            Some(slice_lock) => {
                let mut slice = slice_lock.write();
                match slice.status {
                    HeaderSliceStatus::Waiting => {
                        slice.headers = Some(headers);
                        slice.status = HeaderSliceStatus::Downloaded;
                    }
                    unexpected_status => {
                        tracing::warn!("FetchReceiveStage ignores a headers slice that we didn't request starting at: {}; status = {:?}", start_block_num, unexpected_status);
                    }
                }
            }
            None => {
                tracing::warn!(
                    "FetchReceiveStage ignores a headers slice that we didn't request starting at: {}",
                    start_block_num
                );
            }
        }
    }

    fn receive_headers(
        &self,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = BlockHeadersMessage> + Send>>> {
        let in_stream = self
            .sentry
            .read()
            .receive_messages(EthMessageId::BlockHeaders)?;

        let out_stream = in_stream.map(|message| match message {
            Message::BlockHeaders(message) => message,
            _ => panic!("unexpected type {:?}", message.eth_id()),
        });

        Ok(Box::pin(out_stream))
    }
}
