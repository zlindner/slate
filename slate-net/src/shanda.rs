/// Encrypt bytes using Maplestory's custom encryption algorithm.
pub fn encrypt(data: &mut [u8]) {
    let size: usize = data.len();
    let mut c: u8;
    let mut a: u8;
    for _ in 0..3 {
        a = 0;
        for j in (1..(size + 1)).rev() {
            c = data[size - j];
            c = rotl(c, 3);
            c = (c as usize).overflowing_add(j).0 as u8;
            c ^= a;
            a = c;
            c = rotr(a, j as u32);
            c ^= 0xFF;
            c = c.overflowing_add(0x48).0;
            data[size - j] = c;
        }
        a = 0;
        for j in (1..(size + 1)).rev() {
            c = data[j - 1];
            c = rotl(c, 4);
            c = (c as usize).overflowing_add(j).0 as u8;
            c ^= a;
            a = c;
            c ^= 0x13;
            c = rotr(c, 3);
            data[j - 1] = c;
        }
    }
}

/// Decrypt bytes encrypted with Maplestory's custom encryption algorithm.
pub fn decrypt(data: &mut [u8]) {
    let size: usize = data.len();
    let mut a: u8;
    let mut b: u8;
    let mut c: u8;
    for _ in 0..3 {
        b = 0;
        for j in (1..(size + 1)).rev() {
            c = data[j - 1];
            c = rotl(c, 3);
            c ^= 0x13;
            a = c;
            c ^= b;
            c = (c as usize).overflowing_sub(j).0 as u8;
            c = rotr(c, 4);
            b = a;
            data[j - 1] = c;
        }
        b = 0;
        for j in (1..(size + 1)).rev() {
            c = data[size - j];
            c = c.overflowing_sub(0x48).0;
            c ^= 0xFF;
            c = rotl(c, j as u32);
            a = c;
            c ^= b;
            c = (c as usize).overflowing_sub(j).0 as u8;
            c = rotr(c, 3);
            b = a;
            data[size - j] = c;
        }
    }
}

/// Roll a byte left count times
fn rotl(byte: u8, count: u32) -> u8 {
    let count = count % 8;
    if count > 0 {
        (byte << count) | (byte >> (8 - count))
    } else {
        byte
    }
}

/// Roll a byte right count times
fn rotr(byte: u8, count: u32) -> u8 {
    let count = count % 8;
    if count > 0 {
        (byte >> count) | (byte << (8 - count))
    } else {
        byte
    }
}
