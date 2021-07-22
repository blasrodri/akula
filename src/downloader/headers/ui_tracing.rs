use crate::downloader::{headers::header_slices::HeaderSlices, ui_view::UIView};
use std::sync::Arc;

pub struct HeaderSlicesView {
    header_slices: Arc<HeaderSlices>,
}

impl HeaderSlicesView {
    pub fn new(header_slices: Arc<HeaderSlices>) -> Self {
        Self { header_slices }
    }
}

impl UIView for HeaderSlicesView {
    fn draw(&self) -> anyhow::Result<()> {
        tracing::info!("downloading headers...");
        Ok(())
    }
}
