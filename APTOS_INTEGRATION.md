# Aptos WHIR Integration - Quick Start

This fork of WHIR integrates **Aptos Poseidon** (`aptos-crypto::poseidon_bn254`) and **Keccak256** as additional hash function options for the Merkle commitment scheme.

## What's New

### Hash Functions
- **Blake3** (original) - Fastest native performance
- **Keccak256** (new) - Medium speed, standard cryptographic hash
- **Poseidon** (new) - ZK-friendly, compatible with Aptos keyless circuits

### Key Features
- **Field Requirement**: Poseidon requires `Field256` (BN254)
- **No Input Width Limits**: Supports arbitrary input sizes via automatic batching
- **Circomlib Compatible**: Uses same Poseidon as Aptos keyless circuits
- **Performance**: ~15-20× slower than Blake3, but enables efficient ZK recursion

## Quick Test

### 1. Run single instance with Poseidon
```bash
cargo run --release -- -t PCS -d 18 -r 3 -f Field256 --hash Poseidon --sec ConjectureList -l 128
```

Expected output:
```
Field: Field256 and MT: Poseidon
Number of variables: 18, folding factor: ConstantFromSecondRound(4, 4)
Security level: 128 bits using ConjectureList security
...
Prover time: ~4.4s
Proof size: ~57.6 KiB
Verifier time: ~19ms
```

### 2. Run comprehensive benchmark (all instances + hash functions)
```bash
./benchmark_whir_pcs.sh
```

This script benchmarks:
- **Instance sizes**: 2^18, 2^19, 2^20, 2^21
- **Hash functions**: Blake3, Keccak256, Poseidon
- **Configuration**: BN254, ρ=1/8, 128-bit security, single-threaded
- **Output**: Results saved to `benchmark_results_pcs/`

**Runtime**: ~5-10 minutes for all 12 combinations

## Benchmark Results Summary

### Instance 2^18 (ρ=1/8, BN254, 128-bit security, 1 thread)

| Hash Function | Prover Time | Proof Size | Verifier Time | Verifier Hashes |
|--------------|-------------|------------|---------------|-----------------|
| Blake3       | 0.75s       | 57.6 KiB   | 1.9ms         | 780             |
| Keccak256    | 0.83s       | 57.6 KiB   | 3.0ms         | 780             |
| Poseidon     | 4.4s        | 57.6 KiB   | 19ms          | 780             |

**Key Insight**: Poseidon is ~6× slower for proving but enables efficient ZK recursion. When recursing the verifier in Groth16:
- 780 Poseidon(2) hashes → ~12s proving time (snarkjs, M2 Max, 1 thread)
- This is **3× faster** than proving the baseline Aptos keyless circuit (36.7s)

For complete results across all instance sizes, run `./benchmark_whir_pcs.sh`

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

