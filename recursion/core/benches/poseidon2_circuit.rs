use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use itertools::Itertools;
use serde::Serialize;
use zkhash::ark_ff::UniformRand;

use p3_baby_bear::BabyBear;
use p3_baby_bear::DiffusionMatrixBabyBear;
use p3_matrix::dense::RowMajorMatrix;
use p3_poseidon2::Poseidon2;
use p3_poseidon2::Poseidon2ExternalMatrixGeneral;
use sp1_core::stark::StarkGenericConfig;
use sp1_core::utils::inner_perm;
use sp1_core::{
    air::MachineAir,
    utils::{uni_stark_prove, BabyBearPoseidon2},
};

use p3_symmetric::Permutation;
use sp1_recursion_core::{
    poseidon2::{Poseidon2Chip, Poseidon2Event},
    runtime::ExecutionRecord,
};

fn benchmark_my_function(c: &mut Criterion) {
    for size in (10..=15).map(|exp| 2usize.pow(exp)) {
        c.bench_function(format!("poseidon2_circuit_{}", size).as_str(), |b| {
            b.iter_batched(
                // Setup code
                || {
                    let rng = &mut rand::thread_rng();

                    let inputs: Vec<[BabyBear; 16]> = (0..size)
                        .map(|_| core::array::from_fn(|_| BabyBear::rand(rng)))
                        .collect_vec();

                    let gt: Poseidon2<
                        BabyBear,
                        Poseidon2ExternalMatrixGeneral,
                        DiffusionMatrixBabyBear,
                        16,
                        7,
                    > = inner_perm();

                    let outputs = inputs
                        .iter()
                        .map(|input| gt.permute(*input))
                        .collect::<Vec<_>>();

                    let mut input_exec = ExecutionRecord::<BabyBear>::default();
                    for (input, output) in inputs.into_iter().zip_eq(outputs) {
                        input_exec
                            .poseidon2_events
                            .push(Poseidon2Event::dummy_from_input(input, output));
                    }

                    let chip = Poseidon2Chip {
                        fixed_log2_rows: None,
                        pad: true,
                    };
                    let trace: RowMajorMatrix<BabyBear> = chip
                        .generate_trace(&input_exec, &mut ExecutionRecord::<BabyBear>::default());

                    let config = BabyBearPoseidon2::compressed();
                    let challenger = config.challenger();

                    (config, chip, challenger, trace)
                },
                // Code to benchmark
                |(config, chip, mut challenger, trace)| {
                    let _ = uni_stark_prove(&config, &chip, &mut challenger, trace);
                },
                // Benchmarking policy
                criterion::BatchSize::LargeInput,
            );
        });
    }
}

criterion_group!(benches, benchmark_my_function);
criterion_main!(benches);
