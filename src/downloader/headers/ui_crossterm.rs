use crate::downloader::{
    headers::header_slices::{HeaderSliceStatus, HeaderSlices},
    ui_view::UIView,
};
use crossterm::{cursor, style, terminal, QueueableCommand};
use std::{
    io::{stdout, Write},
    sync::Arc,
};

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
        let statuses = self.header_slices.clone_statuses();

        let mut stdout = stdout();

        // save the logging position
        stdout.queue(cursor::SavePosition {})?;

        // draw at the top of the window
        stdout.queue(cursor::MoveTo(0, 0))?;
        stdout.queue(terminal::EnableLineWrap {})?;

        // overall progress
        stdout.queue(style::Print("downloading headers... "))?;
        stdout.queue(terminal::Clear(terminal::ClearType::UntilNewLine))?;
        stdout.queue(cursor::MoveToNextLine(1))?;

        // slice statuses
        for status in statuses {
            let c = char::from(status);
            stdout.queue(style::Print(c))?;
        }
        stdout.queue(terminal::Clear(terminal::ClearType::UntilNewLine))?;

        // delimiter line
        stdout.queue(cursor::MoveToNextLine(1))?;
        stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
        stdout.queue(style::Print("\n"))?;

        stdout.queue(cursor::RestorePosition {})?;
        stdout.flush()?;

        Ok(())
    }
}

impl From<HeaderSliceStatus> for char {
    fn from(status: HeaderSliceStatus) -> Self {
        match status {
            HeaderSliceStatus::Empty => '-',
            HeaderSliceStatus::Waiting => '<',
            HeaderSliceStatus::Downloaded => '.',
            HeaderSliceStatus::Verified => '#',
            HeaderSliceStatus::Saved => '+',
        }
    }
}
