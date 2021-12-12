use std::ops::{BitAnd, BitXor};

use itertools::Itertools;
use rand::RngCore;
use rand::rngs::OsRng;
use rand::seq::SliceRandom;
use sha2::{Digest, Sha256};
use sha2::digest::Update;

use crate::circuit::{Circuit, Gate};
use crate::garbled_circuit::{GarbledCircuit, GarbledDecoder, GarbledEncoder};

const H_BYTES: usize = 256 / 8;
const LABEL_BYTES: usize = H_BYTES / 2;

pub struct ClassicEncoder {
    inner: Vec<(Vec<u8>, Vec<u8>)>,
}

impl GarbledEncoder<Vec<u8>> for ClassicEncoder {
    fn encode(&self, input: &[bool]) -> Vec<Vec<u8>> {
        input.iter().cloned()
            .zip(&self.inner)
            .map(|(b, e)| {
                if b { &e.1 } else { &e.0 }.clone()
            })
            .collect()
    }
}

pub struct ClassicDecoder {
    inner: Vec<(Vec<u8>, Vec<u8>)>,
}

impl GarbledDecoder<Vec<u8>> for ClassicDecoder {
    fn decode(&self, input: &[Vec<u8>]) -> Vec<bool> {
        input.iter()
            .zip(&self.inner)
            .map(|(b, e)| {
                if eq(b, &e.1) {
                    true
                } else if eq(b, &e.0) {
                    false
                } else {
                    panic!("Invalid")
                }
            })
            .collect()
    }
}

pub struct Classic {
    input_length: usize,
    output_length: usize,
    gates: Vec<(usize, Gate)>,
    ciphers: Vec<Vec<Vec<u8>>>,
}

impl GarbledCircuit<ClassicEncoder, ClassicDecoder> for Classic {
    type Label = Vec<u8>;

    fn evaluate(&self, mut input: Vec<Self::Label>) -> Vec<Self::Label> {
        assert_eq!(input.len(), self.input_length);

        let mut values = {
            input.resize(self.input_length + self.gates.len(), Default::default());
            input
        };

        self.gates.iter().cloned()
            .zip(&self.ciphers)
            .for_each(|((output, gate), ciphers)| {
                let h = match gate {
                    Gate::Inv(a) => {
                        let label = &values[a];
                        hash(&[label, &output.to_be_bytes()])
                    }
                    Gate::And(a, b) | Gate::Xor(a, b) => {
                        let left_label = &values[a];
                        let right_label = &values[b];
                        hash(&[left_label, right_label, &output.to_be_bytes()])
                    }
                };
                if let Ok(Some(mut correct)) = ciphers.iter()
                    .map(|c| xor(c, &h))
                    .filter(|c| c[LABEL_BYTES..].iter().all(|&e| e == 0))
                    .at_most_one() {
                    correct.resize(LABEL_BYTES, 0);
                    values[output] = correct;
                } else {
                    panic!("Too many or few correct");
                }
            });

        values.into_iter()
            .rev()
            .take(self.output_length)
            .rev()
            .collect()
    }

    fn garble_circuit(circuit: &Circuit) -> (Self, ClassicEncoder, ClassicDecoder) {
        let labels = (0..circuit.input_length + circuit.gates.len())
            .map(|_| {
                let mut label0 = vec![0u8; LABEL_BYTES];
                OsRng.fill_bytes(&mut label0);
                let mut label1 = vec![0u8; LABEL_BYTES];
                OsRng.fill_bytes(&mut label1);
                (label0, label1)
            })
            .collect::<Vec<_>>();

        let encoding = labels.iter()
            .cloned()
            .take(circuit.input_length)
            .collect::<Vec<_>>();

        let decoding = labels.iter()
            .cloned()
            .rev()
            .take(circuit.output_length)
            .rev()
            .collect::<Vec<_>>();

        let ciphers = circuit.gates.iter()
            .cloned()
            .map(|(output, gate)| {
                match gate {
                    Gate::Inv(a) => {
                        let mut c = [false, true].iter().cloned()
                            .map(|value| {
                                let label = if value { &labels[a].1 } else { &labels[a].0 };
                                let h = hash(&[label, &output.to_be_bytes()]);
                                let mut result = if value {
                                    &labels[output].0
                                } else {
                                    &labels[output].1
                                }.clone();
                                result.resize(2 * LABEL_BYTES, 0);
                                xor(&result, &h)
                            })
                            .collect::<Vec<_>>();
                        c.shuffle(&mut OsRng);
                        c
                    }
                    Gate::And(a, b) => {
                        garble_binary(a, b, output, &labels, bool::bitand)
                    }
                    Gate::Xor(a, b) => {
                        garble_binary(a, b, output, &labels, bool::bitxor)
                    }
                }
            })
            .collect::<Vec<_>>();

        (
            Classic {
                input_length: circuit.input_length,
                output_length: circuit.output_length,
                gates: circuit.gates.clone(),
                ciphers,
            },
            ClassicEncoder { inner: encoding },
            ClassicDecoder { inner: decoding },
        )
    }
}

fn garble_binary<F>(a: usize, b: usize, output: usize, labels: &Vec<(Vec<u8>, Vec<u8>)>, mut f: F) -> Vec<Vec<u8>>
    where F: FnMut(bool, bool) -> bool {
    let mut c = [false, true].iter().cloned()
        .cartesian_product([false, true].iter().cloned())
        .map(|(left, right)| {
            let left_label = if left { &labels[a].1 } else { &labels[a].0 };
            let right_label = if right { &labels[b].1 } else { &labels[b].0 };
            let h = hash(&[left_label, right_label, &output.to_be_bytes()]);
            let mut result = if f(left, right) {
                &labels[output].1
            } else {
                &labels[output].0
            }.clone();
            result.extend([0u8; LABEL_BYTES]);
            xor(&result, &h)
        })
        .collect::<Vec<_>>();
    c.shuffle(&mut OsRng);
    c
}

fn hash(input: &[&[u8]]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    input.iter()
        .cloned()
        .for_each(|i| Update::update(&mut hasher, i));
    hasher.finalize().to_vec()
}

fn xor(a: &[u8], b: &[u8]) -> Vec<u8> {
    assert_eq!(a.len(), b.len());
    a.iter()
        .zip(b)
        .map(|(a, b)| a ^ b)
        .collect()
}

fn eq(a: &[u8], b: &[u8]) -> bool {
    assert_eq!(a.len(), b.len());
    a.iter()
        .zip(b)
        .all(|(a, b)| a == b)
}

#[cfg(test)]
mod tests {
    use crate::test_garbled_circuit;

    test_garbled_circuit!(Classic);
}
