use crate::KT128;
use std::cmp;

/// Generates static byte pattern of length 251, following
/// https://www.ietf.org/archive/id/draft-irtf-cfrg-kangarootwelve-17.html#name-test-vectors
///
/// Taken from https://github.com/itzmeanjan/turboshake/blob/81243e8e/src/tests.rs#L4-L9
#[allow(dead_code)]
fn pattern() -> [u8; 251] {
    (0..251).map(|i| i).collect::<Vec<u8>>().try_into().unwrap()
}

/// Generates bytearray of length n by repeating static byte pattern returned by `pattern()`,
/// following https://www.ietf.org/archive/id/draft-irtf-cfrg-kangarootwelve-17.html#name-test-vectors
///
/// Taken from https://github.com/itzmeanjan/turboshake/blob/81243e8e/src/tests.rs#L11-L25
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

/// Helper function for computing KT128 digest ( of length olen -bytes ) for some (M, C) pair.
#[allow(dead_code)]
fn kt128(msg: &[u8], cstr: &[u8], olen: usize) -> Vec<u8> {
    let mut out = vec![0u8; olen];
    KT128::hash(msg, cstr).squeeze(&mut out);

    out
}

/// Ensures functional correctness of KT128 ( non-incremental hashing API ) using test vectors
/// collected from https://www.ietf.org/archive/id/draft-irtf-cfrg-kangarootwelve-17.html#name-test-vectors
/// and using pseudocode https://www.ietf.org/archive/id/draft-irtf-cfrg-kangarootwelve-17.html#name-kt128
#[test]
fn known_answer_test_for_kt128() {
    assert_eq!(
        const_hex::encode(kt128(&[], &[], 32)),
        "1ac2d450fc3b4205d19da7bfca1b37513c0803577ac7167f06fe2ce1f0ef39e5"
    );

    assert_eq!(
        const_hex::encode(kt128(&[], &[], 64)),
        "1ac2d450fc3b4205d19da7bfca1b37513c0803577ac7167f06fe2ce1f0ef39e54269c056b8c82e48276038b6d292966cc07a3d4645272e31ff38508139eb0a71"
    );

    assert_eq!(
        const_hex::encode(&kt128(&[], &[], 10032)[10000..]),
        "e8dc563642f7228c84684c898405d3a834799158c079b12880277a1d28e2ff6d"
    );

    assert_eq!(
        const_hex::encode(kt128(&ptn(17usize.pow(0)), &[], 32)),
        "2bda92450e8b147f8a7cb629e784a058efca7cf7d8218e02d345dfaa65244a1f"
    );

    assert_eq!(
        const_hex::encode(kt128(&ptn(17usize.pow(1)), &[], 32)),
        "6bf75fa2239198db4772e36478f8e19b0f371205f6a9a93a273f51df37122888"
    );

    assert_eq!(
        const_hex::encode(kt128(&ptn(17usize.pow(2)), &[], 32)),
        "0c315ebcdedbf61426de7dcf8fb725d1e74675d7f5327a5067f367b108ecb67c"
    );

    assert_eq!(
        const_hex::encode(kt128(&ptn(17usize.pow(3)), &[], 32)),
        "cb552e2ec77d9910701d578b457ddf772c12e322e4ee7fe417f92c758f0d59d0"
    );

    assert_eq!(
        const_hex::encode(kt128(&ptn(17usize.pow(4)), &[], 32)),
        "8701045e22205345ff4dda05555cbb5c3af1a771c2b89baef37db43d9998b9fe"
    );

    assert_eq!(
        const_hex::encode(kt128(&ptn(17usize.pow(5)), &[], 32)),
        "844d610933b1b9963cbdeb5ae3b6b05cc7cbd67ceedf883eb678a0a8e0371682"
    );

    assert_eq!(
        const_hex::encode(kt128(&ptn(17usize.pow(6)), &[], 32)),
        "3c390782a8a4e89fa6367f72feaaf13255c8d95878481d3cd8ce85f58e880af8"
    );

    assert_eq!(
        const_hex::encode(kt128(&[0xff; 0], &ptn(41usize.pow(0)), 32)),
        "fab658db63e94a246188bf7af69a133045f46ee984c56e3c3328caaf1aa1a583"
    );

    assert_eq!(
        const_hex::encode(kt128(&[0xff; 1], &ptn(41usize.pow(1)), 32)),
        "d848c5068ced736f4462159b9867fd4c20b808acc3d5bc48e0b06ba0a3762ec4"
    );

    assert_eq!(
        const_hex::encode(kt128(&[0xff; 3], &ptn(41usize.pow(2)), 32)),
        "c389e5009ae57120854c2e8c64670ac01358cf4c1baf89447a724234dc7ced74"
    );

    assert_eq!(
        const_hex::encode(kt128(&[0xff; 7], &ptn(41usize.pow(3)), 32)),
        "75d2f86a2e644566726b4fbcfc5657b9dbcf070c7b0dca06450ab291d7443bcf"
    );

    assert_eq!(
        const_hex::encode(kt128(&ptn(8191), &[], 32)),
        "1b577636f723643e990cc7d6a659837436fd6a103626600eb8301cd1dbe553d6"
    );

    assert_eq!(
        const_hex::encode(kt128(&ptn(8192), &[], 32)),
        "48f256f6772f9edfb6a8b661ec92dc93b95ebd05a08a17b39ae3490870c926c3"
    );

    assert_eq!(
        const_hex::encode(kt128(&ptn(8192), &ptn(8189), 32)),
        "3ed12f70fb05ddb58689510ab3e4d23c6c6033849aa01e1d8c220a297fedcd0b"
    );

    assert_eq!(
        const_hex::encode(kt128(&ptn(8192), &ptn(8190), 32)),
        "6a7c1b6a5cd0d8c9ca943a4a216cc64604559a2ea45f78570a15253d67ba00ae"
    );
}
