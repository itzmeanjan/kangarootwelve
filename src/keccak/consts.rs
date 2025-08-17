use turboshake::keccak;

/// Keccak-p[1600] to run applied 12 times, same as TurboSHAKE https://github.com/itzmeanjan/turboshake/blob/ddc435053f9194d5d54b092604be89b023ddecaf/src/keccak.rs#L10-L11.
pub const ROUNDS: usize = 12;

/// Keccak permutation lane rotation bit factor table, taken from https://github.com/itzmeanjan/turboshake/blob/ddc435053f9194d5d54b092604be89b023ddecaf/src/keccak.rs#L16-L17.
pub const ROT: [i32; keccak::LANE_CNT] = [0, 1, 62, 28, 27, 36, 44, 6, 55, 20, 3, 10, 43, 25, 39, 41, 45, 15, 21, 8, 18, 2, 61, 56, 14];

/// Keccak-p[1600] permutation round constants, taken from https://github.com/itzmeanjan/turboshake/blob/ddc435053f9194d5d54b092604be89b023ddecaf/src/keccak.rs#L19-L20.
pub const RC: [i64; ROUNDS] = [
    0x000000008000808bu64 as i64,
    0x800000000000008bu64 as i64,
    0x8000000000008089u64 as i64,
    0x8000000000008003u64 as i64,
    0x8000000000008002u64 as i64,
    0x8000000000000080u64 as i64,
    0x000000000000800au64 as i64,
    0x800000008000000au64 as i64,
    0x8000000080008081u64 as i64,
    0x8000000000008080u64 as i64,
    0x0000000080000001u64 as i64,
    0x8000000080008008u64 as i64,
];
