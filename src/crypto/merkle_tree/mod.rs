use std::sync::atomic::{AtomicUsize, Ordering};

use ark_crypto_primitives::{merkle_tree::DigestConverter, Error};

pub mod blake3;
pub mod digest;
pub mod keccak;
pub mod parameters;
pub mod poseidon;
pub mod proof;

#[derive(Debug, Default)]
pub struct HashCounter;

static HASH_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl HashCounter {
    pub(crate) fn add() -> usize {
        HASH_COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    pub fn reset() {
        HASH_COUNTER.store(0, Ordering::SeqCst);
    }

    pub fn get() -> usize {
        HASH_COUNTER.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Default)]
pub struct QueryCounter;

static QUERY_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl QueryCounter {
    pub fn add(count: usize) -> usize {
        QUERY_COUNTER.fetch_add(count, Ordering::SeqCst)
    }

    pub fn reset() {
        QUERY_COUNTER.store(0, Ordering::SeqCst);
    }

    pub fn get() -> usize {
        QUERY_COUNTER.load(Ordering::SeqCst)
    }
}

/// A trivial converter where digest of previous layer's hash is the same as next layer's input.
pub struct IdentityDigestConverter<T> {
    _prev_layer_digest: T,
}

impl<T> DigestConverter<T, T> for IdentityDigestConverter<T> {
    type TargetType = T;
    fn convert(item: T) -> Result<T, Error> {
        Ok(item)
    }
}
