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

pub fn u8_to_bits(mut x: u8) -> Vec<bool> {
    let mut bits = vec![false; 8];
    for i in (0..8).rev() {
        bits[i] = x & 1 != 0;
        x >>= 1;
    }
    bits
}

pub fn bits_to_u8(bits: &[bool]) -> u8 {
    assert_eq!(bits.len(), 8);
    let mut x = 0;
    for i in 0..8 {
        x <<= 1;
        x |= bits[i] as u8;
    }
    x
}

#[cfg(test)]
mod tests {
    use crate::util::*;

    #[test]
    fn test_u64_conversion() {
        assert_eq!(bits_to_u64(&u64_to_bits(3248)), 3248);
    }

    #[test]
    fn test_u8_conversion() {
        assert_eq!(bits_to_u8(&u8_to_bits(123)), 123);
    }
}