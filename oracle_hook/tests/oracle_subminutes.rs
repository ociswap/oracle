mod helper;
use common::{
    hooks::AfterSwapState,
    oracle::{AccumulatedObservation, ObservationInterval},
};
use helper::*;
use pretty_assertions::assert_eq;
use scrypto::prelude::*;
use scrypto_math::*;
use scrypto_testenv::environment::TestHelperExecution;
use test_case::test_case;

#[test]
fn test_first_minute_same_transaction() {
    // let seconds: Vec<u64> = (0..9).collect();
    let mut helper = OracleTestHelper::new_with_swap_state_seconds(&vec![]);

    let states = AfterSwapState::from_test_datapoints(
        10,
        helper.pool_address.unwrap(),
        helper.x_address(),
        helper.y_address(),
    );

    helper.jump_to_timestamp_seconds(120);
    helper.load_hook_auth();
    helper.after_swap(states[0].clone(), helper.y_address(), dec!("1.2344"));
    helper.after_swap(states[1].clone(), helper.y_address(), dec!("1.2344"));
    helper.execute_expect_success(false);

    helper.jump_to_timestamp_seconds(180);
    helper.load_hook_auth();
    helper.after_swap(states[2].clone(), helper.y_address(), dec!("1.2344"));
    helper.execute_expect_success(false);

    let receipt = helper.observation(180).execute_expect_success(false);

    let result: Vec<AccumulatedObservation> = receipt.outputs("observation");

    assert_eq!(
        result,
        vec![AccumulatedObservation {
            timestamp: 180,
            price_sqrt_log_acc: log(states[1].price_sqrt),
        }]
    );
}

#[test]
fn test_first_minute_same_second() {
    // let seconds: Vec<u64> = (0..9).collect();
    let seconds: Vec<u64> = vec![90, 90, 90, 120];

    let mut helper = OracleTestHelper::new_with_swap_state_seconds(&seconds);

    let receipt = helper.observation(120).execute_expect_success(false);

    let output: Vec<AccumulatedObservation> = receipt.outputs("observation");

    assert_eq!(
        output,
        vec![AccumulatedObservation {
            timestamp: 120,
            price_sqrt_log_acc: log(TEST_DATAPOINTS[2]),
        }]
    )
}

#[test]
fn test_first_minute_multiple_seconds() {
    // let seconds: Vec<u64> = (0..9).collect();
    let seconds: Vec<u64> = vec![90, 105, 105, 110, 110, 110, 120];

    let mut helper = OracleTestHelper::new_with_swap_state_seconds(&seconds);

    let receipt = helper.observation(120).execute_expect_success(false);

    let output: Vec<AccumulatedObservation> = receipt.outputs("observation");

    let weighted_average = weighted_average(TEST_DATAPOINTS.clone(), vec![15, 0, 5, 0, 0, 10]);

    assert_eq!(
        output,
        vec![AccumulatedObservation {
            timestamp: 120,
            price_sqrt_log_acc: log(weighted_average),
        }]
    )
}

#[test_case(vec![90, 105, 105, 110, 110, 110, 120], vec![15, 0, 5, 0, 0, 10], 120)]
fn test_first_minute_multiple_seconds_2(
    seconds: Vec<u64>,
    weights: Vec<u64>,
    inspected_timestamp: u64,
) {
    // let seconds: Vec<u64> = (0..9).collect();

    let mut helper = OracleTestHelper::new_with_swap_state_seconds(&seconds);

    let receipt = helper
        .observation(inspected_timestamp)
        .execute_expect_success(false);

    let output: Vec<AccumulatedObservation> = receipt.outputs("observation");

    let weighted_average = weighted_average(TEST_DATAPOINTS.clone(), weights);

    assert_eq!(
        output,
        vec![AccumulatedObservation {
            timestamp: inspected_timestamp,
            price_sqrt_log_acc: log(weighted_average),
        }]
    )
}

// #[test_case(240, 30, 20)]
// fn test_first_minute_multiple_seconds_3(seconds: u64, weights: u64 , inspected_timestamp: u64) {
//     // let seconds: Vec<u64> = (0..9).collect();

//     let mut helper = OracleTestHelper::new_with_swap_state_seconds(&seconds);

//     let receipt = helper.observation(inspected_timestamp).execute_expect_success(false);

//     let output: Vec<AccumulatedObservation> = receipt.outputs("observation");

//     let weighted_average = weighted_average(TEST_DATAPOINTS.clone(), weights);

//     assert_eq!(
//         output,
//         vec![AccumulatedObservation {
//             timestamp: inspected_timestamp,
//             price_sqrt_log_acc: log(weighted_average),
//         }]
//     )
// }

#[test_case(vec![70, 75, 85, 120, 130])]
#[test_case(vec![70, 75, 85, 125, 135])]
#[test_case(vec![70, 75, 85, 125, 135, 150, 150, 160, 187, 205])]
#[test_case(vec![123, 130, 130, 130, 135, 135, 607])]
#[test_case(vec![123, 607, 908])]
fn test_assert_observations_batch(seconds: Vec<u64>) {
    assert_observations_batch(&seconds, get_observations_from_swap_seconds(&seconds));
}

// #[test]
// fn test_assert_observations_batch1() {
//     let seconds: Vec<u64> = vec![123, 130, 130, 130, 135, 135, 607];
//     let weights: Vec<u64> = vec![7, 0, 0, 5, 0, 45];
//     println!("Averages: {:?}", get_averages_from_swap_seconds(&seconds));
//     let average = weighted_average(TEST_DATAPOINTS.clone(), weights);
//     let average_log = average.ln();
//     println!("average: {}", average);
//     println!("average_ln: {:?}", average_log);
// }

// #[test]
// fn test_assert_observations_batch2() {
//     // let seconds: Vec<u64> = vec![TEST_DATAPOINTS[0], ];
//     let seconds: Vec<u64> = vec![123, 607, 908];

//     let manual_averages: Vec<PreciseDecimal> = vec![
//         TEST_DATAPOINTS[0],
//         (TEST_DATAPOINTS[0] * 7 + TEST_DATAPOINTS[1] * 53) / 60
//         // (TEST_DATAPOINTS[1]*8+TEST_DATAPOINTS[2]*52)/60
//     ];

//     let mut manual_accumulation: Vec<Decimal> = vec![];
//     manual_accumulation.push(log(manual_averages[0]) + log(TEST_DATAPOINTS[0]) * 7);
//     manual_accumulation.push(
//         manual_accumulation[0] + log(manual_averages[1]) + log(TEST_DATAPOINTS[1]) * 4
//     );

//     println!("\nAverages:\n{:#?}", get_averages_from_swap_seconds(&seconds));
//     println!("\nAverages (manual):\n{:#?}", manual_averages);
//     println!("\nAccumulation:\n{:#?}", get_observations_from_swap_seconds(&seconds));
//     println!("\nAccumulation (manual):\n{:#?}", manual_accumulation);

//     // println!("Averages: {:?}", get_averages_from_swap_seconds(&seconds));
//     // let average = weighted_average(TEST_DATAPOINTS.clone(), weights);
//     // let average_log = average.ln();
//     // println!("average: {}", average);
//     // println!("average_ln: {:?}", average_log);

//     // numbers = [123, 130, 130, 130, 135, 135]
//     // weights = [7, 0, 0, 5, 0, 45]
// }

// binary search already partially being tested above
// observations which don't exist (binary search + interpolation);
// returned averages
