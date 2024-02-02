use solana_program::pubkey::Pubkey;

pub mod initial_authority {
    #[cfg(feature = "testing")]
    sanctum_macros::declare_program_keys!("9S3avfRxH9RYbMHbvxnhwiwpdF9iuXG7uWiatqWvQskT", []);

    // TODO: set actual initial authority key
    #[cfg(not(feature = "testing"))]
    sanctum_macros::declare_program_keys!("TH1S1SNoTAVAL1DPUBKEYDoNoTUSE11111111111111", []);
}

pub const CURRENT_PROGRAM_VERS: u8 = 1;

/// 10% of trading fees
pub const DEFAULT_TRADING_PROTOCOL_FEE_BPS: u16 = 1_000;

/// 10% of LP withdrawal fees
pub const DEFAULT_LP_PROTOCOL_FEE_BPS: u16 = 1_000;

/// TODO: SET TO FLAT FEE PRICING PROGRAM ID
pub const DEFAULT_PRICING_PROGRAM: Pubkey = Pubkey::new_from_array([0; 32]);
