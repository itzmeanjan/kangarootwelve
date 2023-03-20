use turboshake::sponge;

#[derive(Copy, Clone)]
struct KangarooTwelve {
    state: [u64; 25],
    is_ready: usize,
    squeezable: usize,
}

impl KangarooTwelve {
    const CAPACITY_BITS: usize = 256;
    const RATE_BITS: usize = 1600 - Self::CAPACITY_BITS;
    const RATE_BYTES: usize = Self::RATE_BITS / 8;
    const RATE_WORDS: usize = Self::RATE_BYTES / 8;

    /// Create a new instance of K12 Extendable Output Function (XOF), into which
    /// arbitrary number of message bytes can be absorbed and arbitrary many bytes
    /// can be squeezed out.
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            state: [0u64; 25],
            is_ready: usize::MIN,
            squeezable: 0,
        }
    }

    /// Given that N -bytes input message is already absorbed into sponge state, this
    /// routine is used for squeezing M -bytes out of consumable part of sponge state
    /// ( i.e. rate portion of the state )
    ///
    /// Note, this routine can be called arbitrary number of times, for squeezing arbitrary
    /// number of bytes from sponge Keccak\[256\].
    ///
    /// Make sure you absorb message bytes first, then only call this function, otherwise
    /// it can't squeeze anything out.
    ///
    /// Adapted from https://github.com/itzmeanjan/turboshake/blob/81243e8ebe792b8af53abf6b8a9dae6744949896/src/turboshake128.rs#L87-L109
    #[inline(always)]
    pub fn squeeze(&mut self, out: &mut [u8]) {
        if self.is_ready != usize::MAX {
            return;
        }

        sponge::squeeze::<{ Self::RATE_BYTES }, { Self::RATE_WORDS }>(
            &mut self.state,
            &mut self.squeezable,
            out,
        );
    }
}
