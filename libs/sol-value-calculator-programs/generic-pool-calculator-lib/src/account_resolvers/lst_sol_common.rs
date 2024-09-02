use generic_pool_calculator_interface::GenericPoolCalculatorError;
use solana_program::pubkey::Pubkey;
use solana_readonly_account::{pubkey::ReadonlyAccountPubkey, ReadonlyAccountData};

use crate::{utils::read_programdata_addr, GenericPoolSolValCalc, LstSolCommonKeys};

/// NB: This struct requires a impl-specific resolver to resolve to in order to derive
/// lst from pool_state and check them
pub struct LstSolCommonIntermediateArgs<Q: ReadonlyAccountPubkey + ReadonlyAccountData> {
    pub lst_mint: Pubkey,
    pub pool_state: Pubkey,
    pub pool_program: Q,
}

impl<Q: ReadonlyAccountPubkey + ReadonlyAccountData> LstSolCommonIntermediateArgs<Q> {
    pub fn resolve<P: GenericPoolSolValCalc>(
        self,
    ) -> Result<LstSolCommonKeys, GenericPoolCalculatorError> {
        if self.pool_program.pubkey() != P::POOL_PROGRAM_ID {
            return Err(GenericPoolCalculatorError::WrongPoolProgram);
        }
        Ok(LstSolCommonKeys {
            lst_mint: self.lst_mint,
            pool_state: self.pool_state,
            state: P::CALCULATOR_STATE_PDA,
            pool_program: P::POOL_PROGRAM_ID,
            pool_program_data: read_programdata_addr(&self.pool_program)?,
        })
    }
}

/// Struct that uses defined const for POOL_PROGRAM_PROGDATA
/// so that it can be used without fetching POOL_PROGRAM
///
/// NB: This struct requires a impl-specific resolver to resolve to in order to derive
/// lst from pool_state and check them
pub struct LstSolCommonIntermediateKeys {
    pub lst_mint: Pubkey,
    pub pool_state: Pubkey,
}

impl LstSolCommonIntermediateKeys {
    pub const fn resolve<P: GenericPoolSolValCalc>(self) -> LstSolCommonKeys {
        LstSolCommonKeys {
            lst_mint: self.lst_mint,
            pool_state: self.pool_state,
            state: P::CALCULATOR_STATE_PDA,
            pool_program: P::POOL_PROGRAM_ID,
            pool_program_data: P::POOL_PROGRAM_PROGDATA_ID,
        }
    }
}
