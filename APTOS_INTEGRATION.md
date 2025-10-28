# Aptos WHIR Integration - Quick Start

This fork of WHIR integrates **Aptos Poseidon** (`aptos-crypto::poseidon_bn254`) and **Keccak256** as additional hash function options for the Merkle commitment scheme.

## What's New

### Hash Functions
- **Blake3** (original) - Fastest, ~1.5ms for 8K elements (arity 4)
- **Keccak256** (new) - Medium speed, ~1.9ms for 8K elements (arity 4)
- **Poseidon** (new) - ZK-friendly, ~67ms for 8K elements (arity 4), compatible with Aptos keyless circuits

### Key Features
- **Field Requirement**: Poseidon requires `Field256` (BN254)
- **No Input Width Limits**: Supports arbitrary input sizes via automatic batching
- **Circomlib Compatible**: Uses same Poseidon as Aptos keyless circuits
- **Performance**: ~15-20× slower than Blake3, but enables efficient ZK recursion

## Quick Test

### 1. Build the project
```bash
cargo build --release
```

### 2. Run WHIR with Poseidon
```bash
cargo run --release -- -t PCS -d 18 -r 3 -f Field256 --hash Poseidon --sec ConjectureList
```

Expected output:
```
Field: Field256 and MT: Poseidon
Number of variables: 18, folding factor: ConstantFromSecondRound(4, 4)
Security level: 100 bits using ConjectureList security and 18 bits of PoW
...
Prover time: ~4.4s
Proof size: ~57.6 KiB
Verifier time: ~19ms
```

### 3. Compare hash functions
```bash
# Native hash performance (10K hashes)
cargo bench --bench hash_comparison

# WHIR prover benchmarks
cargo run --release --bin benchmark -- -d 18 -r 3 -f Field256 --hash Blake3 --sec ConjectureList
cargo run --release --bin benchmark -- -d 18 -r 3 -f Field256 --hash Keccak --sec ConjectureList
cargo run --release --bin benchmark -- -d 18 -r 3 -f Field256 --hash Poseidon --sec ConjectureList
```

## Benchmark Results Summary

### Instance 2^18 (ρ=1/8, BN254, CB mode)

| Hash Function | Prover Time | Proof Size | Verifier Time | Verifier Hashes |
|--------------|-------------|------------|---------------|-----------------|
| Blake3       | 0.75s       | 57.6 KiB   | 1.9ms         | 780             |
| Keccak256    | 0.83s       | 57.6 KiB   | 3.0ms         | 780             |
| Poseidon     | 4.4s        | 57.6 KiB   | 19ms          | 780             |

**Key Insight**: Poseidon is ~6× slower for proving but enables efficient ZK recursion. When recursing the verifier in Groth16:
- 780 Poseidon(2) hashes → ~12s proving time (snarkjs, M2 Max, 1 thread)
- This is **3× faster** than proving the baseline Aptos keyless circuit (36.7s)

## Implementation Details

### Modified Files
1. **`src/crypto/merkle_tree/poseidon.rs`** - Aptos Poseidon integration
2. **`src/crypto/merkle_tree/keccak.rs`** - Keccak256 integration
3. **`Cargo.toml`** - Added `aptos-crypto` dependency
4. **`benches/hash_comparison.rs`** - Hash performance benchmarks

### Dependencies
```toml
aptos-crypto = { git = "https://github.com/aptos-labs/aptos-core", branch = "main" }
```

## Related Work

- **WHIR Verifier Recursion**: See `aptos-labs/research/cryptography/whir_verifier_recursion/` for Circom circuits simulating WHIR verifier in Groth16
- **Original WHIR**: [WizardOfMenlo/whir](https://github.com/WizardOfMenlo/whir)
- **WHIR Paper**: [eprint.iacr.org/2024/1586](https://eprint.iacr.org/2024/1586)

## Notes

- This is an **academic prototype** - not production ready
- Poseidon integration is specific to BN254 field (Field256)
- All benchmarks performed on Apple M2 Max
- Security level: 100 bits (ConjectureList mode)

