use std::sync::Arc;

use bytes::Bytes;

use super::*;
use crate::iterators::impls::StorageIterator;
use crate::table::SsTableBuilder;

#[test]
fn test_sst_build_single_key() {
    let mut builder = SsTableBuilder::new(16);
    builder.add(b"233", b"233333");
    builder.build("").unwrap();
}

#[test]
fn test_sst_build_two_blocks() {
    let mut builder = SsTableBuilder::new(16);
    builder.add(b"11", b"11");
    builder.add(b"22", b"22");
    builder.add(b"33", b"11");
    builder.add(b"44", b"22");
    builder.add(b"55", b"11");
    builder.add(b"66", b"22");
    assert!(builder.meta.len() >= 2);
    builder.build("").unwrap();
}

fn key_of(idx: usize) -> Vec<u8> {
    format!("key_{:03}", idx * 5).into_bytes()
}

fn value_of(idx: usize) -> Vec<u8> {
    format!("value_{:010}", idx).into_bytes()
}

fn num_of_keys() -> usize {
    100
}

fn generate_sst() -> SsTable {
    let mut builder = SsTableBuilder::new(128);
    for idx in 0..num_of_keys() {
        let key = key_of(idx);
        let value = value_of(idx);
        builder.add(&key[..], &value[..]);
    }
    builder.build("").unwrap()
}

#[test]
fn test_sst_build_all() {
    generate_sst();
}

#[test]
fn test_sst_decode() {
    let sst = generate_sst();
    let meta = sst.block_metas.clone();
    let new_sst = SsTable::open(sst.file).unwrap();
    assert_eq!(new_sst.block_metas, meta);
}

fn as_bytes(x: &[u8]) -> Bytes {
    Bytes::copy_from_slice(x)
}

#[test]
fn test_sst_iterator() {
    let sst = Arc::new(generate_sst());
    let mut iter = SsTableIterator::create_and_seek_to_first(sst).unwrap();
    for _ in 0..5 {
        for i in 0..num_of_keys() {
            let key = iter.key();
            let value = iter.value();
            assert_eq!(
                key,
                key_of(i),
                "expected key: {:?}, actual key: {:?}",
                as_bytes(&key_of(i)),
                as_bytes(key)
            );
            assert_eq!(
                value,
                value_of(i),
                "expected value: {:?}, actual value: {:?}",
                as_bytes(&value_of(i)),
                as_bytes(value)
            );
            iter.next().unwrap();
        }
        iter.seek_to_first().unwrap();
    }
}

#[test]
fn test_sst_seek_key() {
    let sst = Arc::new(generate_sst());
    let mut iter = SsTableIterator::create_and_seek_to_key(sst, &key_of(0)).unwrap();
    for offset in 1..=5 {
        for i in 0..num_of_keys() {
            let key = iter.key();
            let value = iter.value();
            assert_eq!(
                key,
                key_of(i),
                "expected key: {:?}, actual key: {:?}",
                as_bytes(&key_of(i)),
                as_bytes(key)
            );
            assert_eq!(
                value,
                value_of(i),
                "expected value: {:?}, actual value: {:?}",
                as_bytes(&value_of(i)),
                as_bytes(value)
            );
            iter.seek_to_key(&format!("key_{:03}", i * 5 + offset).into_bytes())
                .unwrap();
        }
        iter.seek_to_key(b"k").unwrap();
    }
}
