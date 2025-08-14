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
    let l = ((bw + 7) / 8) as usize;

    for i in 0..l {
        res[l - i - 1] = (x >> (i * 8)) as u8;
    }
    res[l] = l as u8;

    (res, l + 1)
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
