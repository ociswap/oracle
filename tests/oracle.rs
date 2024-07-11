mod helper;
use helper::*;
use oracle::{oracle::accumulated_log, AccumulatedObservation, ObservationInterval};
use pretty_assertions::assert_eq;
use scrypto::prelude::*;
use scrypto_testenv::environment::TestHelperExecution;
use std::ops::Range;
use test_case::test_case;
use test_oracle::test_oracle::HookCall;

#[test]
fn test_instantiate_instant() {
    let mut helper = OracleTestHelper::new();
    helper.instantiate_instant();
    helper.execute_expect_success(false);
}

// Test access

#[test]
fn test_after_instantiate_auth_failure() {
    let mut helper = OracleTestHelper::new();
    helper.instantiate_instant();

    let after_instantiate_state = helper.after_instantiate_state_empty();

    helper.after_instantiate(after_instantiate_state);
    helper.execute_expect_failure(false);
}

#[test]
fn test_after_instantiate_auth_success() {
    let mut helper = OracleTestHelper::new();
    helper.instantiate_instant();

    helper.load_hook_auth();
    helper.after_instantiate(helper.after_instantiate_state_empty());
    helper.execute_expect_success(false);
}

#[test]
fn test_before_swap_auth_failure() {
    let mut helper = OracleTestHelper::new();
    helper.instantiate_instant();

    helper.after_swap(helper.after_swap_state_dummy(), helper.y_address(), dec!(1));
    helper.execute_expect_failure(false);
}

#[test]
fn test_before_swap_auth_success() {
    let mut helper = OracleTestHelper::new();
    helper.instantiate_instant();

    helper.load_hook_auth();
    helper.after_swap(helper.after_swap_state_dummy(), helper.y_address(), dec!(1));
    helper.execute_expect_success(false);
}

// Get calls

#[test]
fn test_get_calls() {
    let mut helper = OracleTestHelper::new();
    helper.instantiate_instant();

    helper.get_calls();
    let receipt = helper.execute_expect_success(false);
    let output: Vec<Vec<HookCall>> = receipt.outputs("get_calls");

    assert_eq!(
        output,
        vec![vec![HookCall::AfterInstantiate, HookCall::AfterSwap]]
    );
}

// Assert inputs unchanged by calls

#[test]
fn test_after_instantiate_assert_outputs_equal_inputs() {
    let mut helper = OracleTestHelper::new();
    helper.assert_outputs_equal_inputs(HookCall::AfterInstantiate);
}

#[test]
fn test_after_swap_assert_outputs_equal_inputs() {
    let mut helper = OracleTestHelper::new();
    helper.assert_outputs_equal_inputs(HookCall::AfterSwap);
}

// Test ranges

#[test]
fn test_get_observation_for_timestamp_outside_range_before() {
    let mut helper = OracleTestHelper::new();
    helper.instantiate_instant();

    helper.jump_to_timestamp_minutes(5);
    helper
        .load_hook_auth()
        .after_swap_default()
        .execute_expect_success(false);
    helper.jump_to_timestamp_minutes(10);
    helper
        .load_hook_auth()
        .after_swap_default()
        .execute_expect_success(false);
    helper.jump_to_timestamp_minutes(15);
    helper
        .load_hook_auth()
        .after_swap_default()
        .execute_expect_success(false);
    helper.jump_to_timestamp_minutes(20);

    // Older than oldest observation
    helper
        .observation_in_minutes(4)
        .execute_expect_failure(false);
}

#[test]
fn test_get_observation_for_timestamp_outside_range_after() {
    let mut helper = OracleTestHelper::new();
    helper.instantiate_instant();

    helper.jump_to_timestamp_minutes(5);
    helper
        .load_hook_auth()
        .after_swap_default()
        .execute_expect_success(false);
    helper.jump_to_timestamp_minutes(10);
    helper
        .load_hook_auth()
        .after_swap_default()
        .execute_expect_success(false);
    helper.jump_to_timestamp_minutes(15);
    helper
        .load_hook_auth()
        .after_swap_default()
        .execute_expect_success(false);
    helper.jump_to_timestamp_minutes(20);

    // After current time
    helper
        .observation_in_minutes(21)
        .execute_expect_failure(false);
}

#[test]
fn test_get_observation_for_timestamp_inside_range_after() {
    let mut helper = OracleTestHelper::new();
    helper.instantiate_instant();

    helper.jump_to_timestamp_minutes(5);
    helper
        .load_hook_auth()
        .after_swap_default()
        .execute_expect_success(false);
    helper.jump_to_timestamp_minutes(10);
    helper
        .load_hook_auth()
        .after_swap_default()
        .execute_expect_success(false);
    helper.jump_to_timestamp_minutes(15);
    helper
        .load_hook_auth()
        .after_swap_default()
        .execute_expect_success(false);
    helper.jump_to_timestamp_minutes(20);

    // After latest observation but before current time
    helper
        .observation_in_minutes(16)
        .execute_expect_success(false);
}

#[test]
fn test_get_observation_for_timestamp_in_range() {
    let mut helper = OracleTestHelper::new();
    helper.instantiate_instant();

    helper.jump_to_timestamp_minutes(5 - 1);
    helper
        .load_hook_auth()
        .after_swap_default()
        .execute_expect_success(false);

    helper.jump_to_timestamp_minutes(5);
    helper
        .load_hook_auth()
        .after_swap_default()
        .execute_expect_success(false);
    helper.jump_to_timestamp_minutes(10);
    helper
        .load_hook_auth()
        .after_swap_default()
        .execute_expect_success(false);
    helper.jump_to_timestamp_minutes(15);
    helper
        .load_hook_auth()
        .after_swap_default()
        .execute_expect_success(false);
    helper.jump_to_timestamp_minutes(20);

    // Oldest observation
    helper
        .observation_in_minutes(5)
        .execute_expect_success(false);

    // Non-existing observation in range
    helper
        .observation_in_minutes(9)
        .execute_expect_success(false);

    // Existing observation in range
    helper
        .observation_in_minutes(10)
        .execute_expect_success(false);

    // Newest observation
    helper
        .observation_in_minutes(15)
        .execute_expect_success(false);

    // Non-existing, later than most recent observation
    helper
        .observation_in_minutes(17)
        .execute_expect_success(false);

    // Non-existing, after current time
    helper
        .observation_in_minutes(21)
        .execute_expect_failure(false);
}

// Test observations limit, increase capacity

#[test]
fn test_observations_limit() {
    let mut helper = OracleTestHelper::new();
    helper.instantiate_instant();
    helper.observations_limit();
    let receipt = helper.execute_expect_success(false);
    let outputs: Vec<u16> = receipt.outputs("observations_limit");

    assert_eq!(outputs, vec![10]);
}

// Observations stored
#[test]
fn test_observations_stored_0() {
    let mut helper = OracleTestHelper::new();
    helper.instantiate_instant();

    helper.observations_stored();
    let outputs: Vec<u16> = helper
        .execute_expect_success(false)
        .outputs("observations_stored");

    assert_eq!(outputs, vec![0]);
}

#[test]
fn test_observations_stored_1() {
    let timestamps: Vec<u64> = (4..5).collect();
    let mut helper = OracleTestHelper::new_with_observations_minutes(&timestamps);

    helper.observations_stored();
    let outputs: Vec<u16> = helper
        .execute_expect_success(false)
        .outputs("observations_stored");

    assert_eq!(outputs, vec![1]);
}

#[test]
fn test_observations_stored_10() {
    let timestamps: Vec<u64> = (4..14).collect();
    let mut helper = OracleTestHelper::new_with_observations_minutes(&timestamps);

    helper.observations_stored();
    let outputs: Vec<u16> = helper
        .execute_expect_success(false)
        .outputs("observations_stored");

    assert_eq!(outputs, vec![10]);
}

#[test]
fn test_observations_stored_15() {
    let timestamps: Vec<u64> = (4..15).collect();
    let mut helper = OracleTestHelper::new_with_observations_minutes(&timestamps);

    helper.observations_stored();
    let outputs: Vec<u16> = helper
        .execute_expect_success(false)
        .outputs("observations_stored");

    assert_eq!(outputs, vec![10]);
}

// test only the first observation in a timestamp creates and observation
#[test]
fn test_observations_stored_repeated_timestamp() {
    let timestamps: Vec<u64> = vec![4, 4, 4, 4];
    let mut helper = OracleTestHelper::new_with_observations_minutes(&timestamps);

    helper.observations_stored();
    let outputs: Vec<u16> = helper
        .execute_expect_success(false)
        .outputs("observations_stored");

    assert_eq!(outputs, vec![1]);
}

#[test]
fn test_observations_stored_repeated_timestamp_2() {
    let timestamps: Vec<u64> = vec![4, 5, 5, 7];
    let mut helper = OracleTestHelper::new_with_observations_minutes(&timestamps);

    helper.observations_stored();
    let outputs: Vec<u16> = helper
        .execute_expect_success(false)
        .outputs("observations_stored");

    assert_eq!(outputs, vec![3]);
}

// Last observation index
#[test_case(4..4, None)]
#[test_case(4..5, Some(0))]
#[test_case(4..14, Some(9))]
#[test_case(4..15, Some(0))]
#[test_case(4..16, Some(1))]
fn test_last_observation_index(timestamps: Range<u64>, expected: Option<u16>) {
    let timestamps: Vec<u64> = timestamps.collect();
    let mut helper = OracleTestHelper::new_with_observations_minutes(&timestamps);

    helper.last_observation_index();
    let outputs: Vec<Option<u16>> = helper
        .execute_expect_success(false)
        .outputs("last_observation_index");

    assert_eq!(outputs, vec![expected]);
}

// Oldest observation timestamp
#[test_case(4..4, None)]
#[test_case(4..5, Some(240))]
#[test_case(4..14, Some(240))]
#[test_case(4..15, Some(300))]
#[test_case(4..16, Some(360))]
fn test_oldest_observation_timestamp(timestamps: Range<u64>, expected: Option<u64>) {
    let timestamps: Vec<u64> = timestamps.collect();
    let mut helper = OracleTestHelper::new_with_observations_minutes(&timestamps);

    helper.oldest_observation_timestamp();
    let outputs: Vec<Option<u64>> = helper
        .execute_expect_success(false)
        .outputs("oldest_observation_timestamp");

    assert_eq!(outputs, vec![expected]);
}

// Get observation special cases

#[test]
fn test_observation_timestamp_older_than_oldest_fails() {
    let minutes: Vec<u64> = (2..20).collect();
    let mut helper = OracleTestHelper::new_with_observations_minutes(&minutes);

    helper
        .observation_in_minutes(9)
        .execute_expect_failure(false);
    helper
        .observation_in_minutes(10)
        .execute_expect_success(false);
}

#[test]
fn test_observation_timestamp_equals_last_timestamp() {
    let timestamps: Vec<u64> = (4..8).collect();
    let mut helper = OracleTestHelper::new_with_observations_minutes(&timestamps);

    let receipt = helper
        .observation_in_minutes(7)
        .execute_expect_success(false);

    let outputs: Vec<AccumulatedObservation> = receipt.outputs("observation");

    let expected_observations = get_observations_from_observation_minutes(&timestamps);
    let expected = interpolate_batch_minutes(&expected_observations, &vec![7]);

    assert_eq!(outputs, expected);
}

#[test]
fn test_observation_timestamp_later_than_last_but_before_current() {
    let minutes: Vec<u64> = (4..7).collect();

    let mut helper = OracleTestHelper::new_with_observations_minutes(&minutes);
    helper.jump_to_timestamp_minutes(20);
    helper.observation_in_minutes(12);

    let receipt = helper.execute_expect_success(false);
    let outputs: Vec<AccumulatedObservation> = receipt.outputs("observation");

    let seconds = convert_observation_minutes_to_swap_state_seconds(&minutes);
    let (averages, observations, last_value) = generate_oracle_data(&seconds);

    let last_observation = observations.last().unwrap();
    let last_average = averages.last().unwrap().clone();

    let price_sqrt_log_acc = accumulated_log(
        last_observation.price_sqrt_log_acc,
        last_average,
        last_value,
        6,
    );

    let expected = vec![AccumulatedObservation {
        timestamp: 12 * 60,
        price_sqrt_log_acc,
    }];

    assert_eq!(outputs, expected);
}

// Test binary search

#[test_case(4..5, 4..5, true; "one")] // last observation returned directly (no binary search)
#[test_case(4..6, 4..6, true; "two")]
#[test_case(4..6, 5..6, true; "two_only_last")] // last observation returned directly (no binary search)
#[test_case(4..9, 7..8, true; "second_last")]
#[test_case(4..14, 4..14, true; "even")]
#[test_case(4..13, 4..13, true; "odd")]
#[test_case(4..20, 10..20, true; "even_above_limit")]
#[test_case(4..15, 5..15, true; "odd_above_limit")]
#[test_case(4..20, 9..10, false; "even_too_old_above_shift")]
#[test_case(4..15, 4..5, false; "odd_too_old_above_limit")]
#[test_case(4..8, 4..8, true; "timestamp_equals_last_timestamp")]
#[test_case(4..8, 8..9, false; "later_than_current_time_fails")]
fn test_binary_search_observations(
    minutes: Range<u64>,
    target_minutes: Range<usize>,
    expect_success: bool,
) {
    let minutes: Vec<u64> = minutes.collect();

    let mut helper = OracleTestHelper::new_with_observations_minutes(&minutes);
    let expected_stored = get_observations_from_observation_minutes(&minutes);

    let mut expected: Vec<AccumulatedObservation> = vec![];
    for minute in target_minutes {
        let seconds = (minute * 60) as u64;
        if expect_success {
            helper.observation(seconds);
            expected.push(get_observation(&expected_stored, seconds));
        } else {
            helper.observation(seconds).execute_expect_failure(false);
        }
    }

    if !expect_success {
        return;
    }

    let receipt = helper.execute_expect_success(false);
    let output: Vec<AccumulatedObservation> = receipt.outputs("observation");

    assert_eq!(output, expected);
}

// Interpolation

#[test]
fn test_interpolation() {
    let timestamps: Vec<u64> = vec![3, 6, 10];

    let mut helper = OracleTestHelper::new_with_observations_minutes(&timestamps);
    helper.observation_in_minutes(4);
    helper.observation_in_minutes(8);

    let receipt = helper.execute_expect_success(false);
    let outputs: Vec<AccumulatedObservation> = receipt.outputs("observation");

    let observations_expected = get_observations_from_observation_minutes(&timestamps);
    let expected = vec![
        get_observation(&observations_expected, 4 * 60),
        get_observation(&observations_expected, 8 * 60),
    ];

    assert_vecs_similar(outputs, expected);
}

// Test observation_intervals

#[test]
fn test_observation_intervals_left_lesser_than_right() {
    let minutes: Vec<u64> = (4..9).collect();
    let intervals: Vec<(u64, u64)> = vec![(6, 7), (7, 8)];

    let intervals_in_seconds: Vec<(u64, u64)> = intervals
        .iter()
        .map(|(start, end)| (start * 60, end * 60))
        .collect();

    let mut helper = OracleTestHelper::new_with_observations_minutes(&minutes);
    helper.observation_intervals(intervals_in_seconds);
    let receipt = helper.execute_expect_success(false);

    let result: Vec<Vec<ObservationInterval>> = receipt.outputs("observation_intervals");

    let expected = get_intervals_from_observation_minutes(&minutes, &intervals);

    assert_eq!(result, vec![expected]);
}

#[test]
fn test_observation_intervals_left_bigger_than_right() {
    let timestamps: Vec<u64> = (4..9).collect();
    let intervals: Vec<(u64, u64)> = vec![(6, 7), (7, 6)];

    let intervals_in_seconds: Vec<(u64, u64)> = intervals
        .iter()
        .map(|(start, end)| (start * 60, end * 60))
        .collect();

    let mut helper = OracleTestHelper::new_with_observations_minutes(&timestamps);
    helper.observation_intervals(intervals_in_seconds);

    helper.execute_expect_failure(false);
}

#[test]
fn test_observation_intervals_left_equal_right() {
    let timestamps: Vec<u64> = (4..9).collect();
    let intervals: Vec<(u64, u64)> = vec![(6, 7), (7, 7)];

    let intervals_in_seconds: Vec<(u64, u64)> = intervals
        .iter()
        .map(|(start, end)| (start * 60, end * 60))
        .collect();

    let mut helper = OracleTestHelper::new_with_observations_minutes(&timestamps);
    helper.observation_intervals(intervals_in_seconds);

    helper.execute_expect_failure(false);
}

#[test]
fn test_observation_intervals() {
    let timestamps: Vec<u64> = (4..14).collect();

    let intervals: Vec<(u64, u64)> = vec![(4, 13), (5, 7), (7, 12)];
    let intervals_in_seconds: Vec<(u64, u64)> = intervals
        .iter()
        .map(|(start, end)| (start * 60, end * 60))
        .collect();

    let mut helper = OracleTestHelper::new_with_observations_minutes(&timestamps);
    helper.observation_intervals(intervals_in_seconds);
    let receipt = helper.execute_expect_success(false);
    let outputs: Vec<Vec<ObservationInterval>> = receipt.outputs("observation_intervals");

    let expected: Vec<Vec<ObservationInterval>> = vec![get_intervals_from_observation_minutes(
        &timestamps,
        &intervals,
    )];

    assert_eq!(outputs, expected);
}

#[test]
fn test_observation_intervals_seconds_rounding() {
    let timestamps: Vec<u64> = (4..14).collect();
    let swap_seconds = convert_observation_minutes_to_swap_state_seconds(&timestamps);

    let intervals_in_seconds: Vec<(u64, u64)> = vec![(635, 660), (660, 735)];

    let mut helper = OracleTestHelper::new_with_observations_minutes(&timestamps);
    helper.observation_intervals(intervals_in_seconds.clone());
    let receipt = helper.execute_expect_success(false);
    let outputs: Vec<Vec<ObservationInterval>> = receipt.outputs("observation_intervals");

    let expected: Vec<Vec<ObservationInterval>> = vec![get_intervals_from_swap_seconds(
        &swap_seconds,
        &intervals_in_seconds,
    )];

    assert_eq!(outputs, expected);
}
