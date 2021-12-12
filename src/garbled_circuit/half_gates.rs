use aes::{Aes128, Block, BlockEncrypt, NewBlockCipher};
use rand::RngCore;
use rand::rngs::OsRng;

use crate::circuit::{Circuit, Gate};
use crate::garbled_circuit::{GarbledCircuit, GarbledDecoder, GarbledEncoder};

pub struct HalfGatesEncoder {
    inner: Vec<(Block, Block)>,
}

impl GarbledEncoder<Block> for HalfGatesEncoder {
    fn encode(&self, input: &[bool]) -> Vec<Block> {
        assert_eq!(input.len(), self.inner.len());

        self.inner.iter()
            .zip(input)
            .map(|(e, &b)| match b {
                false => e.0.clone(),
                true => e.1.clone(),
            })
            .collect()
    }
}

pub struct HalfGatesDecoder {
    inner: Vec<bool>,
}

impl GarbledDecoder<Block> for HalfGatesDecoder {
    fn decode(&self, input: &[Block]) -> Vec<bool> {
        assert_eq!(input.len(), self.inner.len());

        self.inner.iter()
            .cloned()
            .zip(input)
            .map(|(d, o)| get_lsb(o) ^ d)
            .collect()
    }
}

#[derive(Debug, Copy, Clone)]
enum GarbledGate {
    // TODO: avoid copy instruction when removing INV.
    Copy(usize),
    And(usize, usize),
    Xor(usize, usize),
}

pub struct HalfGates {
    input_length: usize,
    output_length: usize,
    gates: Vec<(usize, GarbledGate)>,
    ciphers: Vec<(Block, Block)>,
    key: Block,
}

impl GarbledCircuit<HalfGatesEncoder, HalfGatesDecoder> for HalfGates {
    type Label = Block;

    fn evaluate(&self, mut input: Vec<Self::Label>) -> Vec<Self::Label> {
        assert_eq!(self.input_length, input.len());

        let block_cipher = Aes128::new(&self.key);

        let mut values = {
            input.resize(self.input_length + self.gates.len(), Default::default());
            input
        };

        let mut and_count = 0;
        for &(output, gate) in self.gates.iter() {
            match gate {
                GarbledGate::And(a, b) => {
                    let sa = get_lsb(&values[a]);
                    let sb = get_lsb(&values[b]);
                    // TODO use indexes
                    // TODO error handling for and_count
                    let (tg, te) = &self.ciphers[and_count];
                    // First half gate
                    let mut ha = values[a].clone();
                    block_cipher.encrypt_block(&mut ha);
                    let mut wg = ha.clone();
                    if sa {
                        wg = xor_blocks(&wg, tg);
                    }
                    // Second half gate
                    let mut hb = values[b].clone();
                    block_cipher.encrypt_block(&mut hb);
                    let mut we = hb.clone();
                    if sb {
                        we = xor_blocks(&we, &xor_blocks(&te, &values[a]));
                    }

                    values[output] = xor_blocks(&wg, &we);

                    and_count += 1;
                }
                GarbledGate::Xor(a, b) => {
                    values[output] = xor_blocks(&values[a], &values[b]);
                }
                GarbledGate::Copy(a) => {
                    values[output] = values[a].clone();
                }
            }
        }

        values.iter()
            .rev()
            .take(self.output_length)
            .rev()
            .cloned()
            .collect()
    }

    fn garble_circuit(circuit: &Circuit) -> (Self, HalfGatesEncoder, HalfGatesDecoder) {
        let key = {
            let mut key: Block = Default::default();
            OsRng.fill_bytes(&mut key);
            key
        };

        let block_cipher = Aes128::new(&key);

        let r = {
            let mut r: Block = Default::default();
            OsRng.fill_bytes(&mut r);
            // Assume little endian for ease of use.
            set_lsb(&mut r, true);
            r
        };
        let mut labels: Vec<(Block, Block)> = vec![(Default::default(), Default::default()); circuit.input_length + circuit.gates.len()];
        for i in 0..circuit.input_length {
            OsRng.fill_bytes(&mut labels[i].0);
            labels[i].1 = xor_blocks(&labels[i].0, &r);
        }
        let encoding = labels.iter()
            .cloned()
            .take(circuit.input_length)
            .collect::<Vec<_>>();

        let mut ciphers = vec![];
        let garbled_gates = circuit.gates.iter()
            .map(|&(output, gate)| {
                (output, match gate {
                    Gate::Inv(a) => {
                        labels[output] = (labels[a].1.clone(), labels[a].0.clone());
                        GarbledGate::Copy(a)
                    }
                    Gate::And(a, b) => {
                        let pa = get_lsb(&labels[a].0);
                        let pb = get_lsb(&labels[b].0);
                        // TODO: indexes
                        let _j = labels.len();
                        let _j_prime = labels.len() + 1;
                        // First half gate
                        let h0 = {
                            let mut h0 = labels[a].0.clone();
                            block_cipher.encrypt_block(&mut h0);
                            h0
                        };
                        let h1 = {
                            let mut h1 = labels[a].1.clone();
                            block_cipher.encrypt_block(&mut h1);
                            h1
                        };
                        let mut tg = xor_blocks(&h0, &h1);
                        if pb {
                            tg = xor_blocks(&tg, &r);
                        }
                        let mut w0g = h0.clone();
                        if pa {
                            w0g = xor_blocks(&w0g, &tg);
                        }
                        // Second half gate
                        let h0 = {
                            let mut h0 = labels[b].0.clone();
                            block_cipher.encrypt_block(&mut h0);
                            h0
                        };
                        let h1 = {
                            let mut h1 = labels[b].1.clone();
                            block_cipher.encrypt_block(&mut h1);
                            h1
                        };
                        let te = xor_blocks(&xor_blocks(&h0, &h1), &labels[a].0);
                        let mut w0e = h0.clone();
                        if pb {
                            // TODO simplify
                            w0e = xor_blocks(&w0e, &xor_blocks(&te, &labels[a].0));
                        }
                        // Combine halves
                        labels[output].0 = xor_blocks(&w0g, &w0e);
                        labels[output].1 = xor_blocks(&labels[output].0, &r);
                        ciphers.push((tg, te));
                        GarbledGate::And(a, b)
                    }
                    Gate::Xor(a, b) => {
                        labels[output].0 = xor_blocks(&labels[a].0, &labels[b].0);
                        labels[output].1 = xor_blocks(&labels[output].0, &r);
                        GarbledGate::Xor(a, b)
                    }
                })
            })
            .collect::<Vec<_>>();

        let decoding = labels.iter()
            .rev()
            .take(circuit.output_length)
            .rev()
            .map(|(w0, _)| w0[0] & 1 != 0)
            .collect::<Vec<_>>();

        (
            HalfGates {
                input_length: circuit.input_length,
                output_length: circuit.output_length,
                gates: garbled_gates,
                ciphers,
                key,
            },
            HalfGatesEncoder { inner: encoding },
            HalfGatesDecoder { inner: decoding },
        )
    }
}

fn xor_blocks(a: &Block, b: &Block) -> Block {
    let mut block: Block = Default::default();
    block.iter_mut()
        .zip(a.iter().zip(b.iter()))
        .for_each(|(dst, (a, b))| *dst = a ^ b);
    block
}

// TODO: consider inline
fn get_lsb(s: &[u8]) -> bool {
    s[0] & 1 != 0
}

// TODO: consider inline
fn set_lsb(s: &mut [u8], b: bool) {
    if b {
        s[0] |= 1;
    } else {
        s[0] &= 0;
    }
}

#[cfg(test)]
mod tests {
    use rand::thread_rng;

    use crate::test_garbled_circuit;

    #[test]
    fn test_lsb() {
        let mut s = [8, 9, 10];
        set_lsb(&mut s, true);
        assert_eq!(get_lsb(&s), true);
        set_lsb(&mut s, false);
        assert_eq!(get_lsb(&s), false);
    }

    #[test]
    fn test_xor_blocks() {
        let mut a: Block = Default::default();
        let mut b: Block = Default::default();

        thread_rng().fill_bytes(&mut a);
        thread_rng().fill_bytes(&mut b);

        let c = xor_blocks(&a, &b);

        for i in 0..a.len() {
            assert_eq!(a[i] ^ b[i], c[i]);
        }
    }

    test_garbled_circuit!(HalfGates);
}
