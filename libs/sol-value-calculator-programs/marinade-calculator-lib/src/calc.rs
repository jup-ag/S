use marinade_calculator_interface::{
    FeeCents, MarinadeCalculatorError, MarinadeState, StakeSystem, ValidatorSystem,
};
use sanctum_token_ratio::{AmtsAfterFee, MathError, U64FeeFloor, U64RatioFloor, U64ValueRange};
use sol_value_calculator_lib::SolValueCalculator;
use solana_program::program_error::ProgramError;

pub const MAX_BP_CENTS: u32 = 1_000_000;

/// Parameters from MarinadeState required to calculate SOL value
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarinadeStateCalc {
    pub paused: bool,
    pub delayed_unstake_cooling_down: u64,
    pub emergency_cooling_down: u64,
    pub total_active_balance: u64,
    pub available_reserve_balance: u64,
    pub circulating_ticket_balance: u64,
    pub msol_supply: u64,
    pub delayed_unstake_fee_bp_cents: u32,
}

impl From<&MarinadeState> for MarinadeStateCalc {
    fn from(
        MarinadeState {
            paused,
            available_reserve_balance,
            validator_system:
                ValidatorSystem {
                    total_active_balance,
                    ..
                },
            emergency_cooling_down,
            stake_system:
                StakeSystem {
                    delayed_unstake_cooling_down,
                    ..
                },
            circulating_ticket_balance,
            msol_supply,
            delayed_unstake_fee:
                FeeCents {
                    bp_cents: delayed_unstake_fee_bp_cents,
                },
            ..
        }: &MarinadeState,
    ) -> Self {
        Self {
            paused: *paused,
            delayed_unstake_cooling_down: *delayed_unstake_cooling_down,
            emergency_cooling_down: *emergency_cooling_down,
            total_active_balance: *total_active_balance,
            available_reserve_balance: *available_reserve_balance,
            circulating_ticket_balance: *circulating_ticket_balance,
            msol_supply: *msol_supply,
            delayed_unstake_fee_bp_cents: *delayed_unstake_fee_bp_cents,
        }
    }
}

impl From<MarinadeState> for MarinadeStateCalc {
    fn from(marinade_state: MarinadeState) -> Self {
        (&marinade_state).into()
    }
}

/// Reference
/// https://github.com/marinade-finance/liquid-staking-program/blob/26147376b75d8c971963da458623e646f2795e15/programs/marinade-finance/src/state/mod.rs#L96
impl MarinadeStateCalc {
    pub const fn verify_marinade_not_paused(&self) -> Result<(), MarinadeCalculatorError> {
        if self.paused {
            Err(MarinadeCalculatorError::MarinadePaused)
        } else {
            Ok(())
        }
    }

    pub const fn total_cooling_down(&self) -> Option<u64> {
        self.delayed_unstake_cooling_down
            .checked_add(self.emergency_cooling_down)
    }

    pub fn total_lamports_under_control(&self) -> Option<u64> {
        let tcd = self.total_cooling_down()?;
        self.total_active_balance
            .checked_add(tcd)
            .and_then(|x| x.checked_add(self.available_reserve_balance))
    }

    pub fn total_virtual_staked_lamports(&self) -> Option<u64> {
        Some(
            self.total_lamports_under_control()?
                .saturating_sub(self.circulating_ticket_balance),
        )
    }

    pub fn msol_to_sol_ratio(&self) -> Option<U64RatioFloor<u64, u64>> {
        Some(U64RatioFloor {
            num: self.total_virtual_staked_lamports()?,
            denom: self.msol_supply,
        })
    }

    pub const fn delayed_unstake_fee(&self) -> U64FeeFloor<u32, u32> {
        U64FeeFloor {
            fee_num: self.delayed_unstake_fee_bp_cents,
            fee_denom: MAX_BP_CENTS,
        }
    }
}

impl SolValueCalculator for MarinadeStateCalc {
    fn calc_lst_to_sol(&self, msol_amount: u64) -> Result<U64ValueRange, ProgramError> {
        let ratio = self.msol_to_sol_ratio().ok_or(MathError)?;
        let sol_value_of_msol_burned = ratio.apply(msol_amount)?;
        let AmtsAfterFee {
            amt_after_fee: lamports_for_user,
            ..
        } = self.delayed_unstake_fee().apply(sol_value_of_msol_burned)?;
        Ok(U64ValueRange::single(lamports_for_user))
    }

    fn calc_sol_to_lst(&self, lamports_for_user: u64) -> Result<U64ValueRange, ProgramError> {
        let U64ValueRange { min, max } = self
            .delayed_unstake_fee()
            .reverse_from_amt_after_fee(lamports_for_user)?;
        let ratio = self.msol_to_sol_ratio().ok_or(MathError)?;
        let U64ValueRange { min, .. } = ratio.reverse(min)?;
        let U64ValueRange { max, .. } = ratio.reverse(max)?;
        if min > max {
            return Err(MathError.into());
        }
        Ok(U64ValueRange { min, max })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    prop_compose! {
        fn total_cooling_down()
            (delayed_unstake_cooling_down in any::<u64>())
            (
                emergency_cooling_down in 0..=(u64::MAX - delayed_unstake_cooling_down),
                delayed_unstake_cooling_down in Just(delayed_unstake_cooling_down)
            ) -> (u64, u64) {
                (emergency_cooling_down, delayed_unstake_cooling_down)
            }
    }

    prop_compose! {
        fn total_active_cooling_down()
            ((emergency_cooling_down, delayed_unstake_cooling_down) in total_cooling_down())
            (
                total_active_balance in 0..=(u64::MAX - emergency_cooling_down - delayed_unstake_cooling_down),
                emergency_cooling_down in Just(emergency_cooling_down),
                delayed_unstake_cooling_down in Just(delayed_unstake_cooling_down)
            ) -> (u64, u64, u64) {
                (total_active_balance, emergency_cooling_down, delayed_unstake_cooling_down)
            }
    }

    prop_compose! {
        fn total_lamports_under_control()
            ((total_active_balance, emergency_cooling_down, delayed_unstake_cooling_down) in total_active_cooling_down())
            (
                available_reserve_balance in 0..=(u64::MAX - total_active_balance - emergency_cooling_down - delayed_unstake_cooling_down),
                total_active_balance in Just(total_active_balance),
                emergency_cooling_down in Just(emergency_cooling_down),
                delayed_unstake_cooling_down in Just(delayed_unstake_cooling_down),
            ) -> (u64, u64, u64, u64) {
                (available_reserve_balance, total_active_balance, emergency_cooling_down, delayed_unstake_cooling_down)
            }
    }

    prop_compose! {
        fn marinade_calc()
            (
                (
                    available_reserve_balance,
                    total_active_balance,
                    emergency_cooling_down,
                    delayed_unstake_cooling_down,
                ) in total_lamports_under_control(),
                circulating_ticket_balance: u64,
                msol_supply: u64,
                delayed_unstake_fee_bp_cents in 0..=MAX_BP_CENTS
            ) -> MarinadeStateCalc {
                MarinadeStateCalc {
                    paused: false,
                    delayed_unstake_cooling_down,
                    emergency_cooling_down,
                    total_active_balance,
                    available_reserve_balance,
                    circulating_ticket_balance,
                    msol_supply,
                    delayed_unstake_fee_bp_cents,
                }
            }
    }

    prop_compose! {
        fn marinade_state_and_lst_amount()
            (calc in marinade_calc())
            (msol_amount in 0..=calc.msol_supply, calc in Just(calc)) -> (u64, MarinadeStateCalc) {
                (msol_amount, calc)
            }
    }

    proptest! {
        #[test]
        fn lst_sol_round_trip((msol_amount, calc) in marinade_state_and_lst_amount()) {
            let U64ValueRange { min: sol_amt, max: max_sol_amt } = calc.calc_lst_to_sol(msol_amount).unwrap();
            prop_assert_eq!(sol_amt, max_sol_amt);
            let U64ValueRange { min, max } = calc.calc_sol_to_lst(sol_amt).unwrap();

            // TODO: figure out the diff_at_most wide range, is this rly just errors accumulating?

            // round trip from min should not exceed original
            let min_round_trip = calc.calc_lst_to_sol(min).unwrap();
            prop_assert!(sol_amt >= min_round_trip.min, "{sol_amt} {}", min_round_trip.min);
            prop_assert!(sol_amt >= min_round_trip.max, "{sol_amt} {}", min_round_trip.max);
            //test_utils::prop_assert_diff_at_most!(min_round_trip, sol_amt, 1_000);

            // round trip from max should not be smaller than original
            let max_round_trip = calc.calc_lst_to_sol(max).unwrap();
            prop_assert!(sol_amt <= max_round_trip.min, "{sol_amt} {}", max_round_trip.min);
            prop_assert!(sol_amt <= max_round_trip.max, "{sol_amt} {}", max_round_trip.max);
            //test_utils::prop_assert_diff_at_most!(max_round_trip, sol_amt, 1_000);
        }
    }
}
