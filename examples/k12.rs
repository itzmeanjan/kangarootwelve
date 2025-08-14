use kangarootwelve::KangarooTwelve;
use rand::{RngCore, thread_rng};

fn main() {
    const MLEN: usize = 64;
    const CSTRLEN: usize = 1;
    const DLEN: usize = 32;

    let mut msg = vec![0u8; MLEN];
    let mut cstr = vec![0u8; CSTRLEN];
    let mut dig = vec![0u8; DLEN];

    let mut rng = thread_rng();
    rng.fill_bytes(&mut msg);
    cstr[0] = 0xff;

    let mut hasher = KangarooTwelve::hash(&msg, &cstr);
    hasher.squeeze(&mut dig[..DLEN / 2]);
    hasher.squeeze(&mut dig[DLEN / 2..]);

    println!("Message              = {}", hex::encode(&msg));
    println!("Customization String = {}", hex::encode(&cstr));
    println!("Digest               = {}", hex::encode(&dig));
}
