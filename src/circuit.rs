use std::str::FromStr;

// Gates needed: INV, AND, XOR
// Rewrite Inv -> Xor
#[derive(Debug, Copy, Clone)]
pub enum Gate {
    Inv(usize),
    And(usize, usize),
    Xor(usize, usize),
}

#[derive(Clone, Debug)]
pub struct Circuit {
    pub(crate) input_length: usize,
    pub(crate) output_length: usize,
    pub(crate) gates: Vec<(usize, Gate)>,
}

impl Circuit {
    pub fn verify_topology(&self) -> bool {
        let mut populated = vec![false; self.input_length + self.gates.len()];
        populated[..self.input_length].fill(true);

        self.gates.iter()
            .all(|&(output_wire, g)| {
                let valid = match g {
                    Gate::Inv(x) => populated[x],
                    Gate::And(x, y) | Gate::Xor(x, y) =>
                        populated[x] && populated[y],
                };
                populated[output_wire] = true;
                valid
            }
            )
    }

    pub fn evaluate(&self, mut input: Vec<bool>) -> Vec<bool> {
        assert_eq!(self.input_length, input.len());

        let mut values = {
            input.resize(self.input_length + self.gates.len(), false);
            input
        };

        for &(output_wire, gate) in self.gates.iter() {
            values[output_wire] = match gate {
                Gate::Inv(x) => !values[x],
                Gate::And(x, y) => values[x] && values[y],
                Gate::Xor(x, y) => values[x] ^ values[y],
            };
        }

        values.into_iter()
            .rev()
            .take(self.output_length)
            .rev()
            .collect()
    }
}

impl FromStr for Circuit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (header, gates) = s.trim().split_once("\n\n").unwrap();
        let mut header_lines = header.lines()
            .map(|l| l.trim()
                .split_ascii_whitespace()
                .map(|e| e.parse::<usize>().unwrap())
            );
        let mut first_line = header_lines.next().unwrap();
        let gate_count = first_line.next().unwrap();
        let wire_count = first_line.next().unwrap();
        let input_length = header_lines.next().unwrap().skip(1).sum::<usize>();
        let output_length = header_lines.next().unwrap().skip(1).sum::<usize>();

        assert_eq!(input_length + gate_count, wire_count);

        let mut real_gates = Vec::with_capacity(gate_count);

        for raw_gate in gates.lines() {
            let mut data = raw_gate.split_ascii_whitespace();
            let input_count = data.next().unwrap().parse::<usize>().unwrap();
            let output_count = data.next().unwrap().parse::<usize>().unwrap();
            assert_eq!(output_count, 1);

            let inputs = (0..input_count)
                .map(|_| data.next().unwrap().parse::<usize>().unwrap())
                .collect::<Vec<_>>();
            let output_wire = data.next().unwrap().parse::<usize>().unwrap();

            let operation = data.next().unwrap();
            let gate = match operation {
                "INV" if input_count == 1 => Gate::Inv(inputs[0]),
                "AND" if input_count == 2 => Gate::And(inputs[0], inputs[1]),
                "XOR" if input_count == 2 => Gate::Xor(inputs[0], inputs[1]),
                _ => panic!("Invalid operation/input count")
            };
            real_gates.push((output_wire, gate))
        }

        Ok(Circuit {
            input_length,
            output_length,
            gates: real_gates,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::circuit::Circuit;
    use crate::util::{bits_to_u64, u64_to_bits};

    fn evaluate_u64(circuit: &Circuit, inputs: &[u64]) -> Vec<bool> {
        let input = inputs.iter()
            .cloned()
            .map(u64_to_bits)
            .flatten()
            .collect::<Vec<_>>();
        circuit.evaluate(input)
    }

    #[test]
    fn test_zero_equal() {
        let circuit: Circuit = include_str!("../circuits/zero_equal.txt").parse().unwrap();
        assert!(circuit.verify_topology());

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
        let circuit: Circuit = include_str!("../circuits/adder64.txt").parse().unwrap();
        assert!(circuit.verify_topology());

        assert_eq!(binop_u64(&circuit, 0, 0), 0);
        assert_eq!(binop_u64(&circuit, 1, 1), 2);
        assert_eq!(binop_u64(&circuit, 10, 5), 15);
        assert_eq!(binop_u64(&circuit, 0, 5000), 5000);
        assert_eq!(binop_u64(&circuit, 300, 0), 300);
    }

    #[test]
    fn test_subtract_64() {
        let circuit: Circuit = include_str!("../circuits/subtract64.txt").parse().unwrap();
        assert!(circuit.verify_topology());

        assert_eq!(binop_u64(&circuit, 0, 0), 0);
        assert_eq!(binop_u64(&circuit, 1, 1), 0);
        assert_eq!(binop_u64(&circuit, 10, 5), 5);
        assert_eq!(binop_u64(&circuit, 5000, 300), 4700);
        assert_eq!(binop_u64(&circuit, 14894156165, 155644), 14894156165 - 155644);
    }

    #[test]
    fn test_mult_64() {
        let circuit: Circuit = include_str!("../circuits/mult64.txt").parse().unwrap();
        assert!(circuit.verify_topology());

        assert_eq!(binop_u64(&circuit, 0, 0), 0);
        assert_eq!(binop_u64(&circuit, 1, 1), 1);
        assert_eq!(binop_u64(&circuit, 10, 5), 50);
        assert_eq!(binop_u64(&circuit, 5000, 300), 5000 * 300);
        assert_eq!(binop_u64(&circuit, 14894155, 155644), 14894155 * 155644);
    }

    #[test]
    fn test_divide_64() {
        let circuit: Circuit = include_str!("../circuits/udivide64.txt").parse().unwrap();
        assert!(circuit.verify_topology());

        assert_eq!(binop_u64(&circuit, 0, 1), 0);
        assert_eq!(binop_u64(&circuit, 1, 1), 1);
        assert_eq!(binop_u64(&circuit, 15, 5), 3);
        assert_eq!(binop_u64(&circuit, 5000, 200), 25);
        assert_eq!(binop_u64(&circuit, 258290865, 165465), 1561);
    }
}
