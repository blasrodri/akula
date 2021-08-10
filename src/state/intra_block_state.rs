use crate::{common::EMPTY_HASH, models::Account};

use super::{delta::*, object::*, *};
use bytes::Bytes;
use ethereum::Log;
use ethereum_types::*;
use std::collections::*;

#[derive(Debug)]
pub struct IntraBlockState<'storage, 'r, R>
where
    R: StateBuffer<'storage>,
{
    db: &'r R,

    pub(crate) objects: HashMap<Address, Object>,
    pub(crate) storage: HashMap<Address, Storage>,

    // pointer stability?
    pub(crate) existing_code: HashMap<H256, Bytes<'storage>>,
    pub(crate) new_code: HashMap<H256, Bytes<'storage>>,

    pub(crate) journal: Vec<Box<dyn Delta<'storage, 'r, R>>>,

    // substate
    pub(crate) self_destructs: HashSet<Address>,
    pub(crate) logs: Vec<Log>,
    pub(crate) touched: HashSet<Address>,
    pub(crate) refund: u64,
    // EIP-2929 substate
    pub(crate) accessed_addresses: HashSet<Address>,
    pub(crate) accessed_storage_keys: HashMap<Address, HashSet<H256>>,
}

async fn get_object<'m, 'storage, 'r, R: StateReader<'storage>>(
    db: &'r R,
    objects: &'m mut HashMap<Address, Object>,
    address: Address,
) -> anyhow::Result<Option<&'m mut Object>> {
    Ok(match objects.entry(address) {
        hash_map::Entry::Occupied(entry) => Some(entry.into_mut()),
        hash_map::Entry::Vacant(entry) => {
            let accdata = db.read_account_data(address).await?;

            if let Some(account) = accdata {
                Some(entry.insert(Object {
                    initial: Some(account.clone()),
                    current: Some(account),
                }))
            } else {
                None
            }
        }
    })
}

async fn ensure_object<'m, 'storage, 'r, R: StateReader<'storage>>(
    db: &'r R,
    objects: &'m mut HashMap<Address, Object>,
    journal: &mut Vec<Box<dyn Delta<'storage, 'r, R>>>,
    address: Address,
) -> anyhow::Result<()> {
    if let Some(obj) = get_object(db, objects, address).await? {
        if obj.current.is_none() {
            journal.push(Box::new(UpdateDelta::new(address, obj.clone())));
            obj.current = Some(Account::default());
        }
    } else {
        journal.push(Box::new(CreateDelta::new(address)));
        objects.entry(address).insert(Object {
            current: Some(Account::default()),
            ..Default::default()
        });
    }

    Ok(())
}

async fn get_or_create_object<'m, 'storage, 'r, R: StateReader<'storage>>(
    db: &'r R,
    objects: &'m mut HashMap<Address, Object>,
    journal: &mut Vec<Box<dyn Delta<'storage, 'r, R>>>,
    address: Address,
) -> anyhow::Result<&'m mut Object> {
    ensure_object(db, objects, journal, address).await?;
    Ok(objects.get_mut(&address).unwrap())
}

impl<'storage, 'r, R: StateBuffer<'storage>> IntraBlockState<'storage, 'r, R> {
    pub fn new(db: &'r R) -> Self {
        Self {
            db,
            objects: Default::default(),
            storage: Default::default(),
            existing_code: Default::default(),
            new_code: Default::default(),
            journal: Default::default(),
            self_destructs: Default::default(),
            logs: Default::default(),
            touched: Default::default(),
            refund: Default::default(),
            accessed_addresses: Default::default(),
            accessed_storage_keys: Default::default(),
        }
    }

    pub fn db(&self) -> &'r R {
        self.db
    }

    pub async fn exists(&mut self, address: Address) -> anyhow::Result<bool> {
        let obj = get_object(self.db, &mut self.objects, address).await?;

        if let Some(obj) = obj {
            if obj.current.is_some() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    // https://eips.ethereum.org/EIPS/eip-161
    pub async fn is_dead(&mut self, address: Address) -> anyhow::Result<bool> {
        let obj = get_object(self.db, &mut self.objects, address).await?;

        if let Some(obj) = obj {
            if let Some(current) = &obj.current {
                return Ok(current.code_hash == EMPTY_HASH
                    && current.nonce == 0
                    && current.balance.is_zero());
            }
        }

        Ok(true)
    }

    pub async fn create_contract(&mut self, address: Address) -> anyhow::Result<()> {
        let mut created = Object {
            current: Some(Account::default()),
            ..Default::default()
        };

        let mut prev_incarnation: Option<u64> = None;
        let prev = get_object(self.db, &mut self.objects, address).await?;
        if let Some(prev) = prev {
            created.initial = prev.initial.clone();
            if let Some(prev_current) = &prev.current {
                created.current.as_mut().unwrap().balance = prev_current.balance;
                prev_incarnation = Some(prev_current.incarnation);
            } else if let Some(prev_initial) = &prev.initial {
                prev_incarnation = Some(prev_initial.incarnation);
            }
            self.journal
                .push(Box::new(UpdateDelta::new(address, prev.clone())));
        } else {
            self.journal.push(Box::new(CreateDelta::new(address)));
        }

        if prev_incarnation.unwrap_or(0) == 0 {
            // prev_incarnation = self.db.previous_incarnation(address);
        }

        // created.current->incarnation = *prev_incarnation + 1;

        // objects_[address] = created;

        // auto it{storage_.find(address)};
        // if (it == storage_.end()) {
        //     journal_.emplace_back(new state::StorageCreateDelta{address});
        // } else {
        //     journal_.emplace_back(new state::StorageWipeDelta{address, it->second});
        //     storage_.erase(address);
        // }

        todo!()
    }

    // void destruct(const evmc::address& address);

    // void record_suicide(const evmc::address& address) noexcept;
    // void destruct_suicides();
    // void destruct_touched_dead();

    // size_t number_of_self_destructs() const noexcept { return self_destructs_.size(); }

    pub async fn get_balance(&mut self, address: Address) -> anyhow::Result<U256> {
        Ok(get_object(self.db, &mut self.objects, address)
            .await?
            .map(|object| object.current.as_ref().map(|current| current.balance))
            .flatten()
            .unwrap_or_else(U256::zero))
    }
    // void set_balance(const evmc::address& address, const intx::uint256& value) noexcept;
    // void add_to_balance(const evmc::address& address, const intx::uint256& addend) noexcept;
    // void subtract_from_balance(const evmc::address& address, const intx::uint256& subtrahend) noexcept;

    // void touch(const evmc::address& address) noexcept;

    pub async fn get_nonce(&mut self, address: Address) -> anyhow::Result<u64> {
        if let Some(object) = get_object(self.db, &mut self.objects, address).await? {
            if let Some(current) = &object.current {
                return Ok(current.nonce);
            }
        }

        Ok(0)
    }
    pub async fn set_nonce(&mut self, address: Address, nonce: u64) -> anyhow::Result<()> {
        let object =
            get_or_create_object(self.db, &mut self.objects, &mut self.journal, address).await?;
        self.journal
            .push(Box::new(UpdateDelta::new(address, object.clone())));

        object.current.as_mut().unwrap().nonce = 0;

        Ok(())
    }

    // ByteView get_code(const evmc::address& address) const noexcept;
    // evmc::bytes32 get_code_hash(const evmc::address& address) const noexcept;
    // void set_code(const evmc::address& address, Bytes code) noexcept;

    // evmc_access_status access_account(const evmc::address& address) noexcept;

    // evmc_access_status access_storage(const evmc::address& address, const evmc::bytes32& key) noexcept;

    // evmc::bytes32 get_current_storage(const evmc::address& address, const evmc::bytes32& key) const noexcept;

    // // https://eips.ethereum.org/EIPS/eip-2200
    // evmc::bytes32 get_original_storage(const evmc::address& address, const evmc::bytes32& key) const noexcept;

    // void set_storage(const evmc::address& address, const evmc::bytes32& key, const evmc::bytes32& value) noexcept;

    // void write_to_db(uint64_t block_number);

    // Snapshot take_snapshot() const noexcept;
    // void revert_to_snapshot(const Snapshot& snapshot) noexcept;

    // void finalize_transaction();

    // // See Section 6.1 "Substate" of the Yellow Paper
    // void clear_journal_and_substate();

    // void add_log(const Log& log) noexcept;

    // const std::vector<Log>& logs() const noexcept { return logs_; }

    // void add_refund(uint64_t addend) noexcept;
    // void subtract_refund(uint64_t subtrahend) noexcept;

    // uint64_t get_refund() const noexcept { return refund_; }
}
