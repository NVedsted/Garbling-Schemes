use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use cc_garbling::garbled_circuit::{GarbledCircuit, GarbledEncoder};
use cc_garbling::garbled_circuit::classic::Classic;
use cc_garbling::garbled_circuit::half_gates::HalfGates;

const INPUTS: [(&str, &str); 8] = [
    ("adder64", include_str!("../circuits/adder64.txt")),
    ("mult64", include_str!("../circuits/mult64.txt")),
    ("subtract64", include_str!("../circuits/subtract64.txt")),
    ("udivide64", include_str!("../circuits/udivide64.txt")),
    ("zero_equal", include_str!("../circuits/zero_equal.txt")),
    ("aes_128", include_str!("../circuits/aes_128.txt")),
    ("sha256", include_str!("../circuits/sha256.txt")),
    ("sha512", include_str!("../circuits/sha512.txt")),
];

macro_rules! garble {
    ($group:ident, $gc_name:literal, $gc:ty, $circuit_name:ident, $circuit:ident) => {
        $group.bench_with_input(
            BenchmarkId::new($gc_name, $circuit_name), &$circuit,
            |b, circuit| b.iter(|| {
                <$gc>::garble_circuit(&circuit)
            }));
    };
}

fn bench_garble(c: &mut Criterion) {
    let mut g = c.benchmark_group("garble");
    for (name, input) in INPUTS {
        let circuit = input.parse().unwrap();
        garble!(g, "HalfGates", HalfGates, name, circuit);
        garble!(g, "Classic", Classic, name, circuit);
    }
}

macro_rules! evaluate {
    ($group:ident, $name:literal, $gc:ty) => {
        for (name, input) in INPUTS {
            let circuit = input.parse().unwrap();
            let (gc, enc, _) = <$gc>::garble_circuit(&circuit);
            let input = enc.encode(&vec![false; circuit.input_length]);
            $group.bench_with_input(
                BenchmarkId::new($name, name), &input,
                |b, input| b.iter(|| {
                    gc.evaluate((*input).clone())
                }));
        }
    };
}

fn bench_evaluate(c: &mut Criterion) {
    let mut g = c.benchmark_group("evaluate");
    evaluate!(g, "HalfGates", HalfGates);
    evaluate!(g, "Classic", Classic);
}

criterion_group!(benches,
    bench_garble,
    bench_evaluate,
);
criterion_main!(benches);
