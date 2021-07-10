use anyhow::Result;
use bytes::Bytes;
use crate::{
    kv::traits::{Cursor, MutableCursor},
    kv::{CustomTable, Table},
    MutableTransaction
};
use std::{collections::{HashMap, BTreeMap, BTreeSet}, ops::Deref};
use thiserror::Error;
use akula_table_defs::{TABLES, TableInfo};

#[derive(Debug, Error)]
enum MutationError {
    #[error("Table not found.")]
    TableNotFound,
    #[error("Wrong type of table (dupsort vs. Non-dupsort)")]
    WrongTable,
}

type TableName = string::String<static_bytes::Bytes>;
type SimpleBucket = BTreeMap<Vec<u8>, Option<Vec<u8>>>;
type SimpleBuffer = HashMap<TableName, SimpleBucket>;
type DupSortValues = BTreeSet<Vec<u8>>;

#[derive(Default)]
struct DupSortBucket {
    insert: DupSortValues,
    delete: DupSortValues,
}

type DupSortBuffer = HashMap<TableName, DupSortBucket>;

fn init_buffers() -> (HashMap<TableName, SimpleBucket>, HashMap<TableName, DupSortBucket>) {
    let mut simple_buffer = SimpleBuffer::default();
    let mut dupsort_buffer = DupSortBuffer::default();

    for (table_name, info) in TABLES.deref() {
        if info.dup_sort.is_some() {
            dupsort_buffer.insert(TableName::from_str(table_name), Default::default());
        } else {
            simple_buffer.insert(TableName::from_str(table_name), Default::default());
        }
    }

    (simple_buffer, dupsort_buffer)
}

struct Mutation<'tx, Tx: MutableTransaction<'tx>> {
    parent: &'tx mut Tx,
    simple_buffer: HashMap<TableName, SimpleBucket>,
    dupsort_buffer: HashMap<TableName, DupSortBucket>,
    sequence_increment: HashMap<TableName, u64>,
}

impl<'tx, Tx: MutableTransaction<'tx>> Mutation<'tx, Tx> {
    fn new(parent: &'tx mut Tx) -> Self {
        let (simple_buffer, dupsort_buffer) = init_buffers();
        Mutation {
            parent,
            simple_buffer,
            dupsort_buffer,
            sequence_increment: Default::default(),
        }
    }

    async fn get<T>(&'tx self, table: &T, key: &[u8]) -> Result<Option<Bytes<'tx>>>
    where
        T: Table,
    {
        let table_name = table.db_name();

        if let Some(bucket) = self.simple_buffer.get(&table_name) {
            Ok(match bucket.get(key) {
                Some(entry) => match entry {
                    Some(value) => Some(value.as_slice().into()),
                    None => None,
                }
                None => self.parent.get(table, key).await?
            })
        }

        else if let Some(bucket) = self.dupsort_buffer.get(&table_name) {
            todo!()
        }

        else {
            Err(MutationError::TableNotFound.into())
        }

    }

    async fn read_sequence<T>(&'tx self, table: &T) -> Result<u64>
    where
        T: Table
    {
        let parent_value = self.parent.read_sequence(table).await?;
        let name = table.db_name();
        let increment = self.sequence_increment.get(&name).unwrap_or(&0);

        Ok(parent_value + increment)
    }

    async fn set<T>(&mut self, table: &T, k: &[u8], v: &[u8]) -> Result<()>
    where
        T: Table,
    {
        let table_name = table.db_name();

        if let Some(bucket) = self.simple_buffer.get_mut(&table_name) {
            bucket.insert(k.to_vec(), Some(v.to_vec()));
            Ok(())
        }

        else if let Some(bucket) = self.dupsort_buffer.get_mut(&table_name) {
            todo!();
            Ok(())
        }

        else {
            Err(MutationError::TableNotFound.into())
        }
    }

    async fn delete_key<T>(&mut self, table: &T, k: &[u8]) -> Result<()>
    where
        T: Table,
    {
        let table_name = table.db_name();

        if let Some(bucket) = self.simple_buffer.get_mut(&table_name) {
            bucket.insert(k.to_vec(), None);
            Ok(())
        }

        else if let Some(bucket) = self.dupsort_buffer.get_mut(&table_name) {
            todo!();
            Ok(())
        }

        else {
            Err(MutationError::TableNotFound.into())
        }
    }

    async fn delete_pair<T>(&mut self, table: &T, k: &[u8], v: &[u8]) -> Result<()>
    where
        T: Table,
    {
        let table_name = table.db_name();

        if let Some(bucket) = self.simple_buffer.get_mut(&table_name) {
            if let Some(value_or_none) = bucket.get(k) {
                if let Some(value) = value_or_none {
                    if value == v {
                        bucket.insert(k.to_vec(), None);
                    }
                }
            }
            Ok(())
        }

        else if let Some(bucket) = self.dupsort_buffer.get_mut(&table_name) {
            todo!();
            Ok(())
        }

        else {
            Err(MutationError::TableNotFound.into())
        }
    }

    async fn commit(self) -> anyhow::Result<()> {
        for (table_name, bucket) in self.simple_buffer {
            let table = CustomTable { 0: table_name };
            let mut cursor = self.parent.mutable_cursor(&table).await?;
            for (ref key, ref maybe_value) in bucket {
                match maybe_value {
                    Some(ref value) => cursor.put(key, value).await?,
                    None => {
                        let maybe_deleted_pair = cursor.seek_exact(key).await?;
                        if let Some((ref key, ref value)) = maybe_deleted_pair {
                            cursor.delete(key, value).await?;
                        }
                    }
                };
            }
        }

        // TODO: dupsort buckets

        for (table_name, increment) in self.sequence_increment {
            if increment > 0 {
                let table = CustomTable { 0: table_name };
                self.parent.increment_sequence(&table, increment).await?;
            }
        }

        Ok(())
    }

    async fn increment_sequence<T>(&mut self, table: &T, amount: u64) -> Result<u64>
    where
        T: Table
    {
        let parent_value = self.parent.read_sequence(table).await?;
        let name = table.db_name();
        let increment = self.sequence_increment.get(&name).unwrap_or(&0);
        let current = parent_value + increment;
        self.sequence_increment.insert(name, current + amount);
        Ok(current)
    }

}
