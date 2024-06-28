use flex_pool_hooks::{AfterInstantiateState, HookCall};
use oracle::{Oracle, ObservationInterval, AccumulatedObservation};
use common::time::Time;
use scrypto::prelude::*;

#[blueprint]
#[types(u16, AccumulatedObservation)]
mod oracle_hook {
    use flex_pool_hooks::AfterSwapState;

    enable_method_auth! {
        roles {
            hook_admin => updatable_by: [OWNER];
        },
        methods {
            get_calls => PUBLIC;
            observations_limit => PUBLIC;
            observation => PUBLIC;
            observation_intervals => PUBLIC;
            observations_stored => PUBLIC;
            last_observation_index => PUBLIC;
            after_instantiate => restrict_to: [hook_admin];
            after_swap => restrict_to: [hook_admin];
        }
    }
    struct OracleHook {
        calls: Vec<HookCall>,

        pool_address: Option<ComponentAddress>,
        x_address: Option<ResourceAddress>,
        y_address: Option<ResourceAddress>,

        oracle: Oracle,

        last_price_sqrt: PreciseDecimal,
    }

    impl OracleHook {
        pub fn instantiate() -> (Global<OracleHook>, Bucket) {
            let hook_badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);

            let hook_global = (Self {
                calls: vec![HookCall::AfterInstantiate, HookCall::AfterSwap],

                pool_address: None,
                x_address: None,
                y_address: None,

                oracle: Oracle::new(),

                last_price_sqrt: pdec!(0),
            })
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .roles(roles!(
                hook_admin => rule!(require(hook_badge.resource_address()));
            ))
            .globalize();

            (hook_global, hook_badge.into())
        }

        pub fn get_calls(&mut self) -> Vec<HookCall> {
            self.calls.clone()
        }

        pub fn after_instantiate(
            &mut self,
            state: AfterInstantiateState,
        ) -> (AfterInstantiateState,) {
            debug!(
                "[ORACLE HOOK] After instantiate, observation count: {:?}",
                self.oracle.last_observation_index()
            );

            debug!(
                "[ORACLE HOOK] After instantiate, clock_time: {} unix-minutes, i.e. {} unix-seconds",
                Clock::current_time_rounded_to_minutes().seconds_since_unix_epoch,
                Clock::time_in_minutes()
            );

            self.pool_address = Some(state.pool_address);
            self.x_address = Some(state.x_address);
            self.y_address = Some(state.y_address);

            (state,)
        }

        pub fn after_swap(
            &mut self,
            swap_state: AfterSwapState,
            input_bucket: Bucket,
        ) -> (AfterSwapState, Bucket) {
            debug!("STORED: {}", self.oracle.observations_stored());
            debug!("TIME CURRENT: {}", Clock::time_in_minutes());

            self.oracle.observe(swap_state.price_sqrt);

            (swap_state, input_bucket)
        }

        /// Returns an AccumulatedObservation for a given timestamp. A few scenarios can happen:
        /// - If an observation exists for the provided timestamp, it is returned
        /// - If no observation matches the timestamp, but the timestamp is within the range captured by the oracle,
        /// an interpolated observation is generated from the two closest ones
        /// - If the timestamp is more recent than the lastest stored timestamp, but lesser or equal than the current timestamp,
        /// a new observation is generated
        /// - Other timestamps will yield cause a panic.
        pub fn observation(&self, seconds: u64) -> AccumulatedObservation {
            self.oracle.observation(seconds)
        }

        /// For a given timestamp pair tuple, calculates the average price_sqrt.
        /// Receives a vector of such pairs, and returns ObservationInterval's.
        pub fn observation_intervals(
            &self,
            intervals: Vec<(u64, u64)>, // In Unix seconds
        ) -> Vec<ObservationInterval> {
            self.oracle.observation_intervals(intervals)
        }

        pub fn observations_limit(&self) -> u16 {
            self.oracle.observations_limit()
        }

        pub fn observations_stored(&self) -> u16 {
            self.oracle.observations_stored()
        }

        // For testing
        pub fn last_observation_index(&self) -> Option<u16> {
            self.oracle.last_observation_index()
        }
    }
}
