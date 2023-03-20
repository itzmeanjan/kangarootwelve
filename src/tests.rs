#[test]
fn test_length_encode() {
    let (enc, len) = crate::utils::length_encode(0);
    assert_eq!(enc[..len], [0x00]);

    let (enc, len) = crate::utils::length_encode(1);
    assert_eq!(enc[..len], [0x01, 0x01]);

    let (enc, len) = crate::utils::length_encode(12);
    assert_eq!(enc[..len], [0x0c, 0x01]);

    let (enc, len) = crate::utils::length_encode(65538);
    assert_eq!(enc[..len], [0x01, 0x00, 0x02, 0x03]);
}
