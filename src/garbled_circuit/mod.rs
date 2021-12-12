use crate::circuit::Circuit;

pub mod half_gates;
pub mod classic;

pub trait GarbledEncoder<T> {
    fn encode(&self, input: &[bool]) -> Vec<T>;
}

pub trait GarbledDecoder<T> {
    fn decode(&self, input: &[T]) -> Vec<bool>;
}

pub trait GarbledCircuit<E: GarbledEncoder<Self::Label>, D: GarbledDecoder<Self::Label>>: Sized {
    type Label;

    fn evaluate(&self, input: Vec<Self::Label>) -> Vec<Self::Label>;

    fn garble_circuit(circuit: &Circuit) -> (Self, E, D);

    fn garble_compute(&self, encoder: &E, decoder: &D, input: &[bool]) -> Vec<bool> {
        let encoded_input = encoder.encode(input);
        let encoded_output = self.evaluate(encoded_input);
        decoder.decode(&encoded_output)
    }
}

#[cfg(test)]
mod tests {
    #[macro_export]
    macro_rules! test_garbled_circuit {
        ($gc:ty) => {
            use super::*;
            use hex_literal::hex;
            use crate::circuit::Circuit;
            use crate::garbled_circuit::GarbledCircuit;
            use crate::util::{bits_to_u64, u64_to_bits, u8_to_bits, bits_to_u8};

            fn evaluate_u64(circuit: &Circuit, inputs: &[u64]) -> Vec<bool> {
                let (gc, enc, dec) = <$gc>::garble_circuit(&circuit);
                let input = inputs.iter()
                    .cloned()
                    .map(u64_to_bits)
                    .flatten()
                    .collect::<Vec<_>>();
                gc.garble_compute(&enc, &dec, &input)
            }

            #[test]
            fn test_zero_equal() {
                let circuit: Circuit = include_str!("../../circuits/zero_equal.txt").parse().unwrap();

                assert_eq!(evaluate_u64(&circuit, &[0]), [true]);
                assert_eq!(evaluate_u64(&circuit, &[1000]), [false]);
                assert_eq!(evaluate_u64(&circuit, &[5489744564]), [false]);
                assert_eq!(evaluate_u64(&circuit, &[1]), [false]);
            }

            fn binop_u64(circuit: &Circuit, left: u64, right: u64) -> u64 {
                bits_to_u64(&evaluate_u64(circuit, &[left, right]))
            }

            #[test]
            fn test_adder_64() {
                let circuit: Circuit = include_str!("../../circuits/adder64.txt").parse().unwrap();
                assert!(circuit.verify_topology());

                assert_eq!(binop_u64(&circuit, 0, 0), 0);
                assert_eq!(binop_u64(&circuit, 1, 1), 2);
                assert_eq!(binop_u64(&circuit, 10, 5), 15);
                assert_eq!(binop_u64(&circuit, 0, 5000), 5000);
                assert_eq!(binop_u64(&circuit, 300, 0), 300);
            }

            #[test]
            fn test_subtract_64() {
                let circuit: Circuit = include_str!("../../circuits/subtract64.txt").parse().unwrap();
                assert!(circuit.verify_topology());

                assert_eq!(binop_u64(&circuit, 0, 0), 0);
                assert_eq!(binop_u64(&circuit, 1, 1), 0);
                assert_eq!(binop_u64(&circuit, 10, 5), 5);
                assert_eq!(binop_u64(&circuit, 5000, 300), 4700);
                assert_eq!(binop_u64(&circuit, 14894156165, 155644), 14894156165 - 155644);
            }

            #[test]
            fn test_mult_64() {
                let circuit: Circuit = include_str!("../../circuits/mult64.txt").parse().unwrap();
                assert!(circuit.verify_topology());

                assert_eq!(binop_u64(&circuit, 0, 0), 0);
                assert_eq!(binop_u64(&circuit, 1, 1), 1);
                assert_eq!(binop_u64(&circuit, 10, 5), 50);
                assert_eq!(binop_u64(&circuit, 5000, 300), 5000 * 300);
                assert_eq!(binop_u64(&circuit, 14894155, 155644), 14894155 * 155644);
            }

            #[test]
            fn test_divide_64() {
                let circuit: Circuit = include_str!("../../circuits/udivide64.txt").parse().unwrap();
                assert!(circuit.verify_topology());

                assert_eq!(binop_u64(&circuit, 0, 1), 0);
                assert_eq!(binop_u64(&circuit, 1, 1), 1);
                assert_eq!(binop_u64(&circuit, 15, 5), 3);
                assert_eq!(binop_u64(&circuit, 5000, 200), 25);
                assert_eq!(binop_u64(&circuit, 258290865, 165465), 1561);
            }

            #[test]
            fn test_aes_128() {
                let key: [u8; 16] = hex!("74 c9 f1 91 b9 02 f9 6c 32 24 3e 13 b3 5f 12 af");
                let msg: [u8; 16] = hex!("80 14 bf b6 e6 00 f1 cd 5e ec ce c5 11 2c 4c f9");
                let cph: [u8; 16] = hex!("7f 16 ae 5c 79 5b 18 86 b8 ca 08 b6 96 6a 7a 7c");

                let mut input = key.iter().cloned()
                    .flat_map(u8_to_bits)
                    .rev()
                    .collect::<Vec<_>>();
                input.extend(msg.iter().cloned()
                    .flat_map(u8_to_bits)
                    .rev());

                let circuit: Circuit = include_str!("../../circuits/aes_128.txt").parse().unwrap();
                let (gc, enc, dec) = <$gc>::garble_circuit(&circuit);
                let mut output = gc.garble_compute(&enc, &dec, &input);
                output.reverse();
                let output_bytes = output.chunks_exact(8)
                    .map(|e| bits_to_u8(e))
                    .collect::<Vec<_>>();
                assert_eq!(output_bytes, cph);
            }
        };
    }
}
