use bytes::Bytes;
use ethereum_types::{Address, H256};
use std::collections::BTreeMap;

// address -> storage-encoded initial value
pub type AccountChanges<'tx> = BTreeMap<Address, Bytes<'tx>>;

// address -> incarnation -> location -> zeroless initial value
pub type StorageChanges<'tx> = BTreeMap<Address, BTreeMap<u64, BTreeMap<H256, Bytes<'tx>>>>;
