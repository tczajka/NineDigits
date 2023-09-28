use crate::simd::Simd4x32;

pub fn chacha20_block(key: &[u32; 8], nonce: u64, counter: u64) -> [u32; 16] {
    let input: [Simd4x32; 4] = [
        // Magic constant.
        Simd4x32::from_le_bytes(*b"expand 32-byte k"),
        <[u32; 4]>::try_from(&key[0..4]).unwrap().into(),
        <[u32; 4]>::try_from(&key[4..8]).unwrap().into(),
        Simd4x32::from_le_u64([counter, nonce]),
    ];

    let mut x = input;

    for _ in 0..20 {
        quarter_round(&mut x);

        x[1] = x[1].rotate_words_1();
        x[2] = x[2].rotate_words_2();
        x[3] = x[3].rotate_words_3();

        quarter_round(&mut x);

        x[1] = x[1].rotate_words_3();
        x[2] = x[2].rotate_words_2();
        x[3] = x[3].rotate_words_1();
    }

    // Add the input to the output.
    for (a, b) in x.iter_mut().zip(input.iter()) {
        *a += *b;
    }

    // Build the result.
    let mut output = [0; 16];
    for (a, b) in output.chunks_exact_mut(4).zip(x.iter()) {
        a.copy_from_slice(&<[u32; 4]>::from(*b))
    }

    output
}

fn quarter_round(x: &mut [Simd4x32; 4]) {
    x[0] += x[1];
    x[3] = (x[3] ^ x[0]).rotate_bits_16();

    x[2] += x[3];
    x[1] = (x[1] ^ x[2]).rotate_bits_12();

    x[0] += x[1];
    x[3] = (x[3] ^ x[0]).rotate_bits_8();

    x[2] += x[3];
    x[1] = (x[1] ^ x[2]).rotate_bits_7();
}
