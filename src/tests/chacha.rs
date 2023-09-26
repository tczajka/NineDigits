use crate::chacha::chacha20_block;

#[test]
fn test_chacha20() {
    // https://datatracker.ietf.org/doc/html/draft-agl-tls-chacha20poly1305-04

    #[rustfmt::skip]
    let key = [
        0x00010203, 0x04050607, 0x08090a0b, 0x0c0d0e0f,
        0x10111213, 0x14151617, 0x18191a1b, 0x1c1d1e1f
    ].map(|x| u32::from_le_bytes(u32::to_be_bytes(x)));

    let nonce = u64::from_le_bytes(u64::to_be_bytes(0x0001020304050607));

    #[rustfmt::skip]
    let expected_0 = [
        0xf798a189, 0xf195e669, 0x82105ffb, 0x640bb775,
        0x7f579da3, 0x1602fc93, 0xec01ac56, 0xf85ac3c1,
        0x34a4547b, 0x733b4641, 0x3042c944, 0x00491769,
        0x05d3be59, 0xea1c53f1, 0x5916155c, 0x2be8241a,
    ].map(|x| u32::from_le_bytes(u32::to_be_bytes(x)));

    #[rustfmt::skip]
    let expected_1 = [
        0x38008b9a, 0x26bc3594, 0x1e244417, 0x7c8ade66,
        0x89de9526, 0x4986d958, 0x89fb60e8, 0x4629c9bd,
        0x9a5acb1c, 0xc118be56, 0x3eb9b3a4, 0xa472f82e,
        0x09a7e778, 0x492b562e, 0xf7130e88, 0xdfe031c7,
    ].map(|x| u32::from_le_bytes(u32::to_be_bytes(x)));

    assert_eq!(chacha20_block(&key, nonce, 0), expected_0);
    assert_eq!(chacha20_block(&key, nonce, 1), expected_1);
}
