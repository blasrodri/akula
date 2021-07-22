use crate::downloader::{
    chain_config::{ChainConfig, ChainsConfig},
    headers::{
        fetch_receive_stage::FetchReceiveStage, fetch_request_stage::FetchRequestStage,
        header_slices::HeaderSlices, save_stage::SaveStage, verify_stage::VerifyStage,
    },
    opts::Opts,
    sentry_client,
    sentry_client::SentryClient,
    sentry_client_impl::SentryClientImpl,
    sentry_client_reactor::SentryClientReactor,
};
use parking_lot::RwLock;
use std::sync::Arc;

pub struct Downloader {
    opts: Opts,
    chain_config: ChainConfig,
}

impl Downloader {
    pub fn new(opts: Opts, chains_config: ChainsConfig) -> Self {
        let chain_config = chains_config.0[&opts.chain_name].clone();

        Self { opts, chain_config }
    }

    pub async fn run(
        &self,
        sentry_client_opt: Option<Box<dyn SentryClient>>,
    ) -> anyhow::Result<()> {
        let status = sentry_client::Status {
            total_difficulty: ethereum_types::U256::zero(),
            best_hash: ethereum_types::H256::zero(),
            chain_fork_config: self.chain_config.clone(),
            max_block: 0,
        };

        let mut sentry_client = match sentry_client_opt {
            Some(v) => v,
            None => Box::new(SentryClientImpl::new(self.opts.sentry_api_addr.clone()).await?),
        };

        sentry_client.set_status(status).await?;

        let mut sentry_reactor = SentryClientReactor::new(sentry_client);
        sentry_reactor.start();

        let mut ui_system = crate::downloader::ui_system::UISystem::new();
        ui_system.start();

        let header_slices = Arc::new(HeaderSlices::new(50 << 20 /* 50 Mb */));
        let sentry = Arc::new(RwLock::new(sentry_reactor));

        let header_slices_view =
            crate::downloader::headers::HeaderSlicesView::new(Arc::clone(&header_slices));
        ui_system.set_view(Some(Box::new(header_slices_view)));

        let fetch_request_stage =
            FetchRequestStage::new(Arc::clone(&header_slices), Arc::clone(&sentry));

        let fetch_receive_stage =
            FetchReceiveStage::new(Arc::clone(&header_slices), Arc::clone(&sentry));

        let fetch_receive_stage_execute = fetch_receive_stage.execute();
        tokio::pin!(fetch_receive_stage_execute);

        let verify_stage = VerifyStage::new(Arc::clone(&header_slices));

        let save_stage = SaveStage::new(Arc::clone(&header_slices));

        loop {
            tokio::select! {
                result = fetch_request_stage.execute() => {
                    if result.is_err() {
                        tracing::error!("Downloader headers fetch request stage failure: {:?}", result);
                        break;
                    }
                }
                result = &mut fetch_receive_stage_execute => {
                    if result.is_err() {
                        tracing::error!("Downloader headers fetch receive stage failure: {:?}", result);
                        break;
                    }
                }
                else => {
                    break;
                }
            }

            // TODO: async?
            let result = verify_stage.execute();
            if result.is_err() {
                tracing::error!("Downloader headers verify stage failure: {:?}", result);
                break;
            }

            // TODO: async in the loop above?
            let result = save_stage.execute().await;
            if result.is_err() {
                tracing::error!("Downloader headers save stage failure: {:?}", result);
                break;
            }

            if !fetch_receive_stage.can_proceed() {
                break;
            }
        }

        ui_system.stop().await?;

        {
            let mut sentry_reactor = sentry.write();
            sentry_reactor.stop().await?;
        }

        Ok(())
    }
}
