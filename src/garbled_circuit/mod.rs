use crate::circuit::Circuit;

pub mod half_gates;

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
