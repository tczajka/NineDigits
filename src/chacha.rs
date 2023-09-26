pub fn chacha20_block(key: &[u32; 8], nonce: u64, counter: u64) -> [u32; 16] {
    let magic = [*b"expa", *b"nd 3", *b"2-by", *b"te k"].map(u32::from_le_bytes);
    let mut input = [0; 16];
    input[0..4].copy_from_slice(&magic);
    input[4..12].copy_from_slice(key);
    input[12] = counter as u32;
    input[13] = (counter >> 32) as u32;
    input[14] = nonce as u32;
    input[15] = (nonce >> 32) as u32;

    let mut x = input;
    for _ in 0..10 {
        for (a, b, c, d) in [
            (0, 4, 8, 12),
            (1, 5, 9, 13),
            (2, 6, 10, 14),
            (3, 7, 11, 15),
            (0, 5, 10, 15),
            (1, 6, 11, 12),
            (2, 7, 8, 13),
            (3, 4, 9, 14),
        ] {
            let (xa, xb, xc, xd) = quarter_round(x[a], x[b], x[c], x[d]);
            x[a] = xa;
            x[b] = xb;
            x[c] = xc;
            x[d] = xd;
        }
    }
    for (a, b) in x.iter_mut().zip(input.iter()) {
        *a = a.wrapping_add(*b);
    }
    x
}

fn quarter_round(mut a: u32, mut b: u32, mut c: u32, mut d: u32) -> (u32, u32, u32, u32) {
    a = a.wrapping_add(b);
    d ^= a;
    d = d.rotate_left(16);

    c = c.wrapping_add(d);
    b ^= c;
    b = b.rotate_left(12);

    a = a.wrapping_add(b);
    d ^= a;
    d = d.rotate_left(8);

    c = c.wrapping_add(d);
    b ^= c;
    b = b.rotate_left(7);

    (a, b, c, d)
}
