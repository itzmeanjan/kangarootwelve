use crate::KangarooTwelve;
use std::cmp;

/// Ensures functional correctness of length encoder function.
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

/// Generates static byte pattern of length 251, following
/// https://www.ietf.org/archive/id/draft-irtf-cfrg-kangarootwelve-09.html#name-test-vectors
///
/// Taken from https://github.com/itzmeanjan/turboshake/blob/81243e8ebe792b8af53abf6b8a9dae6744949896/src/tests.rs#L4-L9
#[allow(dead_code)]
fn pattern() -> [u8; 251] {
    (0..251).map(|i| i).collect::<Vec<u8>>().try_into().unwrap()
}

/// Generates bytearray of length n by repeating static byte pattern returned by `pattern()`,
/// following https://www.ietf.org/archive/id/draft-irtf-cfrg-kangarootwelve-09.html#name-test-vectors
///
/// Taken from https://github.com/itzmeanjan/turboshake/blob/81243e8ebe792b8af53abf6b8a9dae6744949896/src/tests.rs#L11-L25
#[allow(dead_code)]
fn ptn(n: usize) -> Vec<u8> {
    let mut res = vec![0; n];

    let mut off = 0;
    while off < n {
        let read = cmp::min(n - off, 251);
        res[off..(off + read)].copy_from_slice(&pattern()[..read]);
        off += read;
    }

    res
}

/// Helper function for computing K12 digest ( of length olen -bytes ) for some (M, C) pair.
#[allow(dead_code)]
fn k12(msg: &[u8], cstr: &[u8], olen: usize) -> Vec<u8> {
    let mut out = vec![0u8; olen];
    KangarooTwelve::hash(msg, cstr).squeeze(&mut out);

    out
}

/// Ensures functions correctness of K12 ( non-incremental hashing API ) using test vectors
/// collected from https://www.ietf.org/archive/id/draft-irtf-cfrg-kangarootwelve-09.html#name-test-vectors
/// and using pseudocode https://www.ietf.org/archive/id/draft-irtf-cfrg-kangarootwelve-09.html#name-kangarootwelve
#[test]
fn test_kangaroo_twelve() {
    assert_eq!(
        hex::encode(k12(&[], &[], 32)),
        "1ac2d450fc3b4205d19da7bfca1b37513c0803577ac7167f06fe2ce1f0ef39e5"
    );

    assert_eq!(
        hex::encode(k12(&[], &[], 64)),
        "1ac2d450fc3b4205d19da7bfca1b37513c0803577ac7167f06fe2ce1f0ef39e54269c056b8c82e48276038b6d292966cc07a3d4645272e31ff38508139eb0a71"
    );

    assert_eq!(
        hex::encode(&k12(&[], &[], 10032)[10000..]),
        "e8dc563642f7228c84684c898405d3a834799158c079b12880277a1d28e2ff6d"
    );

    assert_eq!(
        hex::encode(k12(&ptn(17usize.pow(0)), &[], 32)),
        "2bda92450e8b147f8a7cb629e784a058efca7cf7d8218e02d345dfaa65244a1f"
    );

    assert_eq!(
        hex::encode(k12(&ptn(17usize.pow(1)), &[], 32)),
        "6bf75fa2239198db4772e36478f8e19b0f371205f6a9a93a273f51df37122888"
    );

    assert_eq!(
        hex::encode(k12(&ptn(17usize.pow(2)), &[], 32)),
        "0c315ebcdedbf61426de7dcf8fb725d1e74675d7f5327a5067f367b108ecb67c"
    );

    assert_eq!(
        hex::encode(k12(&ptn(17usize.pow(3)), &[], 32)),
        "cb552e2ec77d9910701d578b457ddf772c12e322e4ee7fe417f92c758f0d59d0"
    );

    assert_eq!(
        hex::encode(k12(&ptn(17usize.pow(4)), &[], 32)),
        "8701045e22205345ff4dda05555cbb5c3af1a771c2b89baef37db43d9998b9fe"
    );

    assert_eq!(
        hex::encode(k12(&ptn(17usize.pow(5)), &[], 32)),
        "844d610933b1b9963cbdeb5ae3b6b05cc7cbd67ceedf883eb678a0a8e0371682"
    );

    assert_eq!(
        hex::encode(k12(&[0xff; 0], &ptn(41usize.pow(0)), 32)),
        "fab658db63e94a246188bf7af69a133045f46ee984c56e3c3328caaf1aa1a583"
    );

    assert_eq!(
        hex::encode(k12(&[0xff; 1], &ptn(41usize.pow(1)), 32)),
        "d848c5068ced736f4462159b9867fd4c20b808acc3d5bc48e0b06ba0a3762ec4"
    );

    assert_eq!(
        hex::encode(k12(&[0xff; 3], &ptn(41usize.pow(2)), 32)),
        "c389e5009ae57120854c2e8c64670ac01358cf4c1baf89447a724234dc7ced74"
    );

    assert_eq!(
        hex::encode(k12(&[0xff; 7], &ptn(41usize.pow(3)), 32)),
        "75d2f86a2e644566726b4fbcfc5657b9dbcf070c7b0dca06450ab291d7443bcf"
    );
}
