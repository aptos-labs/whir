<h1 align="center">WHIR 🌪️</h1>

This library was developed using the [arkworks](https://arkworks.rs) ecosystem to accompany [WHIR 🌪️](https://eprint.iacr.org/2024/1586). 
By [Gal Arnon](https://galarnon42.github.io/) [Alessandro Chiesa](https://ic-people.epfl.ch/~achiesa/), [Giacomo Fenzi](https://gfenzi.io), and [Eylon Yogev](https://www.eylonyogev.com/about).

**WARNING:** This is an academic prototype and has not received careful code review. This implementation is NOT ready for production use.

<p align="center">
    <a href="https://github.com/WizardOfMenlo/whir/blob/main/LICENSE-APACHE"><img src="https://img.shields.io/badge/license-APACHE-blue.svg"></a>
    <a href="https://github.com/WizardOfMenlo/whir/blob/main/LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
</p>

# Usage
```
cargo run --release -- --help

Usage: main [OPTIONS]

Options:
  -t, --type <PROTOCOL_TYPE>             [default: PCS]
  -l, --security-level <SECURITY_LEVEL>  [default: 100]
  -p, --pow-bits <POW_BITS>
  -d, --num-variables <NUM_VARIABLES>    [default: 20]
  -e, --evaluations <NUM_EVALUATIONS>    [default: 1]
  -r, --rate <RATE>                      [default: 1]
      --reps <VERIFIER_REPETITIONS>      [default: 1000]
  -k, --fold <FOLDING_FACTOR>            [default: 4]
      --sec <SOUNDNESS_TYPE>             [default: ConjectureList]
      --fold_type <FOLD_OPTIMISATION>    [default: ProverHelps]
  -f, --field <FIELD>                    [default: Goldilocks2]
      --hash <MERKLE_TREE>               [default: Blake3]
  -h, --help                             Print help
  -V, --version                          Print version
```

Options:
- `-t` can be either `PCS` or `LDT` to run as a (multilinear) PCS or a LDT
- `-l` sets the (overall) security level of the scheme
- `-p` sets the number of PoW bits (used for the query-phase). PoW bits for proximity gaps are set automatically.
- `-d` sets the number of variables of the scheme.
- `-e` sets the number of evaluations to prove. Only meaningful in PCS mode.
- `-r` sets the log_inv of the rate (e.g., `-r 3` means ρ=1/8)
- `-k` sets the number of variables to fold at each iteration. 
- `--sec` sets the settings used to compute security. Available `UniqueDecoding`, `ProvableList`, `ConjectureList`
- `--fold_type` sets the settings used to compute folds. Available `Naive`, `ProverHelps`
- `-f` sets the field used, available are `Goldilocks2, Goldilocks3, Field192, Field256`.
- `--hash` sets the hash used for the Merkle tree, available are `Blake3`, `Keccak`, `Poseidon` (Poseidon requires Field256/BN254)

## Aptos Poseidon Integration

This fork integrates **Aptos Poseidon** (`aptos-crypto::poseidon_bn254`) as a hash function option for WHIR:

- **Field Requirement**: Poseidon only works with `Field256` (BN254)
- **Compatibility**: Uses the same Poseidon implementation as Aptos keyless circuits (circomlib-compatible)
- **No Input Width Limits**: Supports arbitrary input sizes via automatic batching
- **Performance**: ~15-20× slower than Blake3, but ZK-friendly for recursion

### Example: Run WHIR with Poseidon
```bash
cargo run --release -- -t PCS -d 18 -r 3 -f Field256 --hash Poseidon --sec ConjectureList
```

## Benchmarking

### Native Hash Performance
Compare Blake3, Keccak256, and Poseidon performance:
```bash
cargo bench --bench hash_comparison
```

### WHIR Prover Benchmarks
Benchmark WHIR PCS across different instance sizes and hash functions:
```bash
# Blake3 (fastest)
cargo run --release --bin benchmark -- -d 18 -r 3 -f Field256 --hash Blake3 --sec ConjectureList

# Keccak256 (medium speed)
cargo run --release --bin benchmark -- -d 18 -r 3 -f Field256 --hash Keccak --sec ConjectureList

# Poseidon (ZK-friendly, slowest)
cargo run --release --bin benchmark -- -d 18 -r 3 -f Field256 --hash Poseidon --sec ConjectureList
```

**Note**: The `benchmark` binary is PCS-only and runs the prover/verifier multiple times for accurate timing.
