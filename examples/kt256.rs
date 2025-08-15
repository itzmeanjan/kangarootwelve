use kangarootwelve::KT256;
use rand::RngCore;

fn main() {
    const MLEN: usize = 64;
    const CSTRLEN: usize = 1;
    const DLEN: usize = 32;

    let mut msg = vec![0u8; MLEN];
    let mut cstr = vec![0u8; CSTRLEN];
    let mut dig = vec![0u8; DLEN];

    let mut rng = rand::rng();
    rng.fill_bytes(&mut msg);
    cstr[0] = 0xff;

    let mut hasher = KT256::hash(&msg, &cstr);
    hasher.squeeze(&mut dig[..DLEN / 2]);
    hasher.squeeze(&mut dig[DLEN / 2..]);

    println!("Using KT256");
    println!("Message              = {}", const_hex::encode(&msg));
    println!("Customization String = {}", const_hex::encode(&cstr));
    println!("Digest               = {}", const_hex::encode(&dig));
}
