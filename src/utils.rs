use std::cmp;

/// Given an unsigned integer ( byte length of input message, which can be zero ),
/// this routine encodes it following algorithm 1 ( on page 5 ) of K12 specification
/// https://keccak.team/files/KangarooTwelve.pdf returning encoded byte array ( of
/// length at max sizeof(usize) + 1 ) and effective length `len` <= sizeof(usize) + 1.
///
/// sizeof(usize) = 4, on 32 -bit targets while it's 8 -bytes wide on 64 -bit targets.
///
/// In case x = 0, returned byte array will have only single byte of interest i.e. effective
/// byte length of 1. While for x = 1, first two bytes of returned array will be of interest
/// i.e. effective byte length of 2.
#[inline(always)]
pub fn length_encode(x: usize) -> ([u8; core::mem::size_of::<usize>() + 1], usize) {
    let mut res = [0u8; core::mem::size_of::<usize>() + 1];

    let bw = usize::MAX.count_ones() - x.leading_zeros();
    let l = bw.div_ceil(8) as usize;

    for i in 0..l {
        res[l - i - 1] = (x >> (i * 8)) as u8;
    }
    res[l] = l as u8;

    (res, l + 1)
}

/// Given message (M), customization string (C) and length of C encoded using `length_encode()`
/// function ( s.t. only first `elen` bytes are of interest ), this routine extracts out `i` -th
/// chunk ( s.t. each chunk is `B` -bytes wide ) along with how many ( must be <= B ) bytes
/// of that chunk are of significance.
///
/// For understanding how this function works, let us assume
///
/// S <- M || C || length_encode(|C|) s.t. |C| <- byte length of C
///
/// We can split S into `n` -chunks s.t. first (n - 1) chunks are of length B while the
/// last one is of length <= B. So n = ⌈|S|/ B⌉
///
/// n must be 1, because it's guaranteed that S will be atleast 1 -byte wide even if both M
/// and C are empty. Then it must be the case that 0 <= i < n.
///
/// You may want to take a look at section 3.{2, 3} of the K12 specification
/// https://keccak.team/files/KangarooTwelve.pdf for understanding why this function exists.
#[inline(always)]
pub fn get_ith_chunk<const B: usize>(i: usize, msg: &[u8], cstr: &[u8], enc: &[u8]) -> ([u8; B], usize) {
    let l0 = msg.len();
    let l1 = l0 + cstr.len();
    let l2 = l1 + enc.len();

    let mut res = [0u8; B];
    let mut off = 0;

    let start_at = i * B;

    if start_at < l0 {
        let readable = cmp::min(l0 - start_at, B);
        res[..readable].copy_from_slice(&msg[start_at..(start_at + readable)]);

        off += readable;
    }

    if (off < B) && ((start_at + off) < l1) {
        let readable = cmp::min(l1 - (start_at + off), B - off);
        let tmp = (start_at + off) - l0;
        res[off..(off + readable)].copy_from_slice(&cstr[tmp..(tmp + readable)]);

        off += readable;
    }

    if (off < B) && ((start_at + off) < l2) {
        let readable = cmp::min(l2 - (start_at + off), B - off);
        let tmp = (start_at + off) - l1;
        res[off..(off + readable)].copy_from_slice(&enc[tmp..(tmp + readable)]);

        off += readable;
    }

    (res, off)
}

#[cfg(test)]
mod test {
    use crate::utils::length_encode;

    #[test]
    fn test_length_encode() {
        let (enc, len) = length_encode(0);
        assert_eq!(enc[..len], [0x00]);

        let (enc, len) = length_encode(1);
        assert_eq!(enc[..len], [0x01, 0x01]);

        let (enc, len) = length_encode(12);
        assert_eq!(enc[..len], [0x0c, 0x01]);

        let (enc, len) = length_encode(65538);
        assert_eq!(enc[..len], [0x01, 0x00, 0x02, 0x03]);
    }
}
