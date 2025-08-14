use crate::{KT128, KT256};
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

/// Helper function for computing KT256 digest ( of length olen -bytes ) for some (M, C) pair.
#[allow(dead_code)]
fn kt256(msg: &[u8], cstr: &[u8], olen: usize) -> Vec<u8> {
    let mut out = vec![0u8; olen];
    KT256::hash(msg, cstr).squeeze(&mut out);

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

/// Ensures functional correctness of KT256 ( non-incremental hashing API ) using test vectors
/// collected from https://www.ietf.org/archive/id/draft-irtf-cfrg-kangarootwelve-17.html#name-test-vectors
/// and using pseudocode https://www.ietf.org/archive/id/draft-irtf-cfrg-kangarootwelve-17.html#name-kt256
#[test]
fn known_answer_test_for_kt256() {
    assert_eq!(
        const_hex::encode(kt256(&[], &[], 64)),
        "b23d2e9cea9f4904e02bec06817fc10ce38ce8e93ef4c89e6537076af8646404e3e8b68107b8833a5d30490aa33482353fd4adc7148ecb782855003aaebde4a9"
    );

    assert_eq!(
        const_hex::encode(kt256(&[], &[], 128)),
        "b23d2e9cea9f4904e02bec06817fc10ce38ce8e93ef4c89e6537076af8646404e3e8b68107b8833a5d30490aa33482353fd4adc7148ecb782855003aaebde4a9b0925319d8ea1e121a609821ec19efea89e6d08daee1662b69c840289f188ba860f55760b61f82114c030c97e5178449608ccd2cd2d919fc7829ff69931ac4d0"
    );

    assert_eq!(
        const_hex::encode(&kt256(&[], &[], 10064)[10000..]),
        "ad4a1d718cf950506709a4c33396139b4449041fc79a05d68da35f1e453522e056c64fe94958e7085f2964888259b9932752f3ccd855288efee5fcbb8b563069"
    );

    assert_eq!(
        const_hex::encode(kt256(&ptn(17usize.pow(0)), &[], 64)),
        "0d005a194085360217128cf17f91e1f71314efa5564539d444912e3437efa17f82db6f6ffe76e781eaa068bce01f2bbf81eacb983d7230f2fb02834a21b1ddd0"
    );

    assert_eq!(
        const_hex::encode(kt256(&ptn(17usize.pow(1)), &[], 64)),
        "1ba3c02b1fc514474f06c8979978a9056c8483f4a1b63d0dccefe3a28a2f323e1cdcca40ebf006ac76ef0397152346837b1277d3e7faa9c9653b19075098527b"
    );

    assert_eq!(
        const_hex::encode(kt256(&ptn(17usize.pow(2)), &[], 64)),
        "de8ccbc63e0f133ebb4416814d4c66f691bbf8b6a61ec0a7700f836b086cb029d54f12ac7159472c72db118c35b4e6aa213c6562caaa9dcc518959e69b10f3ba"
    );

    assert_eq!(
        const_hex::encode(kt256(&ptn(17usize.pow(3)), &[], 64)),
        "647efb49fe9d717500171b41e7f11bd491544443209997ce1c2530d15eb1ffbb598935ef954528ffc152b1e4d731ee2683680674365cd191d562bae753b84aa5"
    );

    assert_eq!(
        const_hex::encode(kt256(&ptn(17usize.pow(4)), &[], 64)),
        "b06275d284cd1cf205bcbe57dccd3ec1ff6686e3ed15776383e1f2fa3c6ac8f08bf8a162829db1a44b2a43ff83dd89c3cf1ceb61ede659766d5ccf817a62ba8d"
    );

    assert_eq!(
        const_hex::encode(kt256(&ptn(17usize.pow(5)), &[], 64)),
        "9473831d76a4c7bf77ace45b59f1458b1673d64bcd877a7c66b2664aa6dd149e60eab71b5c2bab858c074ded81ddce2b4022b5215935c0d4d19bf511aeeb0772"
    );

    assert_eq!(
        const_hex::encode(kt256(&ptn(17usize.pow(6)), &[], 64)),
        "0652b740d78c5e1f7c8dcc1777097382768b7ff38f9a7a20f29f413bb1b3045b31a5578f568f911e09cf44746da84224a5266e96a4a535e871324e4f9c7004da"
    );

    assert_eq!(
        const_hex::encode(kt256(&[0xff; 0], &ptn(41usize.pow(0)), 64)),
        "9280f5cc39b54a5a594ec63de0bb99371e4609d44bf845c2f5b8c316d72b159811f748f23e3fabbe5c3226ec96c62186df2d33e9df74c5069ceecbb4dd10eff6"
    );

    assert_eq!(
        const_hex::encode(kt256(&[0xff; 1], &ptn(41usize.pow(1)), 64)),
        "47ef96dd616f200937aa7847e34ec2feae8087e3761dc0f8c1a154f51dc9ccf845d7adbce57ff64b639722c6a1672e3bf5372d87e00aff89be97240756998853"
    );

    assert_eq!(
        const_hex::encode(kt256(&[0xff; 3], &ptn(41usize.pow(2)), 64)),
        "3b48667a5051c5966c53c5d42b95de451e05584e7806e2fb765eda959074172cb438a9e91dde337c98e9c41bed94c4e0aef431d0b64ef2324f7932caa6f54969"
    );

    assert_eq!(
        const_hex::encode(kt256(&[0xff; 7], &ptn(41usize.pow(3)), 64)),
        "e0911cc00025e1540831e266d94add9b98712142b80d2629e643aac4efaf5a3a30a88cbf4ac2a91a2432743054fbcc9897670e86ba8cec2fc2ace9c966369724"
    );

    assert_eq!(
        const_hex::encode(kt256(&ptn(8191), &[], 64)),
        "3081434d93a4108d8d8a3305b89682cebedc7ca4ea8a3ce869fbb73cbe4a58eef6f24de38ffc170514c70e7ab2d01f03812616e863d769afb3753193ba045b20"
    );

    assert_eq!(
        const_hex::encode(kt256(&ptn(8192), &[], 64)),
        "c6ee8e2ad3200c018ac87aaa031cdac22121b412d07dc6e0dccbb53423747e9a1c18834d99df596cf0cf4b8dfafb7bf02d139d0c9035725adc1a01b7230a41fa"
    );

    assert_eq!(
        const_hex::encode(kt256(&ptn(8192), &ptn(8189), 64)),
        "74e47879f10a9c5d11bd2da7e194fe57e86378bf3c3f7448eff3c576a0f18c5caae0999979512090a7f348af4260d4de3c37f1ecaf8d2c2c96c1d16c64b12496"
    );

    assert_eq!(
        const_hex::encode(kt256(&ptn(8192), &ptn(8190), 64)),
        "f4b5908b929ffe01e0f79ec2f21243d41a396b2e7303a6af1d6399cd6c7a0a2dd7c4f607e8277f9c9b1cb4ab9ddc59d4b92d1fc7558441f1832c3279a4241b8b"
    );
}
