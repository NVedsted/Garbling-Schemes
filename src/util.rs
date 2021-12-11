use aes::Block;

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

pub fn xor_blocks(a: &Block, b: &Block) -> Block {
    let mut block: Block = Default::default();
    block.iter_mut()
        .zip(a.iter().zip(b.iter()))
        .for_each(|(dst, (a, b))| *dst = a ^ b);
    block
}

// TODO: consider inline
pub fn get_lsb(s: &[u8]) -> bool {
    s[0] & 1 != 0
}

// TODO: consider inline
pub fn set_lsb(s: &mut [u8], b: bool) {
    if b {
        s[0] |= 1;
    } else {
        s[0] &= 0;
    }
}

#[cfg(test)]
mod tests {
    use rand::{RngCore, thread_rng};

    use crate::util::*;

    #[test]
    fn test_bit_conversion() {
        assert_eq!(bits_to_u64(&u64_to_bits(3248)), 3248);
    }

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
}