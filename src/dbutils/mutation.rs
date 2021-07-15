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
use std::marker::PhantomData;

#[derive(Debug, Error)]
enum MutationError {
    #[error("Table not found.")]
    TableNotFound,
}

type TableName = string::String<static_bytes::Bytes>;
type SimpleBucket = BTreeMap<Vec<u8>, Option<Vec<u8>>>;
type SimpleBuffer = HashMap<TableName, SimpleBucket>;
type DupSortValues = BTreeSet<Vec<u8>>;

#[derive(Default)]
struct DupSortChanges {
    insert: DupSortValues,
    delete: DupSortValues,
}

type DupSortBucket = BTreeMap<Vec<u8>, DupSortChanges>;
type DupSortBuffer = HashMap<TableName, DupSortBucket>;

fn is_dupsort(table_name: &str) -> bool {
    if let Some(info) = TABLES.get(table_name) {
        info.dup_sort.is_some()
    } else {
        false
    }
}

struct Mutation<'tx: 'm, 'm, Tx: MutableTransaction<'tx>> {
    parent: &'m mut Tx,
    phantom: PhantomData<&'tx i32>,
    simple_buffer: HashMap<TableName, SimpleBucket>,
    dupsort_buffer: HashMap<TableName, DupSortBucket>,
    sequence_increment: HashMap<TableName, u64>,
}

impl<'tx: 'm, 'm, Tx: MutableTransaction<'tx>> Mutation<'tx, 'm, Tx> {
    fn new(parent: &'m mut Tx) -> Self {
        Mutation {
            parent,
            phantom: PhantomData,
            simple_buffer: Default::default(),
            dupsort_buffer: Default::default(),
            sequence_increment: Default::default(),
        }
    }

    async fn get<T>(&'m self, table: &T, key: &[u8]) -> Result<Option<Bytes<'m>>>
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

    async fn read_sequence<T>(&self, table: &T) -> Result<u64>
    where
        T: Table
    {
        let parent_value = self.parent.read_sequence(table).await?;
        let increment = self.sequence_increment.get(&table.db_name()).unwrap_or(&0);

        Ok(parent_value + increment)
    }

    fn get_simple_buffer(&mut self, name: &TableName) -> &mut SimpleBucket {
        if !self.simple_buffer.contains_key(name) {
            self.simple_buffer.insert(name.clone(), Default::default());
        }
        self.simple_buffer.get_mut(name).unwrap()
    }

    fn get_dupsort_buffer(&mut self, name: &TableName) -> &mut DupSortBucket {
        if !self.dupsort_buffer.contains_key(name) {
            self.dupsort_buffer.insert(name.clone(), Default::default());
        }
        self.dupsort_buffer.get_mut(name).unwrap()
    }

    fn get_dupsort_changes(&mut self, name: &TableName, k: &[u8]) -> &mut DupSortChanges {
        let buffer = self.get_dupsort_buffer(name);
        if !buffer.contains_key(k) {
            buffer.insert(k.to_vec(), Default::default());
        }
        buffer.get_mut(k).unwrap()
    }

    async fn set<T>(&mut self, table: &T, k: &[u8], v: &[u8]) -> Result<()>
    where
        T: Table,
    {
        let table_name = table.db_name();

        if is_dupsort(&table_name) {
            let mut changes = self.get_dupsort_changes(&table_name, k);
            changes.insert.insert(v.to_vec());
            changes.delete.remove(v);
        }

        else {
            self.get_simple_buffer(&table_name).insert(k.to_vec(), Some(v.to_vec()));
        }

        Ok(())
    }

    async fn delete_key<T>(&mut self, table: &T, k: &[u8]) -> Result<()>
    where
        T: Table,
    {
        let table_name = table.db_name();

        if is_dupsort(&table_name) {
            todo!();
        }

        else {
            self.get_simple_buffer(&table_name).insert(k.to_vec(), None);
        }

        Ok(())
    }

    async fn delete_pair<T>(&mut self, table: &T, k: &[u8], v: &[u8]) -> Result<()>
    where
        T: Table,
    {
        let table_name = table.db_name();

        if is_dupsort(&table_name) {
            todo!();
        }

        else {
            let bucket = self.get_simple_buffer(&table_name);
            if let Some(value_or_none) = bucket.get(k) {
                if let Some(value) = value_or_none {
                    if value == v {
                        bucket.insert(k.to_vec(), None);
                    }
                }
            }
        }

        Ok(())
    }

    async fn commit(self) -> anyhow::Result<()> {
        for (table_name, bucket) in self.simple_buffer {
            let table = CustomTable { 0: table_name };
            let mut cursor = self.parent.mutable_cursor(&table).await?;
            for (ref key, ref maybe_value) in bucket {
                match maybe_value {
                    Some(ref value) => cursor.put(key, value).await?,
                    None => {
                        let maybe_deleted_pair = cursor.seek_exact(key).await;
                        if let Ok(Some((ref key, ref value))) = maybe_deleted_pair {
                            cursor.delete(key, value).await?;
                        }
                    }
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use mdbx::{Environment, NoWriteMap, DatabaseFlags};

    #[tokio::test]
    async fn test_mutation() {
        let dir = tempdir().unwrap();
        let env = Environment::<NoWriteMap>::new().set_max_dbs(2).open(dir.path()).unwrap();
        let mut tx = env.begin_rw_txn().unwrap();
        let table = CustomTable::from("TxSender".to_string());
        {
            let db = tx.create_db(Some("TxSender"), DatabaseFlags::default()).unwrap();
            tx.put(&db, Bytes::from("a"), Bytes::from("xxx"), Default::default()).unwrap();
            tx.put(&db, Bytes::from("c"), Bytes::from("zzz"), Default::default()).unwrap();
            tx.put(&db, Bytes::from("b"), Bytes::from("yyy"), Default::default()).unwrap();
        }
        {
            let ref_tx = &mut tx;
            let mut mutation = Mutation::new(ref_tx);
            mutation.set(&table, &Bytes::from("a1"), &Bytes::from("aaa")).await.unwrap();
            mutation.set(&table, &Bytes::from("c"), &Bytes::from("bbb")).await.unwrap();
            mutation.delete_key(&table, &Bytes::from("b")).await.unwrap();
            assert_eq!(mutation.get(&table, &Bytes::from("a")).await.unwrap().unwrap(), Bytes::from("xxx"));
            assert_eq!(mutation.get(&table, &Bytes::from("a1")).await.unwrap().unwrap(), Bytes::from("aaa"));
            assert!(mutation.get(&table, &Bytes::from("b")).await.unwrap().is_none());
            mutation.commit().await.unwrap();
        }
        {
            let db = tx.open_db(Some("TxSender")).unwrap();
            assert_eq!(tx.get(&db, &Bytes::from("a")).unwrap().unwrap(), Bytes::from("xxx"));
            assert_eq!(tx.get(&db, &Bytes::from("a1")).unwrap().unwrap(), Bytes::from("aaa"));
            //assert!(tx.get(&db, &Bytes::from("b")).unwrap().is_none());
            assert_eq!(tx.get(&db, &Bytes::from("c")).unwrap().unwrap(), Bytes::from("bbb"));
        }
    }

}