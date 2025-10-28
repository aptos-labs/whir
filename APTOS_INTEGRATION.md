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

### Benchmark Configuration

**Machine:**
- Apple MacBook Pro M2 Max
- macOS 25.0.0 (Darwin)
- Chip: Apple M2 Max (12 CPU cores: 8 performance + 4 efficiency)

**Threading:**
- **Single-threaded execution** (`RAYON_NUM_THREADS=1`)
- Forces deterministic, reproducible results
- All cores available but only 1 thread used per benchmark

**Software:**
- Rust compiler: 1.83.0
- Build mode: `--release` (optimized)
- WHIR configuration: ConjectureList soundness

### Complete Results (BN254, ρ=1/8, 128-bit security, 1 thread)

| Instance | Hash Function | Prover Time | Proof Size | Verifier Time | Verifier Hashes |
|----------|--------------|-------------|------------|---------------|-----------------|
| 2^18     | Blake3       | 1.3s        | 75.1 KiB   | 649µs         | 1.0k            |
| 2^18     | Keccak256    | 1.4s        | 75.2 KiB   | 800µs         | 1.0k            |
| 2^18     | Poseidon     | 36.0s       | 74.7 KiB   | 23.7ms        | 1.0k            |
| 2^19     | Blake3       | 2.7s        | 78.1 KiB   | 697µs         | 1.1k            |
| 2^19     | Keccak256    | 2.9s        | 77.8 KiB   | 872µs         | 1.1k            |
| 2^19     | Poseidon     | 71.6s       | 78.4 KiB   | 24.5ms        | 1.1k            |
| 2^20     | Blake3       | 6.0s        | 81.7 KiB   | 762µs         | 1.2k            |
| 2^20     | Keccak256    | 6.5s        | 82.0 KiB   | 953µs         | 1.2k            |
| 2^20     | Poseidon     | 148.7s      | 81.7 KiB   | 27.8ms        | 1.2k            |
| 2^21     | Blake3       | 13.7s       | 87.4 KiB   | 804µs         | 1.3k            |
| 2^21     | Keccak256    | 14.3s       | 86.8 KiB   | 1.0ms         | 1.3k            |
| 2^21     | Poseidon     | 299.3s      | 87.0 KiB   | 28.9ms        | 1.3k            |

**Key Observations:**
- **Poseidon prover** is ~25-30× slower than Blake3, but enables efficient ZK recursion
- **Poseidon verifier** is ~35-40× slower than Blake3 (~28ms vs ~0.8ms for 2^21)
- **Proof sizes** are nearly identical across all hash functions (~75-87 KiB)
- **Verifier hash count** scales linearly with instance size (1.0k → 1.3k for 2^18 → 2^21)

**ZK Recursion Efficiency:**
- For 2^18: 1,000 Poseidon(2) hashes → ~12s Groth16 proving time (snarkjs, M2 Max, 1 thread)
- This is **3× faster** than proving the baseline Aptos keyless circuit (36.7s)
- Demonstrates Poseidon's advantage for recursive proof systems

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

- **WHIR Verifier Recursion**: See [aptos-labs/research/cryptography/whir_verifier_recursion/](https://github.com/aptos-labs/research/cryptography/whir_verifier_recursion/) for Circom circuits simulating WHIR verifier in Groth16
- **Original WHIR**: [WizardOfMenlo/whir](https://github.com/WizardOfMenlo/whir)
- **WHIR Paper**: [eprint.iacr.org/2024/1586](https://eprint.iacr.org/2024/1586)

## Notes

- This is an **academic prototype** - not production ready
- Poseidon integration is specific to BN254 field (Field256)
- See **Benchmark Configuration** section above for complete hardware/software details

