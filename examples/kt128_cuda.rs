//! Hashing GPU-resident data with the KT128 CUDA backend.

use kangarootwelve::{DeviceBuffer, KT128};
use rand::RngCore;

fn main() {
    const MLEN: usize = 1 << 20;
    const CSTRLEN: usize = 1;
    const DLEN: usize = 32;

    let mut msg = vec![0u8; MLEN];
    let mut cstr = vec![0u8; CSTRLEN];
    let mut dig = vec![0u8; DLEN];

    let mut rng = rand::rng();
    rng.fill_bytes(&mut msg);
    cstr[0] = 0xff;

    // 1) Upload the message to CUDA device memory.
    let dmsg = DeviceBuffer::new(&msg).expect("failed to upload message to device");

    // 2) Hash the already-device-resident data on the GPU.
    let (mut xof, elapsed) = KT128::hash_device(&dmsg, &cstr);

    // 3) Squeeze the digest, in two calls, to show the XOF is incremental.
    xof.squeeze(&mut dig[..DLEN / 2]);
    xof.squeeze(&mut dig[DLEN / 2..]);

    println!("Using KT128 (CUDA, device-resident input)");
    println!("Message length       = {MLEN} bytes");
    println!("Customization String = {}", const_hex::encode(&cstr));
    println!("Digest               = {}", const_hex::encode(&dig));
    println!("GPU absorb time      = {:.3} ms", elapsed.as_secs_f64() * 1e3);
}
