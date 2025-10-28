use std::{borrow::Borrow, marker::PhantomData};

use ark_crypto_primitives::{
    crh::{CRHScheme, TwoToOneCRHScheme},
    Error,
};
use ark_bn254::Fr as BN254Fr;
use ark_ff::PrimeField;
use ark_serialize::CanonicalSerialize;
use aptos_crypto::poseidon_bn254;
use rand::RngCore;
use serde::{Deserialize, Serialize};

use super::{digest::GenericDigest, parameters::MerkleTreeParams, HashCounter};

/// Digest type used in Poseidon-based Merkle trees.
///
/// Alias for a 32-byte generic digest (field element serialized).
pub type PoseidonDigest = GenericDigest<32>;

/// Merkle tree configuration using Poseidon as both leaf and node hasher.
/// Note: Poseidon is only supported for BN254 field (Field256).
pub type PoseidonMerkleTreeParams<F> =
    MerkleTreeParams<F, PoseidonLeafHash<F>, PoseidonCompress<F>, PoseidonDigest>;

/// Leaf hash function using Poseidon over field element inputs.
///
/// This struct implements `CRHScheme` where the input is a slice of
/// field elements `[F]`, and the output is a 32-byte digest derived from
/// a Poseidon hash using Aptos's implementation.
///
/// **IMPORTANT**: This implementation only works with BN254Fr field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct PoseidonLeafHash<F>(#[serde(skip)] PhantomData<F>);

impl<F: PrimeField + CanonicalSerialize + Send> CRHScheme for PoseidonLeafHash<F> {
    type Input = [F];
    type Output = PoseidonDigest;
    type Parameters = ();

    fn setup<R: RngCore>(_: &mut R) -> Result<Self::Parameters, Error> {
        Ok(())
    }

    fn evaluate<T: Borrow<Self::Input>>(
        (): &Self::Parameters,
        input: T,
    ) -> Result<Self::Output, Error> {
        let input_slice = input.borrow();
        
        // Convert input field elements to BN254Fr
        // Aptos's poseidon_bn254 works directly with Vec<BN254Fr>
        let bn254_inputs: Vec<BN254Fr> = input_slice
            .iter()
            .map(|elem| {
                // Serialize F to bytes and reconstruct as BN254Fr
                let mut bytes = Vec::new();
                elem.serialize_compressed(&mut bytes)
                    .expect("Serialization should not fail");
                BN254Fr::from_be_bytes_mod_order(&bytes)
            })
            .collect();
        
        // Hash using Aptos's Poseidon implementation
        // This handles variable-width inputs automatically via batching
        let hash_output = poseidon_bn254::hash_scalars(bn254_inputs)
            .expect("Poseidon hash_scalars should not fail for valid BN254 inputs");
        
        // Serialize the field element output to 32 bytes
        let mut output_bytes = [0u8; 32];
        let mut serialized = Vec::new();
        hash_output.serialize_compressed(&mut serialized)?;
        
        // Take the first 32 bytes (or pad if necessary)
        let len = serialized.len().min(32);
        output_bytes[..len].copy_from_slice(&serialized[..len]);
        
        HashCounter::add();
        Ok(output_bytes.into())
    }
}

/// Node compression function using Poseidon over two 32-byte digests.
///
/// This struct implements `TwoToOneCRHScheme`, combining two digests
/// by deserializing them back to field elements and hashing with Poseidon.
///
/// **IMPORTANT**: This implementation only works with BN254Fr field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct PoseidonCompress<F>(#[serde(skip)] PhantomData<F>);

impl<F: PrimeField + CanonicalSerialize + Send> TwoToOneCRHScheme for PoseidonCompress<F> {
    type Input = PoseidonDigest;
    type Output = PoseidonDigest;
    type Parameters = ();

    fn setup<R: RngCore>(_: &mut R) -> Result<Self::Parameters, Error> {
        Ok(())
    }

    fn evaluate<T: Borrow<Self::Input>>(
        (): &Self::Parameters,
        left_input: T,
        right_input: T,
    ) -> Result<Self::Output, Error> {
        // Deserialize the two digests to BN254Fr
        let left_elem = BN254Fr::from_be_bytes_mod_order(&left_input.borrow().0);
        let right_elem = BN254Fr::from_be_bytes_mod_order(&right_input.borrow().0);
        
        // Hash using Aptos's Poseidon with 2 inputs
        let hash_output = poseidon_bn254::hash_scalars(vec![left_elem, right_elem])
            .expect("Poseidon hash_scalars should not fail for valid BN254 inputs");
        
        // Serialize the output to 32 bytes
        let mut output_bytes = [0u8; 32];
        let mut serialized = Vec::new();
        hash_output.serialize_compressed(&mut serialized)?;
        
        let len = serialized.len().min(32);
        output_bytes[..len].copy_from_slice(&serialized[..len]);
        
        HashCounter::add();
        Ok(output_bytes.into())
    }

    fn compress<T: Borrow<Self::Output>>(
        parameters: &Self::Parameters,
        left_input: T,
        right_input: T,
    ) -> Result<Self::Output, Error> {
        Self::evaluate(parameters, left_input, right_input)
    }
}
