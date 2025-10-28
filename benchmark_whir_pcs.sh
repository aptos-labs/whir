#!/bin/bash
set -e

echo "========================================"
echo "WHIR PCS Benchmark - Single Thread"
echo "========================================"
echo "Configuration:"
echo "  Field: BN254 (Field256)"
echo "  Rate: ρ=1/8 (rate=3)"
echo "  Security: 128 bits"
echo "  Mode: PCS with ConjectureList"
echo "  Threads: 1 (single-threaded)"
echo "========================================"
echo ""

# Force single-threaded execution
export RAYON_NUM_THREADS=1

# Build in release mode first
echo "Building WHIR in release mode..."
cargo build --release --bin main
echo ""

# Define instance sizes
INSTANCES=(18 19 20 21)

# Define hash functions
HASH_FUNCTIONS=("Blake3" "Keccak" "Poseidon")

# Create output directory
OUTPUT_DIR="benchmark_results_pcs"
mkdir -p "$OUTPUT_DIR"

# Log file for all results
RESULTS_FILE="$OUTPUT_DIR/benchmark_summary.txt"
echo "WHIR PCS Benchmark Results" > "$RESULTS_FILE"
echo "Configuration: BN254, ρ=1/8, 128-bit security, 1 thread" >> "$RESULTS_FILE"
echo "Date: $(date)" >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

# Run benchmarks
for INSTANCE in "${INSTANCES[@]}"; do
  echo "========================================"
  echo "Instance Size: 2^${INSTANCE}"
  echo "========================================"
  echo "" | tee -a "$RESULTS_FILE"
  echo "=== Instance 2^${INSTANCE} ===" >> "$RESULTS_FILE"
  
  for HASH in "${HASH_FUNCTIONS[@]}"; do
    echo "----------------------------------------"
    echo "Hash Function: ${HASH}"
    echo "----------------------------------------"
    
    LOG_FILE="$OUTPUT_DIR/whir_2_${INSTANCE}_${HASH}.log"
    
    echo "Running: cargo run --release --bin main -- -t PCS -d ${INSTANCE} -r 3 -f Field256 --hash ${HASH} --sec ConjectureList -l 128"
    echo ""
    
    # Run WHIR and capture output
    cargo run --release --bin main -- \
      -t PCS \
      -d ${INSTANCE} \
      -r 3 \
      -f Field256 \
      --hash ${HASH} \
      --sec ConjectureList \
      -l 128 \
      2>&1 | tee "$LOG_FILE"
    
    # Extract key metrics and append to summary
    echo "Hash: ${HASH}" >> "$RESULTS_FILE"
    grep "Prover time:" "$LOG_FILE" >> "$RESULTS_FILE" || echo "Prover time: N/A" >> "$RESULTS_FILE"
    grep "Proof size:" "$LOG_FILE" >> "$RESULTS_FILE" || echo "Proof size: N/A" >> "$RESULTS_FILE"
    grep "Verifier time:" "$LOG_FILE" >> "$RESULTS_FILE" || echo "Verifier time: N/A" >> "$RESULTS_FILE"
    grep "Average hashes:" "$LOG_FILE" >> "$RESULTS_FILE" || echo "Average hashes: N/A" >> "$RESULTS_FILE"
    echo "" >> "$RESULTS_FILE"
    
    echo ""
  done
  
  echo "" >> "$RESULTS_FILE"
done

echo "========================================"
echo "Benchmark Complete!"
echo "========================================"
echo "Results saved to: $OUTPUT_DIR/"
echo "Summary: $RESULTS_FILE"
echo ""
echo "Quick summary:"
cat "$RESULTS_FILE"

