pub fn u64_to_bits(mut x: u64) -> Vec<bool> {
    let mut bits = vec![false; 64];
    for i in 0..64 {
        bits[i] = x & 1 != 0;
        x >>= 1;
    }
    bits
}

pub fn bits_to_u64(bits: &[bool]) -> u64 {
    assert_eq!(bits.len(), 64);
    let mut x = 0;
    for i in (0..64).rev() {
        x <<= 1;
        x |= bits[i] as u64;
    }
    x
}

#[cfg(test)]
mod tests {
    use crate::util::*;

    #[test]
    fn test_bit_conversion() {
        assert_eq!(bits_to_u64(&u64_to_bits(3248)), 3248);
    }
}