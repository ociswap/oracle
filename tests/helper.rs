use common::pools::SwapType;
use lazy_static::lazy_static;
use oracle::{AccumulatedObservation, ObservationInterval};
use pretty_assertions::assert_eq;
use radix_engine::system::system_modules::execution_trace::ResourceSpecifier::Amount;
use scrypto::{blueprints::consensus_manager::TimePrecision, prelude::*};
use scrypto_math::*;
use scrypto_testenv::*;
use serde_json;
use std::fs::File;
use std::io::Read;
use std::mem;
use test_oracle::test_oracle::{AfterInstantiateState, AfterSwapState, HookCall};

lazy_static! {
    pub static ref TEST_DATAPOINTS: Vec<PreciseDecimal> = {
        let mut file = File::open("./tests/specification/oracle_test_datapoints.json").unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let vectors: HashMap<String, Vec<String>> = serde_json::from_str(&contents).unwrap();

        let retrieve = |vector: &Vec<String>| {
            vector
                .into_iter()
                .map(|i| PreciseDecimal::try_from(i.as_str()).unwrap())
                .collect()
        };

        let prices_sqrt: Vec<PreciseDecimal> = retrieve(&vectors["price_sqrt"]);

        prices_sqrt
    };
    static ref DUMMY_POOL: ComponentAddress = ComponentAddress::try_from_hex(
        "c0cdeecfc24b8c7132d4636883099c5fe0764dd015e93b196030c3ce2dc2"
    )
    .unwrap();
}

pub trait Dummy {
    fn dummy(
        pool_address: ComponentAddress,
        x_address: ResourceAddress,
        y_address: ResourceAddress,
    ) -> Self;
    fn empty(
        pool_address: ComponentAddress,
        x_address: ResourceAddress,
        y_address: ResourceAddress,
    ) -> Self;
    fn from_test_datapoints(
        number_of_observations: usize,
        pool_address: ComponentAddress,
        x_address: ResourceAddress,
        y_address: ResourceAddress,
    ) -> Vec<Self>
    where
        Self: Sized;
}

impl Dummy for AfterInstantiateState {
    fn empty(
        pool_address: ComponentAddress,
        x_address: ResourceAddress,
        y_address: ResourceAddress,
    ) -> Self {
        AfterInstantiateState {
            pool_address: pool_address,
            price_sqrt: None,
            x_address,
            y_address,
            input_fee_rate: dec!(0),
            flash_loan_fee_rate: dec!(0),
        }
    }
    fn dummy(
        pool_address: ComponentAddress,
        x_address: ResourceAddress,
        y_address: ResourceAddress,
    ) -> Self {
        AfterInstantiateState {
            pool_address: pool_address,
            price_sqrt: Some(pdec!("23.36527")),
            x_address,
            y_address,
            input_fee_rate: dec!(0),
            flash_loan_fee_rate: dec!(0),
        }
    }

    fn from_test_datapoints(
        _number_of_observations: usize,
        _pool_address: ComponentAddress,
        _x_address: ResourceAddress,
        _y_address: ResourceAddress,
    ) -> Vec<Self>
    where
        Self: Sized,
    {
        unimplemented!();
    }
}

impl Dummy for AfterSwapState {
    // fn dummy() -> Self {
    //     BeforeSwapState {
    //         pool_address: *DUMMY_POOL,
    //         price_sqrt: pdec!("1.1"),
    //         active_liquidity: pdec!(107),
    //         swap_type: SwapType::BuyX,
    //         input_fee_rate: dec!("0.0145"),
    //         fee_protocol_share: dec!("0.0193"),
    //     }
    // }

    // fn empty() -> Self {
    //     BeforeSwapState {
    //         pool_address: *DUMMY_POOL,
    //         price_sqrt: pdec!("1"),
    //         active_liquidity: pdec!(100),
    //         swap_type: SwapType::BuyX,
    //         input_fee_rate: dec!(0),
    //         fee_protocol_share: dec!(0),
    //     }
    // }

    fn dummy(pool_address: ComponentAddress, _: ResourceAddress, _: ResourceAddress) -> Self {
        AfterSwapState {
            pool_address: pool_address,
            price_sqrt: pdec!("1.1"),
            active_liquidity: pdec!(107),
            swap_type: SwapType::BuyX,
            input_fee_rate: dec!("0.0145"),
            fee_protocol_share: dec!("0.0193"),
            input_address: XRD,            // Assuming default or placeholder
            input_amount: dec!("0"),       // Assuming default value
            output_address: XRD,           // Assuming default or placeholder
            output_amount: dec!("0"),      // Assuming default value
            input_fee_lp: dec!("0"),       // Assuming default value
            input_fee_protocol: dec!("0"), // Assuming default value
        }
    }

    fn empty(pool_address: ComponentAddress, _: ResourceAddress, _: ResourceAddress) -> Self {
        AfterSwapState {
            pool_address: pool_address,
            price_sqrt: pdec!("1"),
            active_liquidity: pdec!(100),
            swap_type: SwapType::BuyX,
            input_fee_rate: dec!(0),
            fee_protocol_share: dec!(0),
            input_address: XRD,            // Assuming default or placeholder
            input_amount: dec!("0"),       // Assuming default value
            output_address: XRD,           // Assuming default or placeholder
            output_amount: dec!("0"),      // Assuming default value
            input_fee_lp: dec!("0"),       // Assuming default value
            input_fee_protocol: dec!("0"), // Assuming default value
        }
    }

    fn from_test_datapoints(
        number_of_states: usize,
        pool_address: ComponentAddress,
        x_address: ResourceAddress,
        y_address: ResourceAddress,
    ) -> Vec<AfterSwapState> {
        assert!(number_of_states <= 20);
        let ref prices_sqrt = *TEST_DATAPOINTS;
        let mut states: Vec<AfterSwapState> = Vec::with_capacity(number_of_states);

        for i in 0..number_of_states {
            let mut new_state = AfterSwapState::empty(pool_address, x_address, y_address);

            new_state.price_sqrt = prices_sqrt[i];

            states.push(new_state);
        }

        states
    }
}

// impl Dummy for AfterSwapState {
//     fn dummy() -> Self {
//         AfterSwapState {
//             pool_address: *DUMMY_POOL,
//             price_sqrt: pdec!("1.53"),
//             active_liquidity: pdec!(111),
//             swap_type: SwapType::BuyX,
//             input_fee_rate: dec!("0.01365"),
//             fee_protocol_share: dec!("0.012176"),
//             input_address: XRD, // TODO fix
//             input_amount: dec!(17),
//             output_address: XRD, // TODO fix
//             output_amount: dec!(11),
//             input_fee_lp: dec!("0.1123"),
//             input_fee_protocol: dec!("0.012348"),
//         }
//     }

//     fn empty() -> Self {
//         AfterSwapState {
//             pool_address: *DUMMY_POOL,
//             price_sqrt: pdec!(1),
//             active_liquidity: pdec!(100),
//             swap_type: SwapType::BuyX,
//             input_fee_rate: dec!(0),
//             fee_protocol_share: dec!(0),
//             input_address: XRD, // TODO fix
//             input_amount: dec!(0),
//             output_address: XRD, // TODO fix
//             output_amount: dec!(0),
//             input_fee_lp: dec!(0),
//             input_fee_protocol: dec!(0),
//         }
//     }

//     fn from_test_datapoints(number_of_observations: usize) -> Vec<Self> {
//         unimplemented!();
//     }
// }

impl TestHelperExecution for OracleTestHelper {
    fn env(&mut self) -> &mut TestEnvironment {
        &mut self.env
    }
}

// struct WrappedRange(u64, u64, u64, u64);
// timestamp_oldest
// timestamp_oldest_index
// timestamp_step
// count

// impl Sized for WrappedRange {

// }

// pub struct WrappedRange {
//     timestamp_oldest: u64,
//     timestamp_oldest_index: u64,
//     timestamp_step: u64,
//     count: u64,
//     vector: Vec<u64>,
//     vector_wrapped: Vec<u64>,
// }

// impl WrappedRange {
//     pub fn new(
//         timestamp_oldest: u64,
//         timestamp_oldest_index: u64,
//         timestamp_step: u64,
//         count: u64
//     ) -> WrappedRange {
//         assert!(timestamp_oldest_index < count - 1); // First observation slots can't be empty

//         let mut vector: Vec<u64> = (0..count)
//             .map(|i| timestamp_oldest + i * timestamp_step)
//             .collect();

//         let mut vector_wrapped = vector.clone();
//         vector_wrapped.rotate_right(timestamp_oldest_index as usize);

//         WrappedRange {
//             timestamp_oldest,
//             timestamp_oldest_index,
//             timestamp_step,
//             count,
//             vector,
//             vector_wrapped,
//         }
//     }

//     fn timestamp_oldest(&self) -> u64 {
//         self.timestamp_oldest
//     }

//     fn timestamp_oldest_index(&self) -> u64 {
//         self.timestamp_oldest_index
//     }

//     fn timestamp_step(&self) -> u64 {
//         self.timestamp_step
//     }

//     fn count(&self) -> u64 {
//         self.count
//     }

//     fn vector(&self) -> &Vec<u64> {
//         &self.vector
//     }

//     fn vector_wrapped(&self) -> &Vec<u64> {
//         &self.vector_wrapped
//     }
// }

// (timestamp_start, timestamp_step, timestamp_number, wrap_number)
// (start, step, number, wrap_number)
// start: initial timestamp
// step: gap between timestamps
// number: number of timestamps
// wrap_number: number of observations what would fit before a wrap occurs

pub trait Approximate {
    fn approximate(&self) -> Self;
}

impl Approximate for AccumulatedObservation {
    fn approximate(&self) -> Self {
        let decimal_places = 15;
        Self {
            timestamp: self.timestamp,
            price_sqrt_log_acc: self
                .price_sqrt_log_acc
                .checked_round(decimal_places, RoundingMode::ToNegativeInfinity)
                .unwrap(),
        }
    }
}

impl Approximate for ObservationInterval {
    fn approximate(&self) -> Self {
        let decimal_places = 15;
        Self {
            start: self.start,
            end: self.end,
            price_sqrt: self
                .price_sqrt
                .checked_round(decimal_places, RoundingMode::ToNegativeInfinity)
                .unwrap(),
        }
    }
}

pub struct OracleTestHelper {
    pub env: TestEnvironment,
    pub oracle_address: Option<ComponentAddress>,
    pub pool_address: Option<ComponentAddress>,
    pub hook_badge_address: Option<ResourceAddress>,
}

impl OracleTestHelper {
    pub fn after_swap_state_dummy(&self) -> AfterSwapState {
        AfterSwapState::dummy(
            self.pool_address.unwrap(),
            self.x_address(),
            self.y_address(),
        )
    }

    // pub fn before_swap_state_empty(&self) -> BeforeSwapState {
    //     BeforeSwapState::empty(self.pool_address.unwrap(), self.x_address(), self.y_address())
    // }

    // pub fn before_swap_state_from_test_datapoints(
    //     &self,
    //     number_of_observations: usize
    // ) -> Vec<BeforeSwapState> {
    //     BeforeSwapState::from_test_datapoints(
    //         number_of_observations,
    //         self.pool_address.unwrap(),
    //         self.x_address(),
    //         self.y_address()
    //     )
    // }

    pub fn after_instantiate_state_dummy(&self) -> AfterInstantiateState {
        AfterInstantiateState::dummy(
            self.pool_address.unwrap(),
            self.x_address(),
            self.y_address(),
        )
    }

    pub fn after_instantiate_state_empty(&self) -> AfterInstantiateState {
        AfterInstantiateState::empty(
            self.pool_address.unwrap(),
            self.x_address(),
            self.y_address(),
        )
    }

    pub fn new() -> Self {
        let packages: HashMap<&str, &str> = vec![("oracle", "test_oracle")].into_iter().collect();
        let env = TestEnvironment::new(packages);

        let fake_pool_address = env.account;

        let mut helper = Self {
            env,
            oracle_address: None,
            pool_address: Some(fake_pool_address),
            hook_badge_address: None,
        };

        helper.advance_timestamp_by_seconds(60);

        helper
    }

    pub fn new_with_observations_minutes(timestamps: &Vec<u64>) -> Self {
        let mut helper = OracleTestHelper::new();
        helper.instantiate_instant();

        helper.add_observations_in_minutes(timestamps);
        helper.execute_expect_success(true);

        helper
    }

    pub fn new_with_swap_state_seconds(seconds: &Vec<u64>) -> Self {
        let mut helper = OracleTestHelper::new();
        helper.instantiate_instant();

        helper.add_swap_state_seconds(seconds);

        helper
    }

    pub fn add_observations_in_minutes(&mut self, minutes: &Vec<u64>) {
        let seconds: Vec<u64> = minutes.iter().map(|value| value * 60).collect();
        self.add_observations_in_seconds(&seconds);
    }

    pub fn add_observations_in_seconds(&mut self, timestamps: &Vec<u64>) {
        if timestamps.is_empty() {
            return;
        }
        let after_swap_states = AfterSwapState::from_test_datapoints(
            timestamps.len() + 1,
            self.pool_address.unwrap(),
            self.x_address(),
            self.y_address(),
        );
        let mut timestamps = timestamps.clone();
        timestamps.insert(0, timestamps[0] - 60);
        println!("Adding observations for timestamps:\n{:#?}", timestamps);
        let zipped_timestamps_states = timestamps.iter().zip(after_swap_states.clone());

        for (timestamp, state) in zipped_timestamps_states {
            self.jump_to_timestamp_seconds(*timestamp);
            self.load_hook_auth();
            self.after_swap(state, self.y_address(), dec!("1.2344"));
            self.execute_expect_success(false);
        }
    }

    pub fn add_swap_state_seconds(&mut self, timestamps: &Vec<u64>) {
        let after_swap_states = AfterSwapState::from_test_datapoints(
            timestamps.len(),
            self.pool_address.unwrap(),
            self.x_address(),
            self.y_address(),
        );
        let zipped_timestamps_states = timestamps.iter().zip(after_swap_states.clone());

        for (timestamp, state) in zipped_timestamps_states {
            self.jump_to_timestamp_seconds(*timestamp);
            self.load_hook_auth();
            self.after_swap(state, self.y_address(), dec!("1.2344"));
            self.execute_expect_success(true);
        }
    }

    pub fn instantiate(&mut self) -> &mut OracleTestHelper {
        let manifest_builder = mem::take(&mut self.env.manifest_builder);
        self.env.manifest_builder = manifest_builder.call_function(
            self.env.package_address("oracle"),
            "TestOracle",
            "instantiate",
            manifest_args!(),
        );
        self.env.new_instruction("instantiate", 1, 0);
        self
    }

    pub fn instantiate_instant(&mut self) -> &mut OracleTestHelper {
        self.instantiate();
        let receipt = self.execute_expect_success(false);
        let (oracle_address, _): (ComponentAddress, Bucket) = receipt.outputs("instantiate")[0];
        let hook_badge_address = receipt
            .execution_receipt
            .expect_commit_success()
            .new_resource_addresses()[0];
        self.oracle_address = Some(oracle_address);
        self.hook_badge_address = Some(hook_badge_address);
        self
    }

    pub fn load_hook_auth(&mut self) -> &mut OracleTestHelper {
        let manifest_builder = mem::take(&mut self.env.manifest_builder);

        self.env().manifest_builder = manifest_builder.create_proof_from_account_of_amount(
            self.env.account,
            self.hook_badge_address.unwrap(),
            dec!(1),
        );
        self.env.new_instruction("load_hook_auth", 1, 0);
        self
    }

    pub fn after_instantiate(
        &mut self,
        after_instantiate_state: AfterInstantiateState,
    ) -> &mut OracleTestHelper {
        let manifest_builder = mem::take(&mut self.env.manifest_builder);

        self.env().manifest_builder = manifest_builder.call_method(
            self.oracle_address.unwrap(),
            "after_instantiate",
            manifest_args!(after_instantiate_state),
        );
        self.env.new_instruction("after_instantiate", 1, 0);
        self
    }

    pub fn after_instantiate_default(&mut self) -> &mut OracleTestHelper {
        let after_instantiate_state = AfterInstantiateState::empty(
            self.pool_address.unwrap(),
            self.x_address(),
            self.y_address(),
        );
        self.after_instantiate(after_instantiate_state);
        self
    }

    pub fn get_calls(&mut self) -> &mut OracleTestHelper {
        let manifest_builder = mem::take(&mut self.env.manifest_builder);

        self.env().manifest_builder = manifest_builder.call_method(
            self.oracle_address.unwrap(),
            "get_calls",
            manifest_args!(),
        );
        self.env.new_instruction("get_calls", 1, 0);
        self
    }

    pub fn after_swap(
        &mut self,
        after_swap_state: AfterSwapState,
        input_address: ResourceAddress,
        input_amount: Decimal,
    ) -> &mut OracleTestHelper {
        let manifest_builder = mem::take(&mut self.env.manifest_builder);

        self.env.manifest_builder = manifest_builder
            .withdraw_from_account(self.env.account, input_address, input_amount)
            .take_from_worktop(input_address, input_amount, self.name("input_bucket"))
            .with_name_lookup(|builder, lookup| {
                let input_bucket = lookup.bucket(self.name("input_bucket"));
                builder.call_method(
                    self.oracle_address.unwrap(),
                    "after_swap",
                    manifest_args!(after_swap_state, input_bucket),
                )
            });

        self.env.new_instruction("after_swap", 3, 2);
        self
    }

    pub fn after_swap_default(&mut self) -> &mut OracleTestHelper {
        let input_address = self.y_address();
        let input_amount = dec!("7.463543");

        let manifest_builder = mem::take(&mut self.env.manifest_builder);

        self.env.manifest_builder = manifest_builder
            .withdraw_from_account(self.env.account, input_address, input_amount)
            .take_from_worktop(input_address, input_amount, self.name("input_bucket"))
            .with_name_lookup(|builder, lookup| {
                let input_bucket = lookup.bucket(self.name("input_bucket"));
                builder.call_method(
                    self.oracle_address.unwrap(),
                    "after_swap",
                    manifest_args!(self.after_swap_state_dummy(), input_bucket),
                )
            });

        self.env.new_instruction("after_swap", 3, 2);
        self
    }

    /*     pub fn after_swap(
        &mut self,
        after_swap_state: AfterSwapState,
        input_address: ResourceAddress,
        input_amount: Decimal
    ) -> &mut OracleTestHelper {
        let manifest_builder = mem::take(&mut self.env.manifest_builder);

        self.env.manifest_builder = manifest_builder
            .withdraw_from_account(self.env.account, input_address, input_amount)
            .take_from_worktop(input_address, input_amount, self.name("input_bucket"))
            .with_name_lookup(|builder, lookup| {
                let input_bucket = lookup.bucket(self.name("input_bucket"));
                builder.call_method(
                    self.oracle_address.unwrap(),
                    "after_swap",
                    manifest_args!(after_swap_state, input_bucket)
                )
            });

        self.env.new_instruction("after_swap", 3, 2);
        self
    } */

    pub fn observation(&mut self, seconds: u64) -> &mut OracleTestHelper {
        let manifest_builder = mem::take(&mut self.env.manifest_builder);

        self.env().manifest_builder = manifest_builder.call_method(
            self.oracle_address.unwrap(),
            "observation",
            manifest_args!(seconds),
        );
        self.env.new_instruction("observation", 1, 0);
        self
    }

    pub fn observation_batch(&mut self, seconds: Vec<u64>) -> &mut OracleTestHelper {
        for timestamp in seconds {
            self.observation(timestamp);
        }
        self
    }

    pub fn observation_in_minutes(&mut self, minutes: u64) -> &mut OracleTestHelper {
        self.observation(minutes * 60);
        self
    }

    pub fn observation_intervals(&mut self, intervals: Vec<(u64, u64)>) -> &mut OracleTestHelper {
        let manifest_builder = mem::take(&mut self.env.manifest_builder);

        self.env().manifest_builder = manifest_builder.call_method(
            self.oracle_address.unwrap(),
            "observation_intervals",
            manifest_args!(intervals),
        );
        self.env.new_instruction("observation_intervals", 1, 0);
        self
    }

    pub fn increase_capacity(&mut self, new_limit: u16) -> &mut OracleTestHelper {
        let manifest_builder = mem::take(&mut self.env.manifest_builder);

        self.env().manifest_builder = manifest_builder.call_method(
            self.oracle_address.unwrap(),
            "increase_capacity",
            manifest_args!(new_limit),
        );
        self.env.new_instruction("increase_capacity", 1, 0);
        self
    }

    pub fn observations_limit(&mut self) -> &mut OracleTestHelper {
        let manifest_builder = mem::take(&mut self.env.manifest_builder);

        self.env().manifest_builder = manifest_builder.call_method(
            self.oracle_address.unwrap(),
            "observations_limit",
            manifest_args!(),
        );
        self.env.new_instruction("observations_limit", 1, 0);
        self
    }

    pub fn oldest_observation_timestamp(&mut self) -> &mut OracleTestHelper {
        let manifest_builder = mem::take(&mut self.env.manifest_builder);

        self.env().manifest_builder = manifest_builder.call_method(
            self.oracle_address.unwrap(),
            "oldest_observation_timestamp",
            manifest_args!(),
        );
        self.env
            .new_instruction("oldest_observation_timestamp", 1, 0);
        self
    }

    pub fn last_observation_index(&mut self) -> &mut OracleTestHelper {
        let manifest_builder = mem::take(&mut self.env.manifest_builder);

        self.env().manifest_builder = manifest_builder.call_method(
            self.oracle_address.unwrap(),
            "last_observation_index",
            manifest_args!(),
        );
        self.env.new_instruction("last_observation_index", 1, 0);
        self
    }

    pub fn observations_stored(&mut self) -> &mut OracleTestHelper {
        let manifest_builder = mem::take(&mut self.env.manifest_builder);

        self.env().manifest_builder = manifest_builder.call_method(
            self.oracle_address.unwrap(),
            "observations_stored",
            manifest_args!(),
        );
        self.env.new_instruction("observations_stored", 1, 0);
        self
    }

    pub fn assert_outputs_equal_inputs(&mut self, hook_call: HookCall) -> &mut OracleTestHelper {
        self.instantiate_instant();
        self.load_hook_auth();
        match hook_call {
            HookCall::AfterInstantiate => {
                let after_instantiate_state = self.after_instantiate_state_dummy();

                self.after_instantiate(after_instantiate_state.clone());

                let receipt = self.execute_expect_success(false);
                let outputs: Vec<(AfterInstantiateState,)> = receipt.outputs("after_instantiate");

                assert_eq!(outputs, vec![(after_instantiate_state,)]);
            }
            HookCall::AfterSwap => {
                self.after_instantiate_default();

                let after_swap_state = self.after_swap_state_dummy();
                let input_address = self.y_address();
                let input_amount = dec!("23.36527");
                self.after_swap(after_swap_state.clone(), input_address, input_amount);

                let receipt = self.execute_expect_success(false);
                let outputs: Vec<(AfterSwapState, Bucket)> = receipt.outputs("after_swap");

                let output_buckets = receipt.output_buckets("after_swap");

                assert_eq!(outputs.len(), 1);
                assert_eq!(outputs[0].0, after_swap_state);

                assert_eq!(
                    output_buckets,
                    vec![vec![Amount(input_address, input_amount)]]
                );
            }
            /*             HookCall::AfterSwap => {
                self.after_instantiate_default();

                let after_swap_state = AfterSwapState::dummy();
                let output_address = self.x_address();
                let output_amount = dec!("23.36527");
                self.after_swap(after_swap_state.clone(), output_address, output_amount);

                let receipt = self.execute_expect_success(false);
                let outputs: Vec<(AfterSwapState, Bucket)> = receipt.outputs("after_swap");

                let output_buckets = receipt.output_buckets("after_swap");

                assert_eq!(outputs.len(), 1);
                assert_eq!(outputs[0].0, after_swap_state);

                assert_eq!(output_buckets, vec![vec![Amount(output_address, output_amount)]]);
            } */
            _ => {
                panic!("HookCall is not allowed");
            }
        }
        self
    }

    pub fn jump_to_timestamp_seconds(&mut self, seconds: u64) {
        let current_time = self
            .env
            .test_runner
            .get_current_time(TimePrecision::Minute)
            .seconds_since_unix_epoch as u64;
        if current_time == seconds {
            return;
        }

        let current_round = self
            .env
            .test_runner
            .get_consensus_manager_state()
            .round
            .number();
        self.env()
            .test_runner
            .advance_to_round_at_timestamp(Round::of(current_round + 1), (seconds * 1000) as i64);
    }

    pub fn advance_timestamp_by_seconds(&mut self, seconds: u64) {
        let current_time = self
            .env()
            .test_runner
            .get_current_time(TimePrecision::Minute)
            .seconds_since_unix_epoch as u64;
        self.jump_to_timestamp_seconds(current_time + seconds)
    }

    pub fn jump_to_timestamp_minutes(&mut self, minutes: u64) {
        self.jump_to_timestamp_seconds(minutes * 60);
    }

    pub fn advance_timestamp_by_minutes(&mut self, minutes: u64) {
        self.advance_timestamp_by_seconds(minutes * 60);
    }

    pub fn x_address(&self) -> ResourceAddress {
        self.env.x_address
    }

    pub fn y_address(&self) -> ResourceAddress {
        self.env.y_address
    }
}

// This is only for deserialization

fn deserialize_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    s.parse::<Decimal>().map_err(serde::de::Error::custom)
}

fn deserialize_precise_decimal<'de, D>(deserializer: D) -> Result<PreciseDecimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    s.parse::<PreciseDecimal>()
        .map_err(serde::de::Error::custom)
}

#[derive(serde::Deserialize)]
struct DeserializedAccumulatedObservation {
    timestamp: u64,
    #[serde(deserialize_with = "deserialize_decimal")]
    acc_price_sqrt_log: Decimal,
}

impl From<DeserializedAccumulatedObservation> for AccumulatedObservation {
    fn from(obs: DeserializedAccumulatedObservation) -> Self {
        Self {
            timestamp: obs.timestamp,
            price_sqrt_log_acc: obs.acc_price_sqrt_log,
        }
    }
}

#[derive(serde::Deserialize)]
struct DeserializedObservationInterval {
    start: u64,
    end: u64,
    #[serde(deserialize_with = "deserialize_decimal")]
    price_sqrt: Decimal,
}

impl From<DeserializedObservationInterval> for ObservationInterval {
    fn from(obs: DeserializedObservationInterval) -> Self {
        Self {
            start: obs.start,
            end: obs.end,
            price_sqrt: obs.price_sqrt,
        }
    }
}

pub fn import_accumulated_observations(path: &str) -> Vec<AccumulatedObservation> {
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let deserialized_accumulated_observations: Vec<DeserializedAccumulatedObservation> =
        serde_json::from_str(&contents).expect("Error during deserialization");

    // Convert deserialized observations to AccumulatedObservation
    let accumulated_observations: Vec<AccumulatedObservation> =
        deserialized_accumulated_observations
            .into_iter()
            .map(AccumulatedObservation::from)
            .collect();

    accumulated_observations
}

pub fn import_observation_intervals(path: &str) -> Vec<ObservationInterval> {
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let deserialized_observation_intervals: Vec<DeserializedObservationInterval> =
        serde_json::from_str(&contents).expect("Error during deserialization");

    let observation_intervals: Vec<ObservationInterval> = deserialized_observation_intervals
        .into_iter()
        .map(ObservationInterval::from)
        .collect();

    observation_intervals
}

pub fn assert_binary_search_result(timestamps: &Vec<u64>, expected: AccumulatedObservation) {
    let target_timestamp = expected.timestamp;

    let mut helper = OracleTestHelper::new_with_observations_minutes(&timestamps);
    let receipt = helper
        .observation_in_minutes(target_timestamp)
        .execute_expect_success(true);
    let output: Vec<AccumulatedObservation> = receipt.outputs("observation");

    assert_vecs_similar(output, vec![expected]);
}

pub fn assert_observations_batch(seconds: &Vec<u64>, expected: Vec<AccumulatedObservation>) {
    let mut helper = OracleTestHelper::new_with_swap_state_seconds(&seconds);

    for observation in expected.iter() {
        helper.observation(observation.timestamp);
    }
    helper.observations_stored();

    let receipt = helper.execute_expect_success(false);
    let output: Vec<AccumulatedObservation> = receipt.outputs("observation");
    let observations_stored: Vec<u16> = receipt.outputs("observations_stored");

    assert_eq!(output, expected.clone());
    assert_eq!(observations_stored, vec![expected.len() as u16]);
}

pub fn assert_vecs_similar<T: Approximate + PartialEq + Debug>(
    mut outputs: Vec<T>,
    mut expected: Vec<T>,
) {
    for obj in &mut outputs {
        *obj = obj.approximate();
    }

    for obj in &mut expected {
        *obj = obj.approximate();
    }

    assert_eq!(outputs, expected);
}

pub fn log(value: PreciseDecimal) -> Decimal {
    value
        .ln()
        .unwrap()
        .checked_truncate(RoundingMode::ToNegativeInfinity)
        .unwrap()
}

fn accumulated_log(
    acc_log: Decimal,
    finalized: PreciseDecimal,
    leaked_value: PreciseDecimal,
    minutes_since_last: u64,
) -> Decimal {
    // TODO confirm math. DO we want to add log(finalized) + log(leaked_value), or log(finalized + leaked_value)

    let leaked_value_log = log(leaked_value);
    let finalized_log = log(finalized);
    let minutes_since_last = if minutes_since_last >= 1 {
        minutes_since_last
    } else {
        1
    };

    // println!("Leaked*minutes_without_transactions{}", leaked_value_log * (minutes_since_last - 1));
    acc_log + finalized_log + leaked_value_log * (minutes_since_last - 1)
}

pub fn weighted_average(values: Vec<PreciseDecimal>, weights: Vec<u64>) -> PreciseDecimal {
    let mut weighted_sum = pdec!(0);
    for i in 0..weights.len() {
        weighted_sum += values[i] * weights[i];
    }

    let normalization: u64 = weights.iter().sum();

    weighted_sum / normalization
}

fn find_indexes_of_value<T: PartialEq>(values: &[T], target: T) -> Vec<usize> {
    values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| if *value == target { Some(index) } else { None })
        .collect()
}

fn get_elements_by_indexes<T: Clone>(values: &[T], indexes: &[usize]) -> Vec<T> {
    indexes
        .iter()
        .filter_map(|&index| values.get(index))
        .cloned()
        .collect()
}

fn consecutive_differences(numbers: &[u64]) -> Vec<u64> {
    numbers
        .windows(2)
        .map(|window| window[1] - window[0])
        .collect()
}

pub fn generate_oracle_data(
    seconds: &Vec<u64>,
) -> (
    Vec<PreciseDecimal>,
    Vec<AccumulatedObservation>,
    PreciseDecimal,
) {
    assert_ne!(seconds[0], 0);
    let values = TEST_DATAPOINTS.clone();

    let timestamps_minutes: Vec<u64> = seconds.iter().map(|v| v / 60).collect();
    let unique_minutes: Vec<u64> = timestamps_minutes
        .iter()
        .copied()
        .collect::<BTreeSet<u64>>()
        .into_iter()
        .collect();

    let mut averages: Vec<PreciseDecimal> = vec![];

    let mut last_values: Vec<PreciseDecimal> = vec![];
    for (i, &minute) in unique_minutes.iter().enumerate() {
        let same_minute_indexes = find_indexes_of_value(&timestamps_minutes, minute);

        let same_minute_seconds = get_elements_by_indexes(&seconds, &same_minute_indexes);
        let same_minute_values = get_elements_by_indexes(&values, &same_minute_indexes);

        let consecutive_differences = consecutive_differences(&same_minute_seconds);

        let mut same_minute_sum = pdec!(0);
        for i in 0..consecutive_differences.len() {
            same_minute_sum += same_minute_values[i] * consecutive_differences[i];
        }

        // Add last value
        same_minute_sum +=
            (60 - (same_minute_seconds.last().unwrap() % 60)) * *same_minute_values.last().unwrap();

        // Add value from previous minute that leaked into current one
        let leaked_value = last_values.last().cloned().unwrap_or(pdec!(0));
        same_minute_sum += leaked_value * (same_minute_seconds[0] % 60);

        let total_duration: u64 = if i != 0 {
            60
        } else {
            60 - (same_minute_seconds.first().unwrap() % 60)
        };

        last_values.push(same_minute_values.last().unwrap().clone());

        let same_minute_avg = same_minute_sum / total_duration;
        averages.push(same_minute_avg);
    }

    let mut accumulated_observations: Vec<AccumulatedObservation> = vec![];

    // Calculate accumulated observations
    if unique_minutes.len() < 2 {
        // No observations if less than 2 unique minutes
        return (averages, vec![], last_values.last().unwrap().clone());
    }

    // Push first observation
    let time_since_beginning = unique_minutes[1] - unique_minutes[0];
    accumulated_observations.push(AccumulatedObservation {
        timestamp: unique_minutes[1] * 60,
        price_sqrt_log_acc: accumulated_log(
            dec!(0),
            averages[0],
            last_values[0],
            time_since_beginning,
        ),
    });
    println!(
        "OBSERVATION {}:\nAcc_log: {}\nfinalized: {}\nleaked_value: {}\nminutes_since_last: {}\nresult: {}",
        0,
        dec!(0),
        averages[0],
        last_values[0],
        time_since_beginning,
        accumulated_observations[0].price_sqrt_log_acc
    );

    for i in 2..unique_minutes.len() {
        let last_observation = accumulated_observations.last().unwrap();

        let time_since_last_obs = unique_minutes[i] - unique_minutes[i - 1];
        let price_sqrt_log_acc = accumulated_log(
            last_observation.price_sqrt_log_acc,
            averages[i - 1],
            last_values[i - 1],
            time_since_last_obs,
        );

        let new_observation = AccumulatedObservation {
            timestamp: unique_minutes[i] * 60,
            price_sqrt_log_acc,
        };

        println!(
            "OBSERVATION {}:\nAcc_log: {}\nfinalized: {}\nleaked_value: {}\nminutes_since_last: {}\nresult: {}",
            i - 1,
            last_observation.price_sqrt_log_acc,
            averages[i - 1],
            last_values[i - 1],
            time_since_last_obs,
            price_sqrt_log_acc
        );

        accumulated_observations.push(new_observation);
    }
    (
        averages,
        accumulated_observations,
        last_values.last().unwrap().clone(),
    )
}

pub fn get_averages_from_swap_seconds(seconds: &Vec<u64>) -> Vec<PreciseDecimal> {
    generate_oracle_data(&seconds).0
}

pub fn get_observations_from_swap_seconds(seconds: &Vec<u64>) -> Vec<AccumulatedObservation> {
    generate_oracle_data(&seconds).1
}

pub fn get_observations_from_observation_minutes(
    minutes: &Vec<u64>,
) -> Vec<AccumulatedObservation> {
    let seconds = convert_observation_minutes_to_swap_state_seconds(&minutes);
    generate_oracle_data(&seconds).1
}

pub fn find_neighbors(
    observations: &Vec<AccumulatedObservation>,
    target_seconds: u64,
) -> (AccumulatedObservation, AccumulatedObservation) {
    let mut left = None;
    let mut right = None;

    for obs in observations {
        let timestamp = obs.timestamp;
        if timestamp <= target_seconds {
            left = Some(obs.clone());
        }
        if timestamp >= target_seconds && right.is_none() {
            right = Some(obs.clone());
            // If the right value is found, we don't need to continue the loop
            break;
        }
    }

    if right.is_none() {
        right = left.clone();
    }

    // println!(
    //     "\n[FIND NEIGHBORS]\nObservations: {:#?}\ntarget_seconds: {}\nleft: {:#?}\nright: {:#?}",
    //     observations
    //         .iter()
    //         .map(|obs| obs.timestamp)
    //         .collect::<Vec<u64>>(),
    //     target_seconds,
    //     left,
    //     right
    // );
    (left.unwrap(), right.unwrap())
}

pub fn arithmetic_mean(
    x_left_seconds: u64,
    x_right_seconds: u64,
    y_left: Decimal,
    y_right: Decimal,
) -> Decimal {
    ((y_right - y_left) * 60) / (x_right_seconds - x_left_seconds) //convert to minutes since that's how we accumulate
}

pub fn geometric_mean(
    x_left_seconds: u64,
    x_right_seconds: u64,
    y_left: Decimal,
    y_right: Decimal,
) -> Decimal {
    let exponent = arithmetic_mean(x_left_seconds, x_right_seconds, y_left, y_right);
    exponent.exp().unwrap()
}

fn interpolate_with_neighbors_(
    neighbors: (AccumulatedObservation, AccumulatedObservation),
    target: u64,
) -> AccumulatedObservation {
    let (left, right) = neighbors;

    assert!(left.timestamp <= right.timestamp);

    if target == left.timestamp {
        return left;
    }

    if target == right.timestamp {
        return right;
    }

    let slope =
        (right.price_sqrt_log_acc - left.price_sqrt_log_acc) / (right.timestamp - left.timestamp);

    let y_target = left.price_sqrt_log_acc + slope * (target - left.timestamp);

    AccumulatedObservation {
        timestamp: target,
        price_sqrt_log_acc: y_target,
    }
}

fn interpolate(
    observations: &Vec<AccumulatedObservation>,
    target_seconds: u64,
) -> AccumulatedObservation {
    let neighbors = find_neighbors(observations, target_seconds);

    if neighbors.0.timestamp == target_seconds {
        return neighbors.0;
    }

    if neighbors.1.timestamp == target_seconds {
        return neighbors.1;
    }

    let interpolated = interpolate_with_neighbors_(neighbors, target_seconds);

    interpolated
}

pub fn get_observation(
    observations: &Vec<AccumulatedObservation>,
    target_seconds: u64,
) -> AccumulatedObservation {
    let target_seconds = (target_seconds / 60) * 60;
    interpolate(observations, target_seconds)
}

pub fn interpolate_batch_seconds(
    observations: &Vec<AccumulatedObservation>,
    seconds: &Vec<u64>,
) -> Vec<AccumulatedObservation> {
    let mut result: Vec<AccumulatedObservation> = vec![];

    for &seconds_ in seconds {
        result.push(get_observation(observations, seconds_));
    }

    result
}

pub fn interpolate_batch_minutes(
    observations: &Vec<AccumulatedObservation>,
    minutes: &Vec<u64>,
) -> Vec<AccumulatedObservation> {
    let seconds = minutes.iter().map(|min| min * 60).collect();
    interpolate_batch_seconds(&observations, &seconds)
}

fn get_interval_(seconds: &Vec<u64>, left_seconds: u64, right_seconds: u64) -> ObservationInterval {
    let observations = get_observations_from_swap_seconds(seconds);

    let left_seconds = (left_seconds / 60) * 60;
    let right_seconds = (right_seconds / 60) * 60;

    let left_interpolated = get_observation(&observations, left_seconds);

    let right_interpolated = get_observation(&observations, right_seconds);

    let average_price_sqrt = geometric_mean(
        left_seconds,
        right_seconds,
        left_interpolated.price_sqrt_log_acc,
        right_interpolated.price_sqrt_log_acc,
    );

    ObservationInterval {
        start: left_seconds,
        end: right_seconds,
        price_sqrt: average_price_sqrt,
    }
}

pub fn convert_observation_minutes_to_swap_state_seconds(minutes: &Vec<u64>) -> Vec<u64> {
    let mut minutes = minutes.clone();
    minutes.insert(0, minutes[0] - 1);
    let seconds = minutes.iter().map(|min| min * 60).collect();
    seconds
}

pub fn get_intervals_from_swap_seconds(
    seconds: &Vec<u64>,
    bounds: &Vec<(u64, u64)>,
) -> Vec<ObservationInterval> {
    let mut result: Vec<ObservationInterval> = vec![];
    for &(left, right) in bounds {
        result.push(get_interval_(seconds, left, right));
    }
    result
}

pub fn get_intervals_from_observation_minutes(
    minutes: &Vec<u64>,
    bounds_minutes: &Vec<(u64, u64)>,
) -> Vec<ObservationInterval> {
    let seconds = convert_observation_minutes_to_swap_state_seconds(minutes);
    let bounds = bounds_minutes
        .iter()
        .map(|(left, right)| (left * 60, right * 60))
        .collect();
    let result = get_intervals_from_swap_seconds(&seconds, &bounds);
    result
}

/////////////////////////////////
/// HELPERS TESTS
use test_case::test_case;
#[test_case(vec![70, 75, 85, 85, 90], vec![5, 10, 0, 5, 30])]
#[test_case(vec![70, 70, 80, 90], vec![0, 10, 10, 30])]
#[test_case(vec![100, 105], vec![5, 15])]
fn test_get_averages_from_swap_seconds_single_minute(seconds: Vec<u64>, weights: Vec<u64>) {
    let averages_expected = weighted_average(TEST_DATAPOINTS.clone(), weights);
    let averages_result = get_averages_from_swap_seconds(&seconds);

    assert_eq!(averages_result, vec![averages_expected]);

    let observations_expected: Vec<AccumulatedObservation> = vec![];
    let observations_result = get_observations_from_swap_seconds(&seconds);

    assert_eq!(observations_result, observations_expected);
}

#[test]
fn test_get_averages_from_swap_seconds_multi_minute_no_leak() {
    let seconds: Vec<u64> = vec![70, 75, 85, 120, 130];
    let mut averages_expected: Vec<PreciseDecimal> = vec![];

    let averages_result = get_averages_from_swap_seconds(&seconds);

    averages_expected.push(weighted_average(
        TEST_DATAPOINTS[0..3].to_vec(),
        vec![5, 10, 35],
    ));
    averages_expected.push(weighted_average(
        TEST_DATAPOINTS[3..].to_vec(),
        vec![10, 50],
    ));

    assert_eq!(averages_result, averages_expected);

    let observations_result = get_observations_from_swap_seconds(&seconds);
    let mut observations_expected: Vec<AccumulatedObservation> = vec![];
    observations_expected.push(AccumulatedObservation {
        timestamp: 120,
        price_sqrt_log_acc: log(averages_expected[0]),
    });

    assert_eq!(observations_result, observations_expected);
}

#[test]
fn test_get_averages_from_swap_seconds_multi_minute_leak() {
    let seconds: Vec<u64> = vec![70, 75, 85, 125, 135];
    let mut averages_expected: Vec<PreciseDecimal> = vec![];

    let result = get_averages_from_swap_seconds(&seconds);

    averages_expected.push(weighted_average(
        TEST_DATAPOINTS[0..3].to_vec(),
        vec![5, 10, 35],
    ));
    averages_expected.push(weighted_average(
        TEST_DATAPOINTS[2..].to_vec(),
        vec![5, 10, 45],
    ));

    assert_eq!(result, averages_expected);

    let observations_result = get_observations_from_swap_seconds(&seconds);
    let mut observations_expected: Vec<AccumulatedObservation> = vec![];
    observations_expected.push(AccumulatedObservation {
        timestamp: 120,
        price_sqrt_log_acc: log(averages_expected[0]),
    });

    assert_eq!(observations_result, observations_expected);
}

#[test]
fn test_get_averages_from_swap_seconds_multi_minute_leak_2() {
    let seconds: Vec<u64> = vec![70, 75, 85, 125, 135, 150, 150, 160, 187, 205];
    let mut averages_expected: Vec<PreciseDecimal> = vec![];

    let result = get_averages_from_swap_seconds(&seconds);

    averages_expected.push(weighted_average(
        TEST_DATAPOINTS[0..3].to_vec(),
        vec![5, 10, 35],
    ));
    averages_expected.push(weighted_average(
        TEST_DATAPOINTS[2..8].to_vec(),
        vec![5, 10, 15, 0, 10, 20],
    ));
    averages_expected.push(weighted_average(
        TEST_DATAPOINTS[7..].to_vec(),
        vec![7, 18, 35],
    ));

    assert_eq!(result, averages_expected);

    let observations_result = get_observations_from_swap_seconds(&seconds);
    let mut observations_expected: Vec<AccumulatedObservation> = vec![];
    observations_expected.push(AccumulatedObservation {
        timestamp: 120,
        price_sqrt_log_acc: log(averages_expected[0]),
    });
    observations_expected.push(AccumulatedObservation {
        timestamp: 180,
        price_sqrt_log_acc: accumulated_log(
            observations_expected[0].price_sqrt_log_acc,
            averages_expected[1],
            TEST_DATAPOINTS[7],
            1,
        ),
    });

    assert_eq!(observations_result, observations_expected);
}

#[test]
fn test_get_averages_from_swap_seconds_multi_minute_leak_big_gap() {
    let seconds: Vec<u64> = vec![123, 130, 130, 130, 135, 135, 607];
    let mut averages_expected: Vec<PreciseDecimal> = vec![];

    let result = get_averages_from_swap_seconds(&seconds);

    averages_expected.push(weighted_average(
        TEST_DATAPOINTS[0..6].to_vec(),
        vec![7, 0, 0, 5, 0, 45],
    ));
    averages_expected.push(weighted_average(TEST_DATAPOINTS[5..].to_vec(), vec![7, 53]));

    assert_eq!(result, averages_expected);

    let observations_result = get_observations_from_swap_seconds(&seconds);
    let mut observations_expected: Vec<AccumulatedObservation> = vec![];
    observations_expected.push(AccumulatedObservation {
        timestamp: 600,
        price_sqrt_log_acc: accumulated_log(dec!(0), averages_expected[0], TEST_DATAPOINTS[5], 8),
    });

    assert_eq!(observations_result, observations_expected);
}

#[test]
fn test_get_averages_from_swap_seconds_multi_minute_leak_big_gap_2() {
    let seconds: Vec<u64> = vec![123, 607, 908];
    let mut averages_expected: Vec<PreciseDecimal> = vec![];

    let result = get_averages_from_swap_seconds(&seconds);

    averages_expected.push(weighted_average(TEST_DATAPOINTS[0..1].to_vec(), vec![57]));
    averages_expected.push(weighted_average(
        TEST_DATAPOINTS[0..2].to_vec(),
        vec![7, 53],
    ));
    averages_expected.push(weighted_average(TEST_DATAPOINTS[1..].to_vec(), vec![8, 52]));

    assert_eq!(result, averages_expected);

    let observations_result = get_observations_from_swap_seconds(&seconds);
    let mut observations_expected: Vec<AccumulatedObservation> = vec![];
    observations_expected.push(AccumulatedObservation {
        timestamp: 600,
        price_sqrt_log_acc: accumulated_log(dec!(0), averages_expected[0], TEST_DATAPOINTS[0], 8),
    });
    observations_expected.push(AccumulatedObservation {
        timestamp: 900,
        price_sqrt_log_acc: accumulated_log(
            observations_expected[0].price_sqrt_log_acc,
            averages_expected[1],
            TEST_DATAPOINTS[1],
            5,
        ),
    });

    assert_eq!(observations_result, observations_expected);
}

///////////////////////////////////////////
/// Test Find neighbors

fn new_observation(seconds: u64) -> AccumulatedObservation {
    AccumulatedObservation {
        timestamp: seconds,
        price_sqrt_log_acc: Decimal::from(seconds),
    }
}

#[test_case(180, 180, 180)]
#[test_case(600, 600, 600)]
#[test_case(240, 180, 300)]
#[test_case(420, 360, 480)]
#[test_case(300, 300, 300)]
#[test_case(360, 360, 360)]
fn test_find_neighbors(target: u64, left: u64, right: u64) {
    let seconds = vec![180, 300, 360, 480, 600];
    let mut observations: Vec<AccumulatedObservation> = vec![];

    for seconds_ in seconds {
        observations.push(new_observation(seconds_));
    }

    let neighbors = find_neighbors(&observations, target);

    assert_eq!(neighbors, (new_observation(left), new_observation(right)));
}

#[test_case(180, 360, 187)]
#[test_case(180, 360, 353)]
#[test_case(180, 360, 240)]
#[test_case(180, 420, 300)]
#[test_case(180, 420, 180)]
#[test_case(180, 420, 420)]
fn test_interpolate_(left: u64, right: u64, target: u64) {
    let left = new_observation(left);
    let right = new_observation(right);

    let result = interpolate_with_neighbors_((left, right), target);

    let expected = AccumulatedObservation {
        timestamp: target,
        price_sqrt_log_acc: Decimal::from(target),
    };

    assert_eq!(result, expected);
}

#[test_case(187)]
#[test_case(353)]
#[test_case(240)]
#[test_case(300)]
#[test_case(180)]
#[test_case(420)]
fn test_interpolate(target: u64) {
    let seconds = vec![180, 300, 360, 480, 600];
    let mut observations: Vec<AccumulatedObservation> = vec![];

    for seconds_ in seconds {
        observations.push(new_observation(seconds_));
    }

    let result = get_observation(&observations, target);

    let target_rounded = (target / 60) * 60;
    let expected = AccumulatedObservation {
        timestamp: target_rounded,
        price_sqrt_log_acc: Decimal::from(target_rounded),
    };

    assert_eq!(result, expected);
}

#[test_case(310, 350 => panics)]
#[test_case(310, 490)]
fn test_get_interval_average(target_left: u64, target_right: u64) {
    let seconds = vec![180, 300, 360, 480, 600];
    let target_left_rounded = (target_left / 60) * 60;
    let target_right_rounded = (target_right / 60) * 60;

    let observations = get_observations_from_swap_seconds(&seconds);

    let interpolated_left = get_observation(&observations, target_left_rounded);
    let interpolated_right = get_observation(&observations, target_right_rounded);

    let mean = geometric_mean(
        target_left_rounded,
        target_right_rounded,
        interpolated_left.price_sqrt_log_acc,
        interpolated_right.price_sqrt_log_acc,
    );

    let expected = ObservationInterval {
        start: target_left_rounded,
        end: target_right_rounded,
        price_sqrt: mean,
    };

    let result = get_interval_(&seconds, target_left, target_right);

    assert_eq!(result, expected);
}
