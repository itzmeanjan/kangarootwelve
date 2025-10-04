/// KangarooTwelve splits input message into N -many equal sized chunks, each of 8kB. Only the last chunk will have length `<= CHUNK_BYTE_LEN`.
pub const CHUNK_BYTE_LEN: usize = 8 * 1024; // 8kB
