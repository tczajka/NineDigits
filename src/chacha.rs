use crate::simd128::{Simd16x8, Simd2x64, Simd4x32};

pub fn chacha20_block(key: &[u8; 32], nonce: u64, counter: u64) -> [u8; 64] {
    let input: [Simd4x32; 4] = [
        // Magic constant.
        Simd16x8::from(*b"expand 32-byte k").into(),
        Simd16x8::from(<[u8; 16]>::try_from(&key[0..16]).unwrap()).into(),
        Simd16x8::from(<[u8; 16]>::try_from(&key[16..32]).unwrap()).into(),
        Simd2x64::from([counter, nonce]).into(),
    ];

    let mut x = input;

    for _ in 0..10 {
        quarter_round(&mut x);

        x[1] = x[1].rotate_words_3();
        x[2] = x[2].rotate_words_2();
        x[3] = x[3].rotate_words_1();

        quarter_round(&mut x);

        x[1] = x[1].rotate_words_1();
        x[2] = x[2].rotate_words_2();
        x[3] = x[3].rotate_words_3();
    }

    // Add the input to the output.
    for (a, b) in x.iter_mut().zip(input.iter()) {
        *a += *b;
    }

    // Build the result.
    let mut output = [0u8; 64];
    for (a, b) in output.chunks_exact_mut(16).zip(x.iter()) {
        a.copy_from_slice(&<[u8; 16]>::from(Simd16x8::from(*b)))
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
