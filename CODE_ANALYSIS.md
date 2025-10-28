# WHIR Repository: Code-to-Protocol Mapping

**Analysis Date**: October 24, 2024  
**Purpose**: Understand how the WHIR implementation maps to the paper's protocol for benchmarking against Groth16

---

## 🎯 Executive Summary

### **Key Discovery: This is a PCS/LDT implementation, NOT an R1CS prover!**

The WHIR repository implements **Layer 3 (WHIR IOPP)** and **Layer 4 (BCS transformation)** from the paper, but it does **NOT** implement **Layer 1 (R1CS → Σ-IOP)** or **Layer 2 (IOP Compiler)**. This means:

❌ **The repository cannot directly prove R1CS circuits like Aptos keyless**  
✅ **The repository can prove polynomial evaluation constraints (PCS mode)**  
✅ **The repository can test low-degree polynomials (LDT mode)**

---

## 1. Architecture: What WHIR Repo Actually Does

```
┌─────────────────────────────────────────────────┐
│ INPUT: Multilinear Polynomial (CoefficientList) │
│        + Evaluation Constraints (Statement)      │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│ CommitmentWriter::commit()                      │
│ • Reed-Solomon encode polynomial                │
│ • Build Merkle tree commitment                  │
│ • Generate out-of-domain samples                │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│ Prover::prove() [WHIR IOPP]                     │
│ • Iterative folding (M rounds)                  │
│ • Sumcheck protocol per round                   │
│ • Query answering via Merkle proofs             │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│ BCS Transformation (spongefish)                 │
│ • Fiat-Shamir via hash transcript               │
│ • Output: Non-interactive proof                 │
└─────────────────────────────────────────────────┘
```

**Comparison with Full WHIR Stack (from paper)**:
```
R1CS → Σ-IOP → IOP Compiler → WHIR IOPP → BCS → SNARK
        ❌         ❌              ✅         ✅
```

---

## 2. Code-to-Protocol Mapping

### **Layer 3: WHIR IOPP Implementation**

| Protocol Component | Code Location | Key Methods |
|-------------------|---------------|-------------|
| **Polynomial Representation** | `poly_utils/coeffs.rs` | `CoefficientList`, `fold()` |
| **Weights (Constraints)** | `whir/statement.rs` | `Weights::evaluation()`, `Weights::linear()` |
| **Statement (Constraint System)** | `whir/statement.rs` | `Statement::new()`, `Statement::add_constraint()`, `Statement::combine()` |
| **Sumcheck Prover** | `sumcheck/sumcheck_single.rs` | `SumcheckSingle::compute_sumcheck_polynomial()`, `compute_sumcheck_polynomials()` |
| **Folding Operation** | `poly_utils/fold.rs` | `compute_fold()` |
| **Merkle Commitment** | `whir/committer/` | `CommitmentWriter::commit()`, `CommitmentReader::parse_commitment()` |
| **WHIR Prover** | `whir/prover.rs` | `Prover::prove()`, `Prover::round()`, `Prover::final_round()` |
| **WHIR Verifier** | `whir/verifier.rs` | `Verifier::verify()` |
| **Parameter Calculation** | `whir/parameters.rs` | `WhirConfig::new()`, `queries()`, `ood_samples()`, `folding_pow_bits()` |

---

## 3. Detailed Code Method Analysis

### **3.1 Prover::prove() - The Main WHIR Loop**

**File**: `src/whir/prover.rs:88-190`

**Maps to**: Construction 5.1 (WHIR IOPP) in the paper

**What it does**:
1. **Lines 106-120**: Convert out-of-domain points to constraints (out-of-domain binding)
   ```rust
   let new_constraints = witness.ood_points
       .into_iter()
       .zip(witness.ood_answers)
       .map(|(point, evaluation)| {
           let weights = Weights::evaluation(MultilinearPoint::expand_from_univariate(point, ...));
           (weights, evaluation)
       })
       .collect();
   ```

2. **Lines 122-153**: Run initial sumcheck if `initial_statement=true` (PCS mode)
   ```rust
   if self.config.initial_statement {
       let [combination_randomness_gen] = prover_state.challenge_scalars()?;
       let mut sumcheck = SumcheckSingle::new(witness.polynomial.clone(), &statement, ...);
       let folding_randomness = sumcheck.compute_sumcheck_polynomials::<PowStrategy, _>(...)?;
       sumcheck_prover = Some(sumcheck);
   }
   ```

3. **Lines 172-174**: Iterate through M rounds
   ```rust
   for _round in 0..=self.config.n_rounds() {
       self.round(prover_state, &mut round_state)?;
   }
   ```

4. **Lines 177-189**: Handle deferred constraints (hints for verifier)

---

### **3.2 Prover::round() - One WHIR Iteration**

**File**: `src/whir/prover.rs:194-371`

**Maps to**: One iteration of Construction 5.1

**What it does**:

1. **Lines 208-210**: Fold polynomial
   ```rust
   let folded_coefficients = round_state.coefficients.fold(&round_state.folding_randomness);
   ```
   - Reduces polynomial from `m` variables to `m-k` variables
   - Uses folding randomness from previous sumcheck

2. **Lines 229-247**: RS-encode and commit to folded polynomial
   ```rust
   let evals = interleaved_rs_encode(folded_coefficients.coeffs(), expansion, folding_factor_next);
   let merkle_tree = MerkleTree::new(&self.config.leaf_hash_params, &self.config.two_to_one_params, leafs_iter)?;
   let root = merkle_tree.root();
   prover_state.add_digest(root)?;
   ```
   - Expands to Reed-Solomon codeword
   - Builds Merkle tree over codeword
   - Sends Merkle root to verifier

3. **Lines 253-258**: Sample out-of-domain points and evaluate
   ```rust
   let (ood_points, ood_answers) = sample_ood_points(
       prover_state,
       round_params.ood_samples,
       num_variables,
       |point| folded_coefficients.evaluate(point),
   )?;
   ```
   - Verifier samples random point outside domain
   - Prover evaluates folded polynomial at that point

4. **Lines 272-279**: Compute STIR queries (shift queries)
   ```rust
   let (stir_challenges, stir_challenges_indexes) = self.compute_stir_queries(
       prover_state, round_state, num_variables, round_params, ood_points
   )?;
   ```
   - Verifier samples query positions in previous domain
   - These will be used to check folding consistency

5. **Lines 287-312**: Answer queries via Merkle proof + folding
   ```rust
   let mut answers: Vec<_> = stir_challenges_indexes
       .iter()
       .map(|i| round_state.prev_merkle_answers[i * leaf_size..(i + 1) * leaf_size].to_vec())
       .collect();
   
   prover_state.hint::<Vec<Vec<F>>>(&answers)?;
   self.merkle_state.write_proof_hint(&round_state.prev_merkle, &stir_challenges_indexes, prover_state)?;
   ```
   - Opens Merkle tree at queried positions
   - Provides folding witnesses (2^k values per query)

6. **Lines 314-348**: Run sumcheck prover for k rounds
   ```rust
   let [combination_randomness_gen] = prover_state.challenge_scalars()?;
   let combination_randomness = expand_randomness(combination_randomness_gen, stir_challenges.len());
   
   let mut sumcheck_prover = ...;
   let folding_randomness = sumcheck_prover.compute_sumcheck_polynomials::<PowStrategy, _>(
       prover_state, folding_factor_next, round_params.folding_pow_bits
   )?;
   ```
   - Batch all constraints (OOD + STIR queries) using random combination
   - Run sumcheck to reduce to next folding randomness

7. **Lines 361-368**: Update state for next round
   ```rust
   round_state.round += 1;
   round_state.domain = new_domain;
   round_state.sumcheck_prover = Some(sumcheck_prover);
   round_state.folding_randomness = folding_randomness;
   round_state.coefficients = folded_coefficients;
   round_state.prev_merkle = merkle_tree;
   round_state.prev_merkle_answers = evals;
   ```

---

### **3.3 SumcheckSingle::compute_sumcheck_polynomials() - Sumcheck Protocol**

**File**: `src/sumcheck/sumcheck_single.rs:162-193`

**Maps to**: Sumcheck protocol within each WHIR iteration

**What it does**:
```rust
pub fn compute_sumcheck_polynomials<S, ProverState>(
    &mut self,
    prover_state: &mut ProverState,
    folding_factor: usize,
    pow_bits: f64,
) -> ProofResult<MultilinearPoint<F>>
{
    let mut res = Vec::with_capacity(folding_factor);
    
    for _ in 0..folding_factor {
        // 1. Compute univariate sumcheck polynomial h(X)
        let sumcheck_poly = self.compute_sumcheck_polynomial();
        prover_state.add_scalars(sumcheck_poly.evaluations())?;
        
        // 2. Perform PoW grinding if needed
        if pow_bits > 0. {
            prover_state.challenge_pow::<S>(pow_bits)?;
        }
        
        // 3. Sample folding randomness α
        let [folding_randomness] = prover_state.challenge_scalars()?;
        res.push(folding_randomness);
        
        // 4. Compress polynomial by binding one variable
        self.compress(F::ONE, &folding_randomness.into(), &sumcheck_poly);
    }
    
    res.reverse();
    Ok(MultilinearPoint(res))
}
```

**Key insight**: Each sumcheck round reduces the polynomial by 1 variable. After `k` rounds, we've reduced by `k` variables (the folding factor).

---

### **3.4 compute_fold() - Polynomial Folding**

**File**: `src/poly_utils/fold.rs:29-68`

**Maps to**: Folding operation in WHIR (reducing polynomial from m variables to m-k variables)

**Formula implemented**:
```rust
g_i = (f_i + f_{i+N/2} + r·(f_i - f_{i+N/2})·ω^{-i}) / 2
```

**What it does**:
```rust
pub fn compute_fold<F: Field>(
    answers: &[F],
    folding_randomness: &[F],
    mut coset_offset_inv: F,
    mut coset_gen_inv: F,
    two_inv: F,
    folding_factor: usize,
) -> F {
    let mut answers = answers.to_vec();
    
    // Perform the folding process `folding_factor` times
    for rec in 0..folding_factor {
        let r = folding_randomness[folding_randomness.len() - 1 - rec];
        let offset = answers.len() / 2;
        let mut coset_index_inv = F::ONE;
        
        // Fold: combine pairs (f_i, f_{i+N/2})
        for i in 0..offset {
            let f0 = answers[i];
            let f1 = answers[i + offset];
            let point_inv = coset_offset_inv * coset_index_inv;
            
            let left = f0 + f1;
            let right = point_inv * (f0 - f1);
            
            // Apply folding transformation with randomness
            answers[i] = two_inv * (left + r * right);
            coset_index_inv *= coset_gen_inv;
        }
        
        answers.truncate(offset);  // Reduce to half size
        
        // Update for next iteration
        coset_offset_inv *= coset_offset_inv;
        coset_gen_inv *= coset_gen_inv;
    }
    
    answers[0]
}
```

---

### **3.5 Statement::combine() - Batching Constraints**

**File**: `src/whir/statement.rs:325-340`

**Maps to**: Random linear combination of constraints (Construction 7.4, step 5)

**What it does**:
```rust
pub fn combine(&self, challenge: F) -> (EvaluationsList<F>, F) {
    let evaluations_vec = vec![F::ZERO; 1 << self.num_variables];
    let mut combined_evals = EvaluationsList::new(evaluations_vec);
    
    let (combined_sum, _) = self.constraints.iter().fold(
        (F::ZERO, F::ONE),
        |(mut acc_sum, gamma_pow), constraint| {
            // Accumulate: W(X) += γ^i · w_i(X)
            constraint.weights.accumulate(&mut combined_evals, gamma_pow);
            
            // Accumulate: S += γ^i · s_i
            acc_sum += constraint.sum * gamma_pow;
            
            (acc_sum, gamma_pow * challenge)
        },
    );
    
    (combined_evals, combined_sum)
}
```

**Formula**: 
- `W(X) = w₁(X) + γ·w₂(X) + γ²·w₃(X) + ...`
- `S = s₁ + γ·s₂ + γ²·s₃ + ...`

---

## 4. Parameter Setting (WHIR-CB Configuration)

### **WhirConfig::new() - Parameter Derivation**

**File**: `src/whir/parameters.rs:107-282`

**Maps to**: Security analysis from WHIR paper (Section 6)

**Key formulas implemented**:

#### **4.1 `log_eta()` - Proximity Parameter** (Lines 305-312)
```rust
pub const fn log_eta(soundness_type: SoundnessType, log_inv_rate: usize) -> f64 {
    match soundness_type {
        SoundnessType::ProvableList => -(0.5 * log_inv_rate as f64 + LOG2_10 + 1.),
        SoundnessType::UniqueDecoding => 0.,
        SoundnessType::ConjectureList => -(log_inv_rate as f64 + 1.),  // ✅ WHIR-CB
    }
}
```

**Maps to**: Proximity parameter δ:
- **UniqueDecoding**: `δ = (1-ρ)/2`
- **ProvableList**: `δ = 1-√ρ-η`
- **ConjectureList**: `δ = 1-ρ-η` ✅ **WHIR-CB: Most aggressive (best performance)**

---

#### **4.2 `queries()` - Number of Queries** (Lines 441-459)
```rust
pub fn queries(
    soundness_type: SoundnessType,
    protocol_security_level: usize,
    log_inv_rate: usize,
) -> usize {
    let num_queries_f = match soundness_type {
        SoundnessType::UniqueDecoding => {
            let rate = 1. / f64::from(1 << log_inv_rate);
            let denom = (0.5 * (1. + rate)).log2();
            -(protocol_security_level as f64) / denom
        }
        SoundnessType::ProvableList => {
            (2 * protocol_security_level) as f64 / log_inv_rate as f64
        }
        SoundnessType::ConjectureList => {
            protocol_security_level as f64 / log_inv_rate as f64  // ✅ WHIR-CB: Fewest queries!
        }
    };
    num_queries_f.ceil() as usize
}
```

**Example**: For λ=128, log_inv_rate=4:
- **UniqueDecoding**: ~300 queries
- **ProvableList**: 64 queries
- **ConjectureList**: 32 queries ✅ **WHIR-CB: 2-10× fewer queries**

---

#### **4.3 `ood_samples()` - Out-of-Domain Samples** (Lines 345-368)
```rust
pub fn ood_samples(
    security_level: usize,
    soundness_type: SoundnessType,
    num_variables: usize,
    log_inv_rate: usize,
    log_eta: f64,
    field_size_bits: usize,
) -> usize {
    match soundness_type {
        SoundnessType::UniqueDecoding => 0,  // No OOD samples needed
        _ => (1..64)
            .find(|&ood_samples| {
                Self::rbr_ood_sample(
                    soundness_type, num_variables, log_inv_rate,
                    log_eta, field_size_bits, ood_samples
                ) >= security_level as f64
            })
            .unwrap_or_else(|| panic!("Could not find appropriate number of OOD samples"))
    }
}
```

**What it does**: Finds minimum number of out-of-domain samples needed to achieve target security level based on list decoding bound analysis.

---

#### **4.4 `folding_pow_bits()` - PoW Bits for Folding** (Lines 402-437)
```rust
pub const fn folding_pow_bits(
    security_level: usize,
    soundness_type: SoundnessType,
    field_size_bits: usize,
    num_variables: usize,
    log_inv_rate: usize,
    log_eta: f64,
) -> f64 {
    let prox_gaps_error = Self::rbr_soundness_fold_prox_gaps(
        soundness_type, field_size_bits, num_variables, log_inv_rate, log_eta
    );
    let sumcheck_error = Self::rbr_soundness_fold_sumcheck(
        soundness_type, field_size_bits, num_variables, log_inv_rate, log_eta
    );
    
    let error = if prox_gaps_error < sumcheck_error {
        prox_gaps_error
    } else {
        sumcheck_error
    };
    
    let candidate = security_level as f64 - error;
    if candidate > 0_f64 { candidate } else { 0_f64 }
}
```

**What it does**: Computes PoW bits needed to compensate for soundness gaps:
- **Proximity gaps error**: From approximate folding
- **Sumcheck error**: From field size limitations
- **PoW bits**: `max(0, λ - min(prox_gaps_error, sumcheck_error))`

---

## 5. Layer 4: BCS Transformation

### **Merkle Tree Commitment**

**File**: `src/crypto/merkle_tree/`

**Components**:
- **Blake3**: `blake3.rs` - Fast hash function (default)
- **Keccak**: `keccak.rs` - Standard cryptographic hash
- **Merkle Tree**: Arkworks' `merkle_tree` module

**Usage in WHIR**:
```rust
// Build Merkle tree over polynomial evaluations
let merkle_tree = MerkleTree::new(
    &self.config.leaf_hash_params,
    &self.config.two_to_one_params,
    leafs_iter,  // Iterator over [F; 2^k] chunks
).unwrap();

// Commit by sending root
let root = merkle_tree.root();
prover_state.add_digest(root)?;

// Open at queried positions
self.merkle_state.write_proof_hint(
    &merkle_tree,
    &query_indexes,
    prover_state
)?;
```

---

### **Fiat-Shamir Transcript**

**External crate**: `spongefish` and `spongefish-pow`

**What it does**:
- **ProverState**: Records all prover messages (field elements, digests, hints)
- **VerifierState**: Replays transcript and samples challenges deterministically
- **Challenge sampling**: Hash transcript → derive random challenges
- **PoW grinding**: Optionally grind for proof-of-work challenges

**Usage**:
```rust
// Initialize transcript with domain separator
let domainsep = DomainSeparator::new("🌪️")
    .commit_statement(&params)
    .add_whir_proof(&params);

let mut prover_state = domainsep.to_prover_state();

// Prover adds messages
prover_state.add_scalars(&field_elements)?;  // Add field elements
prover_state.add_digest(merkle_root)?;       // Add Merkle root

// Prover samples challenges
let [challenge] = prover_state.challenge_scalars()?;

// Prover grinds PoW
prover_state.challenge_pow::<Blake3PoW>(pow_bits)?;

// Verifier replays
let mut verifier_state = domainsep.to_verifier_state(prover_state.narg_string());
```

---

## 6. PCS vs LDT Mode

### **PCS Mode** (`initial_statement=true`)

**File**: `src/bin/main.rs:329-470` (`run_whir_pcs()`)

**Use case**: Prove polynomial evaluation constraints

**Example**:
- Prove `f(z₁) = y₁`, `f(z₂) = y₂`, ...
- Prove `Σ w_i·f(i) = s` (linear constraint)

**Code**:
```rust
let whir_params = ProtocolParameters {
    initial_statement: true,  // ✅ PCS mode
    // ...
};

let mut statement: Statement<F> = Statement::new(num_variables);

// Add evaluation constraints
for point in &points {
    let eval = polynomial.evaluate_at_extension(point);
    let weights = Weights::evaluation(point.clone());
    statement.add_constraint(weights, eval);
}

// Add linear constraints
let linear_claim_weight = Weights::linear(input.clone().into());
let sum = linear_claim_weight.weighted_sum(&poly);
statement.add_constraint(linear_claim_weight, sum);

prover.prove(&mut prover_state, statement, witness).unwrap();
```

---

### **LDT Mode** (`initial_statement=false`)

**File**: `src/bin/main.rs:215-326` (`run_whir_as_ldt()`)

**Use case**: Just test low-degree property (no evaluation constraints)

**Example**: Prove `f` is a degree-d polynomial

**Code**:
```rust
let whir_params = ProtocolParameters {
    initial_statement: false,  // ✅ LDT mode
    // ...
};

let statement = Statement::new(num_variables);  // Empty statement

prover.prove(&mut prover_state, statement, witness).unwrap();
```

---

## 7. CLI Usage

**File**: `src/bin/main.rs`

**Example command**:
```bash
cargo run --release -- \
  --type PCS \                    # or LDT
  --security-level 128 \
  --num-variables 21 \            # 2^21 ≈ 2M (for 1.4M constraints)
  --evaluations 1 \               # Number of evaluation constraints
  --rate 1 \                      # log_inv_rate (rate = 2^-1 = 0.5)
  --fold 4 \                      # Folding factor (k)
  --sec ConjectureList \          # WHIR-CB (best performance)
  --field Goldilocks2 \           # 64-bit field with 2-element extension
  --hash Blake3                   # Blake3 Merkle tree
```

**Output**:
```
Whir (PCS) 🌪️
Field: Goldilocks2 and MT: Blake3
Security level: 128 bits using ConjectureList security and 20 bits of PoW
...
Prover time: 3.9s
Proof size: 156.0 KiB
Verifier time: 1.4ms
Average hashes: 2.7k
```

---

## 8. 🚨 Critical Gap: Missing R1CS Support

### **What's Missing**:

1. **R1CS → GR1CS Σ-IOP** (Construction A.2 in paper)
   - Convert R1CS constraint system to polynomial form
   - Encode witness as multilinear polynomial
   - Generate sumcheck queries for constraint satisfaction

2. **IOP Compiler** (Construction 7.4 in paper)
   - Convert Σ-IOP (with sumcheck queries) to IOP (with polynomial oracles)
   - Handle out-of-domain binding
   - Batch constraints into single WHIR IOPP call

3. **Circuit witness → multilinear polynomial conversion**
   - Map circuit variables to boolean hypercube {0,1}^m
   - Interpolate multilinear polynomial

4. **R1CS constraint encoding**
   - Encode `(Aw) ∘ (Bw) = Cw` as polynomial constraints
   - Handle custom gate types (if any)

---

### **What This Means for Aptos Keyless**:

❌ **Cannot directly use WHIR repo** - It expects:
- Input: `CoefficientList` (multilinear polynomial)
- Constraints: `Statement` (evaluation + linear constraints)

But Aptos keyless circuit provides:
- Input: R1CS matrices (A, B, C)
- Witness: Assignment to circuit variables
- Public inputs: JWT fields, identity commitment, etc.

---

### **Gap Analysis**:

```
┌─────────────────────────────────────────────────────────┐
│ What we have: Circom R1CS circuit (keyless.circom)     │
│ • 1.4M constraints                                       │
│ • Witness: JWT, ephemeral key, pepper, etc.            │
│ • R1CS: (Aw) ∘ (Bw) = Cw                               │
└─────────────────────────────────────────────────────────┘
                         ↓
                    ❌ MISSING ❌
         (R1CS → Multilinear Polynomial)
                         ↓
┌─────────────────────────────────────────────────────────┐
│ What WHIR expects: Multilinear polynomial + Statement   │
│ • CoefficientList<F>                                    │
│ • Statement with evaluation/linear constraints          │
└─────────────────────────────────────────────────────────┘
```

---

## 9. Three Paths Forward

### **Option 1: Build R1CS Frontend (Hard)**

**Estimate**: 2-4 weeks of work

**Tasks**:
1. Implement GR1CS Σ-IOP (Appendix A.2 from paper)
   - ~500-1000 lines of Rust
   - Complex polynomial arithmetic
   - Careful handling of public/private inputs

2. Implement IOP Compiler (Construction 7.4)
   - ~300-500 lines of Rust
   - Batching and combination logic
   - Out-of-domain binding

3. Convert Circom R1CS to multilinear polynomial
   - Parse `.r1cs` file format
   - Map witness to polynomial coefficients
   - Handle variable ordering

4. Testing and debugging
   - Verify against small test circuits
   - Check soundness on invalid witnesses

**Pros**:
- Full control over implementation
- Can optimize for Aptos keyless specifically
- Learning opportunity

**Cons**:
- Significant time investment
- High complexity (easy to introduce bugs)
- Maintenance burden

---

### **Option 2: Use Different STARK System (Easier)**

**Estimate**: 1-2 weeks to integrate

**Options**:
- **Plonky3** with custom PCS
  - Modern STARK framework
  - Supports custom polynomial commitment schemes
  - Could integrate WHIR as backend
  - But: Uses FRI by default, not clear if WHIR is compatible

- **Binius**
  - Very fast binary field STARK
  - But: Only supports binary fields (not BN254 for Aptos)

- **Risc0**
  - Production-ready STARK system
  - But: Uses FRI, not WHIR

**Pros**:
- Faster to get working
- Production-tested code
- Active community support

**Cons**:
- May not support WHIR directly
- Less control over parameters
- May not match paper's exact protocol

---

### **Option 3: Benchmark WHIR PCS Directly (Immediate)**

**Estimate**: 1-2 days

**Goal**: Understand WHIR performance characteristics without full R1CS support

**Approach**:
1. Create simple multilinear polynomial with ~21 variables (2^21 ≈ 2M)
2. Add some evaluation constraints (mimicking circuit size)
3. Run benchmarks with WHIR-CB configuration
4. Measure: prover time, verifier time, proof size, hash count
5. Compare with Groth16 baseline

**Pros**:
- Immediate feedback on performance
- Low risk, high value
- Helps decide if WHIR is worth pursuing

**Cons**:
- Not the full system
- May miss R1CS-specific overhead
- Cannot use real circuit inputs

---

## 10. Concrete Benchmarking Example

### **Simple PCS Benchmark Script**

```rust
use whir::*;

fn benchmark_whir_pcs() {
    // Parameters matching Aptos keyless scale
    let num_variables = 21;  // 2^21 ≈ 2M (close to 1.4M constraints)
    let security_level = 128;
    let folding_factor = 4;
    let rate = 1;  // log_inv_rate
    
    // Create simple polynomial (all 1s for now)
    let num_coeffs = 1 << num_variables;
    let polynomial = CoefficientList::new(vec![F::ONE; num_coeffs]);
    
    // Setup WHIR with WHIR-CB configuration
    let mv_params = MultivariateParameters::new(num_variables);
    let whir_params = ProtocolParameters {
        initial_statement: true,
        security_level,
        pow_bits: 0,  // Start with no PoW for baseline
        folding_factor: FoldingFactor::Constant(folding_factor),
        soundness_type: SoundnessType::ConjectureList,  // WHIR-CB
        starting_log_inv_rate: rate,
        // ... other params
    };
    
    let params = WhirConfig::new(mv_params, whir_params);
    
    // Create statement with a few evaluation constraints
    let mut statement = Statement::new(num_variables);
    let point = MultilinearPoint(vec![F::from(42); num_variables]);
    let eval = polynomial.evaluate_at_extension(&point);
    statement.add_constraint(Weights::evaluation(point), eval);
    
    // Benchmark prover
    let start = Instant::now();
    let committer = CommitmentWriter::new(params.clone());
    let witness = committer.commit(&mut prover_state, &polynomial)?;
    let prover = Prover::new(params.clone());
    prover.prove(&mut prover_state, statement.clone(), witness)?;
    let prover_time = start.elapsed();
    
    // Benchmark verifier
    let verifier = Verifier::new(&params);
    let start = Instant::now();
    verifier.verify(&mut verifier_state, &parsed_commitment, &statement)?;
    let verifier_time = start.elapsed();
    
    // Measure proof size
    let proof_size = prover_state.narg_string().len();
    
    println!("WHIR-CB Benchmark (num_variables={})", num_variables);
    println!("Prover time: {:.2?}", prover_time);
    println!("Verifier time: {:.2?}", verifier_time);
    println!("Proof size: {:.1} KiB", proof_size as f64 / 1024.0);
    println!("Verifier hashes: {}", HashCounter::get());
}
```

**Expected output (based on paper benchmarks for m=21)**:
```
WHIR-CB Benchmark (num_variables=21)
Prover time: ~5-10s (need to measure)
Verifier time: ~2-5ms (need to measure)
Proof size: ~200-300 KiB
Verifier hashes: ~3-5k
```

**Comparison with Groth16**:
| Metric | Groth16 | WHIR-CB (estimated) | Ratio |
|--------|---------|---------------------|-------|
| Prover | 37s | ~5-10s? | **0.13-0.27× (faster!)** |
| Verifier | 800ms | ~2-5ms | **0.0025-0.006× (much faster!)** |
| Proof size | 805 bytes | ~200-300 KiB | **250-370× (much larger)** |

**Note**: These are rough estimates. The actual WHIR prover time may be higher due to:
- Hash operations (Blake3 is fast but not free)
- Merkle tree construction
- FFT operations for RS encoding

---

## 11. Next Steps

### **Immediate (Option 3)**:
1. ✅ Understand code structure (this document)
2. ⏳ Create simple benchmarking script
3. ⏳ Run WHIR-CB with m=21 (2^21 ≈ 2M)
4. ⏳ Measure prover/verifier time, proof size, hash count
5. ⏳ Compare with Groth16 baseline
6. ⏳ Document findings in `Linear_Update.md`

### **If WHIR looks promising**:
1. Decide: Build R1CS frontend (Option 1) or use different STARK (Option 2)
2. Estimate complexity and timeline
3. Prototype minimal viable implementation
4. Benchmark with real Aptos keyless circuit
5. Analyze trade-offs and make final decision

### **If WHIR doesn't meet constraints**:
1. Pivot to HyperPLONK (universal setup, 1 sumcheck + 1 ZK PCS)
2. Investigate other universal SNARKs (Marlin, Sonic, etc.)

---

## 12. References

- **WHIR Paper**: https://eprint.iacr.org/2024/1586
- **WHIR Repository**: https://github.com/WizardOfMenlo/whir
- **Arkworks**: https://arkworks.rs (polynomial and curve libraries)
- **Spongefish**: https://github.com/arkworks-rs/spongefish (Fiat-Shamir transcript)

---

**Document prepared by**: Code analysis for Aptos Universal SNARK research  
**Last updated**: October 24, 2024

