use super::util::{AccountChanges, StorageChanges};
use crate::{models::*, StateBuffer, Transaction};
use async_trait::async_trait;
use bytes::Bytes;
use ethereum::Receipt;
use ethereum_types::*;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    marker::PhantomData,
};

pub struct Buffer<'db, 'tx, Tx>
where
    'db: 'tx,
    Tx: Transaction<'db>,
{
    txn: &'tx Tx,
    _marker: PhantomData<&'db ()>,

    historical_block: Option<u64>,

    headers: BTreeMap<Bytes<'tx>, BlockHeader>,
    bodies: BTreeMap<Bytes<'tx>, BlockBody>,
    difficulty: BTreeMap<Bytes<'tx>, U256>,

    accounts: HashMap<Address, Option<Account>>,

    // address -> incarnation -> location -> value
    storage: HashMap<Address, BTreeMap<u64, HashMap<H256, H256>>>,

    account_changes: BTreeMap<u64, AccountChanges<'tx>>, // per block
    storage_changes: BTreeMap<u64, StorageChanges<'tx>>, // per block

    incarnations: BTreeMap<Address, u64>,
    hash_to_code: BTreeMap<H256, Bytes<'tx>>,
    storage_prefix_to_code_hash: BTreeMap<Bytes<'tx>, H256>,
    receipts: BTreeMap<Bytes<'tx>, Bytes<'tx>>,
    logs: BTreeMap<Bytes<'tx>, Bytes<'tx>>,

    batch_size: usize,

    // Current block stuff
    block_number: u64,
    changed_storage: HashSet<Address>,
}

impl<'db, 'tx, Tx> Buffer<'db, 'tx, Tx>
where
    'db: 'tx,
    Tx: Transaction<'db>,
{
    pub fn new(txn: &'tx Tx) -> Self {
        Self {
            txn,
            _marker: PhantomData,
            historical_block: Default::default(),
            headers: Default::default(),
            bodies: Default::default(),
            difficulty: Default::default(),
            accounts: Default::default(),
            storage: Default::default(),
            account_changes: Default::default(),
            storage_changes: Default::default(),
            incarnations: Default::default(),
            hash_to_code: Default::default(),
            storage_prefix_to_code_hash: Default::default(),
            receipts: Default::default(),
            logs: Default::default(),
            batch_size: Default::default(),
            block_number: Default::default(),
            changed_storage: Default::default(),
        }
    }
}

#[async_trait]
impl<'db, 'tx, Tx> StateBuffer<'tx> for Buffer<'db, 'tx, Tx>
where
    'db: 'tx,
    Tx: Transaction<'db>,
{
    // Readers

    async fn read_account(&self, address: Address) -> anyhow::Result<Option<Account>> {
        todo!()
    }

    async fn read_code(&self, code_hash: H256) -> anyhow::Result<Bytes<'tx>> {
        todo!()
    }

    async fn read_storage(
        &self,
        address: Address,
        incarnation: u64,
        location: H256,
    ) -> anyhow::Result<H256> {
        todo!()
    }

    // Previous non-zero incarnation of an account; 0 if none exists.
    async fn previous_incarnation(&self, address: Address) -> anyhow::Result<u64> {
        todo!()
    }

    async fn read_header(
        &self,
        block_number: u64,
        block_hash: H256,
    ) -> anyhow::Result<Option<BlockHeader>> {
        todo!()
    }

    async fn read_body(
        &self,
        block_number: u64,
        block_hash: H256,
    ) -> anyhow::Result<Option<BlockBody>> {
        todo!()
    }

    async fn total_difficulty(
        &self,
        block_number: u64,
        block_hash: H256,
    ) -> anyhow::Result<Option<U256>> {
        todo!()
    }

    async fn state_root_hash(&self) -> anyhow::Result<H256> {
        todo!()
    }

    async fn current_canonical_block(&self) -> anyhow::Result<u64> {
        todo!()
    }

    async fn canonical_hash(&self, block_number: u64) -> anyhow::Result<Option<H256>> {
        todo!()
    }

    async fn insert_block(&mut self, block: &BlockBody, hash: H256) -> anyhow::Result<()> {
        todo!()
    }

    async fn canonize_block(&mut self, block_number: u64, block_hash: H256) -> anyhow::Result<()> {
        todo!()
    }

    async fn decanonize_block(&mut self, block_number: u64) -> anyhow::Result<()> {
        todo!()
    }

    async fn insert_receipts(
        &mut self,
        block_number: u64,
        receipts: &[Receipt],
    ) -> anyhow::Result<()> {
        todo!()
    }

    /// State changes
    /// Change sets are backward changes of the state, i.e. account/storage values _at the beginning of a block_.

    /// Mark the beggining of a new block.
    /// Must be called prior to calling update_account/update_account_code/update_storage.
    async fn begin_block(&mut self, block_number: u64) -> anyhow::Result<()> {
        todo!()
    }

    async fn update_account(
        &mut self,
        address: Address,
        initial: Option<Account>,
        current: Option<Account>,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn update_account_code(
        &mut self,
        address: Address,
        incarnation: u64,
        code_hash: H256,
        code: Bytes<'tx>,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn update_storage(
        &mut self,
        address: Address,
        incarnation: u64,
        location: H256,
        initial: H256,
        current: H256,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn unwind_state_changes(&mut self, block_number: u64) -> anyhow::Result<()> {
        todo!()
    }
}
