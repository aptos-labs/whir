use divan;
use blake3;
use sha3::{Digest, Keccak256};
use aptos_crypto::poseidon_bn254;
use ark_bn254::Fr as BN254Fr;
use ark_serialize::CanonicalSerialize;

fn main() {
    divan::main();
}

// Benchmark: Hash a long input of 2^13 = 8192 elements using Merkle tree approach
const INPUT_SIZE: usize = 8192; // 2^13
const ARITY_2: usize = 2; // Binary tree
const ARITY_4: usize = 4; // 4-ary tree

#[divan::bench]
fn blake3_merkle_arity2() {
    // Create 8192 field elements and serialize them to bytes (32 bytes each)
    let field_elements: Vec<BN254Fr> = (0..INPUT_SIZE)
        .map(|i| BN254Fr::from(i as u64))
        .collect();
    
    // Convert to 32-byte digests (leaves)
    let mut current_level: Vec<[u8; 32]> = field_elements
        .iter()
        .map(|elem| {
            let mut bytes = [0u8; 32];
            let mut serialized = Vec::new();
            elem.serialize_compressed(&mut serialized).unwrap();
            let len = serialized.len().min(32);
            bytes[..len].copy_from_slice(&serialized[..len]);
            bytes
        })
        .collect();
    
    // Build binary Merkle tree (arity 2)
    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        
        for chunk in current_level.chunks(ARITY_2) {
            let mut combined = Vec::new();
            for digest in chunk {
                combined.extend_from_slice(digest);
            }
            let hash_result: [u8; 32] = blake3::hash(&combined).into();
            next_level.push(hash_result);
        }
        
        current_level = next_level;
    }
    
    divan::black_box(current_level[0]);
}

#[divan::bench]
fn blake3_merkle_arity4() {
    // Create 8192 field elements and serialize them to bytes (32 bytes each)
    let field_elements: Vec<BN254Fr> = (0..INPUT_SIZE)
        .map(|i| BN254Fr::from(i as u64))
        .collect();
    
    // Convert to 32-byte digests (leaves)
    let mut current_level: Vec<[u8; 32]> = field_elements
        .iter()
        .map(|elem| {
            let mut bytes = [0u8; 32];
            let mut serialized = Vec::new();
            elem.serialize_compressed(&mut serialized).unwrap();
            let len = serialized.len().min(32);
            bytes[..len].copy_from_slice(&serialized[..len]);
            bytes
        })
        .collect();
    
    // Build Merkle tree with arity 4
    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        
        for chunk in current_level.chunks(ARITY_4) {
            let mut combined = Vec::new();
            for digest in chunk {
                combined.extend_from_slice(digest);
            }
            let hash_result: [u8; 32] = blake3::hash(&combined).into();
            next_level.push(hash_result);
        }
        
        current_level = next_level;
    }
    
    divan::black_box(current_level[0]);
}

#[divan::bench]
fn keccak256_merkle_arity2() {
    // Create 8192 field elements and serialize them to bytes (32 bytes each)
    let field_elements: Vec<BN254Fr> = (0..INPUT_SIZE)
        .map(|i| BN254Fr::from(i as u64))
        .collect();
    
    // Convert to 32-byte digests (leaves)
    let mut current_level: Vec<[u8; 32]> = field_elements
        .iter()
        .map(|elem| {
            let mut bytes = [0u8; 32];
            let mut serialized = Vec::new();
            elem.serialize_compressed(&mut serialized).unwrap();
            let len = serialized.len().min(32);
            bytes[..len].copy_from_slice(&serialized[..len]);
            bytes
        })
        .collect();
    
    // Build binary Merkle tree (arity 2)
    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        
        for chunk in current_level.chunks(ARITY_2) {
            let mut combined = Vec::new();
            for digest in chunk {
                combined.extend_from_slice(digest);
            }
            let hash_result: [u8; 32] = Keccak256::digest(&combined).into();
            next_level.push(hash_result);
        }
        
        current_level = next_level;
    }
    
    divan::black_box(current_level[0]);
}

#[divan::bench]
fn keccak256_merkle_arity4() {
    // Create 8192 field elements and serialize them to bytes (32 bytes each)
    let field_elements: Vec<BN254Fr> = (0..INPUT_SIZE)
        .map(|i| BN254Fr::from(i as u64))
        .collect();
    
    // Convert to 32-byte digests (leaves)
    let mut current_level: Vec<[u8; 32]> = field_elements
        .iter()
        .map(|elem| {
            let mut bytes = [0u8; 32];
            let mut serialized = Vec::new();
            elem.serialize_compressed(&mut serialized).unwrap();
            let len = serialized.len().min(32);
            bytes[..len].copy_from_slice(&serialized[..len]);
            bytes
        })
        .collect();
    
    // Build Merkle tree with arity 4
    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        
        for chunk in current_level.chunks(ARITY_4) {
            let mut combined = Vec::new();
            for digest in chunk {
                combined.extend_from_slice(digest);
            }
            let hash_result: [u8; 32] = Keccak256::digest(&combined).into();
            next_level.push(hash_result);
        }
        
        current_level = next_level;
    }
    
    divan::black_box(current_level[0]);
}

#[divan::bench]
fn poseidon_merkle_arity2() {
    // Create 8192 field elements
    let field_elements: Vec<BN254Fr> = (0..INPUT_SIZE)
        .map(|i| BN254Fr::from(i as u64))
        .collect();
    
    // Build binary Merkle tree using Poseidon(2)
    let mut current_level = field_elements.clone();
    
    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        
        for chunk in current_level.chunks(ARITY_2) {
            let hash_result = poseidon_bn254::hash_scalars(chunk.to_vec()).unwrap();
            next_level.push(hash_result);
        }
        
        current_level = next_level;
    }
    
    divan::black_box(current_level[0]);
}

#[divan::bench]
fn poseidon_merkle_arity4() {
    // Create 8192 field elements
    let field_elements: Vec<BN254Fr> = (0..INPUT_SIZE)
        .map(|i| BN254Fr::from(i as u64))
        .collect();
    
    // Build Merkle tree with arity 4 using Poseidon(4)
    let mut current_level = field_elements.clone();
    
    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        
        for chunk in current_level.chunks(ARITY_4) {
            let hash_result = poseidon_bn254::hash_scalars(chunk.to_vec()).unwrap();
            next_level.push(hash_result);
        }
        
        current_level = next_level;
    }
    
    divan::black_box(current_level[0]);
}

