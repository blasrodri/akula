use ethereum::Header;
use parking_lot::RwLock;
use std::collections::LinkedList;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HeaderSliceStatus {
    Empty,
    Waiting,
    Downloaded,
    Verified,
    Saved,
}

pub struct HeaderSlice {
    pub start_block_num: u64,
    pub status: HeaderSliceStatus,
    pub headers: Option<Vec<Header>>,
}

pub struct HeaderSlices {
    slices: LinkedList<RwLock<HeaderSlice>>,
    max_slices: usize,
}

pub const HEADER_SLICE_SIZE: usize = 192;

impl HeaderSlices {
    pub fn new(mem_limit: usize) -> Self {
        let max_slices = mem_limit / std::mem::size_of::<Header>() / HEADER_SLICE_SIZE;

        let mut slices = LinkedList::new();
        for i in 0..max_slices {
            let slice = HeaderSlice {
                start_block_num: (i * HEADER_SLICE_SIZE) as u64,
                status: HeaderSliceStatus::Empty,
                headers: None,
            };
            slices.push_back(RwLock::new(slice));
        }

        Self { slices, max_slices }
    }

    pub fn clone_statuses(&self) -> Vec<HeaderSliceStatus> {
        self.slices
            .iter()
            .map(|slice| slice.read().status)
            .collect::<Vec<HeaderSliceStatus>>()
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = &RwLock<HeaderSlice>> + '_> {
        Box::new(self.slices.iter())
    }

    pub fn find(&self, start_block_num: u64) -> Option<&RwLock<HeaderSlice>> {
        self.slices
            .iter()
            .find(|slice| slice.read().start_block_num == start_block_num)
    }

    pub fn has_one_of_statuses(&self, statuses: &[HeaderSliceStatus]) -> bool {
        self.slices
            .iter()
            .any(|slice| statuses.contains(&slice.read().status))
    }
}
