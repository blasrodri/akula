use anyhow::Result;
use bytes::Bytes;
use crate::{
    kv::traits::{Cursor, MutableCursor},
    kv::{CustomTable, Table},
    Transaction,
    MutableTransaction
};
use std::collections::{HashMap, BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Error)]
enum MutationError {
    #[error("Table not found.")]
    TableNotFound,
    #[error("Wrong type of table (dupsort vs. Non-dupsort)")]
    WrongTable
}

type TableName = string::String<static_bytes::Bytes>;
type DupSortValues = BTreeSet<Vec<u8>>;
type DupSortBucket = BTreeMap<Vec<u8>, DupSortValues>;
type SimpleBucket = BTreeMap<Vec<u8>, Option<Vec<u8>>>;

enum Bucket {
    Simple(SimpleBucket),
    DupSort(DupSortBucket, DupSortBucket),
}

struct Mutation<'tx, Tx: MutableTransaction<'tx>> {
    parent: &'tx mut Tx,
    buffer: HashMap<TableName, Bucket>,
    sequence_increment: HashMap<TableName, u64>,
}

impl<'tx, Tx: MutableTransaction<'tx>> Mutation<'tx, Tx> {
    async fn get<T>(&'tx self, table: &T, key: &[u8]) -> Result<Option<Bytes<'tx>>>
    where
        T: Table,
    {
        let bucket = self.buffer
            .get(&table.db_name())
            .ok_or(MutationError::TableNotFound)?;

        match bucket {
            Bucket::Simple(mutate) => {
                Ok(match mutate.get(key) {
                    Some(entry) => match entry {
                        Some(value) => Some(value.as_slice().into()),
                        None => None,
                    }
                    None => self.parent.get(table, key).await?
                })
            },
            Bucket::DupSort(insert, delete) => {
                todo!()
            }
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
        let bucket = self.buffer
            .get_mut(&table.db_name())
            .ok_or(MutationError::TableNotFound)?;

        match bucket {
            Bucket::Simple(mutate) => mutate.insert(k.to_vec(), Some(v.to_vec())),
            Bucket::DupSort(insert, _) => todo!(),
        };

        Ok(())
    }

    async fn del<T>(&mut self, table: &T, k: &[u8]) -> Result<()>
    where
        T: Table,
    {
        let bucket = self.buffer
            .get_mut(&table.db_name())
            .ok_or(MutationError::TableNotFound)?;

        match bucket {
            Bucket::Simple(simple_bucket) => {
                simple_bucket.insert(k.to_vec(), None);
                Ok(())
            },
            Bucket::DupSort(_, _) => Err(MutationError::WrongTable.into())
        }
    }

    async fn commit(self) -> anyhow::Result<()> {
        for (table_name, bucket) in self.buffer {
            let table = CustomTable { 0: table_name };
            let mut cursor = self.parent.mutable_cursor(&table).await?;
            match bucket {
                Bucket::Simple(simple_bucket) => {
                    for (ref key, ref maybe_value) in simple_bucket {
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
                },
                Bucket::DupSort(_, _) => todo!()
            }
        }

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
