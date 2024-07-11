use common::time::*;
use scrypto::prelude::*;
use scrypto_math::*;
use std::cmp::min;

/// The SubObservations object is used to accumulate and manage price square root states within
/// a given minute.
/// When a new minute is reached, it performs a time-weighted averaging of the minute's prices,
/// so that the resulting value is used for logarithmic accumulation in the Oracle object.
#[derive(ScryptoSbor)]
pub struct SubObservations {
    /// The sum of the product of the last price square root and the time elapsed since
    /// the last update.
    price_sqrt_sum: PreciseDecimal,
    /// The last observed price square root.
    price_sqrt_last: PreciseDecimal,
    /// The instant when the last update (call to `new_subobservation` or `finalize`) occurred.
    last_updated: Instant,
    /// The instant when the sub-observations were initialized. After the initialization is
    /// complete, this field is set to `None`.
    initialization: Option<Instant>,
}

impl SubObservations {
    pub fn new() -> Self {
        Self {
            last_updated: Clock::instant(),
            initialization: Some(Clock::instant()),
            price_sqrt_sum: pdec!(0),
            price_sqrt_last: pdec!(0),
        }
    }

    /// Updates the sub-observations with a new price square root value.
    ///
    /// This method is called at the end of every swap, to inform the oracle of the lastest
    /// pool `price_sqrt`. It performs the time-weighted accumulation of the `price_sqrt`s
    /// during each minute.
    ///
    /// # Arguments
    ///
    /// * `price_sqrt` - A `PreciseDecimal` representing the new price square root to be observed.
    pub fn new_subobservation(&mut self, price_sqrt: PreciseDecimal) {
        let current_instant = Clock::instant();

        if current_instant != self.last_updated {
            let delta_marginal_seconds =
                current_instant.seconds_marginal() - self.last_updated.seconds_marginal();
            self.price_sqrt_sum += self.price_sqrt_last * delta_marginal_seconds;
            self.last_updated = current_instant;
        }

        // Updates `price_sqrt_last` without performing accumulation if no seconds have elapsed
        // since the last swap. This approach ensures that for multiple swaps within the
        // same second, only the last swap's price is considered.
        // This mechanism is crucial as the code cannot predict if a swap will be the last within
        // a given second.
        self.price_sqrt_last = price_sqrt;
    }

    /// Calculates the time-weighted average `price_sqrt` for the last active minute
    /// (at which swaps took place) and resets the SubObservations object in order to
    /// prepare it for the new minute, by setting the time properly and resetting the
    /// `price_sqrt_sum`.
    ///
    /// # Returns
    ///
    /// Returns the time-weighted average `price_sqrt` for the last active minute.
    pub fn finalize(&mut self) -> PreciseDecimal {
        // Below, the duration across which the `price_sqrt_sum` is averaged is conditionally
        // set to either:
        // 1) The number of seconds passed between the first swap and the end of the minute,
        //  if this is the first minute being recorded by the SubMinutes object
        // 2) 60 seconds otherwise.
        // This ensures that the first minute is averaged fairly, from the moment at which the
        // first swap took place.
        let duration = match self.initialization.take() {
            Some(instant) => 60 - instant.seconds_marginal(),
            None => 60,
        };
        let price_sqrt_avg = self.price_sqrt_average(duration);

        // Prepare SubObservations for new minute
        // We set the instant rounded to the minute, as if the last transaction took place when the
        // minute dawned.
        // This is meant to allow the object to perform the accumulation correctly when the
        // first swap in the minute takes place, i.e. that the new price is weighted by the number
        // of seconds that passed since the beginning of the minute.
        self.last_updated = Clock::current_time_rounded_to_minutes();
        self.price_sqrt_sum = pdec!(0);

        price_sqrt_avg
    }

    /// Provides a preview of the time-weighted average `price_sqrt` during the last active minute,
    /// without changing the state of SubObservations.
    ///
    /// This method calculates a preview of the time-weighted average price square root without
    /// finalizing the current observations. It is useful for getting an estimate before the minute
    /// ends. This method asserts that no initial observation has been set yet, ensuring it's only
    /// called under appropriate conditions.
    ///
    /// # Returns
    ///
    /// Returns the time-weighted average price square root for the current minute based on the
    /// observations so far.
    pub fn finalize_preview(&self) -> PreciseDecimal {
        assert!(
            self.initialization.is_none(),
            "Not yet possible to retrieve this data. Please wait for a new observation to be
            stored."
        );
        self.price_sqrt_average(60)
    }

    /// Calculates the time-weighted average price square root over a given duration.
    ///
    /// This helper method computes the time-weighted average price square root by taking into
    /// account the sum of price square roots observed and the last observed price square root,
    /// adjusted for the time elapsed since the last observation.
    ///
    /// # Arguments
    ///
    /// * `duration` - The duration in seconds over which to average the price square roots.
    ///
    /// # Returns
    ///
    /// Returns the time-weighted average price square root over the specified duration.
    fn price_sqrt_average(&self, duration: u64) -> PreciseDecimal {
        let delta_marginal_seconds = 60 - self.last_updated.seconds_marginal();
        let price_sqrt_sum = self.price_sqrt_sum + self.price_sqrt_last * delta_marginal_seconds;

        price_sqrt_sum / duration
    }
}

#[derive(ScryptoSbor)]
pub struct Oracle {
    /// A key-value store holding accumulated observations, indexed by a u16, allowing for a
    /// maximum of 65535 observations.
    observations: KeyValueStore<u16, AccumulatedObservation>,
    /// The index of the last observation stored. None if no observations have been stored yet.
    last_observation_index: Option<u16>,
    /// The total number of observations that have been stored.
    observations_stored: u16,
    /// A SubObservations object, used for generating an average of the `price_sqrt` within each
    /// minute.
    sub_observations: Option<SubObservations>,
    observations_limit: u16,
}

impl Oracle {
    pub fn new(observations_limit: u16) -> Self {
        Oracle {
            observations: KeyValueStore::new(),
            observations_stored: 0,
            last_observation_index: None,
            sub_observations: None,
            observations_limit,
        }
    }

    /// Receives and updates the SubObservations object accordingly.
    ///
    /// This method is invoked at the end of each swap performed by the pool. It processes the
    /// square root of the pool's price resulting from the swap. The behavior varies depending on
    /// the state of the `sub_observations` field:
    /// 1) If `sub_observations` is uninitialized (typical before the first swap or `observe` call),
    /// it initializes this field.
    /// 2) If the current instant falls within the same minute as the last swap (and the subsequent
    /// `observe` call), it appends a new sub-observation to the SubObservations object using the
    /// provided price square root.
    /// 3) If the current instant marks the beginning of a new minute relative to the last swap:
    ///   a) It generates a new observation and stores it in the `observations` field.
    ///   b) Similarly to (2), a new sub-observation is inserted in the SubObservations object.
    ///
    /// # Arguments
    ///
    /// * `price_sqrt` - A `PreciseDecimal` representing the square root of the price at the end
    /// of the swap.
    pub fn observe(&mut self, price_sqrt: PreciseDecimal) {
        let current_instant = Clock::instant();

        match &self.sub_observations {
            None => {
                self.sub_observations = Some(SubObservations::new());
            }
            Some(sub_observations) => {
                if current_instant.minutes() != sub_observations.last_updated.minutes() {
                    let observation = self.create_observation();
                    self.insert_observation(observation);
                }
            }
        }

        self.sub_observations
            .as_mut()
            .unwrap()
            .new_subobservation(price_sqrt);
    }

    /// Creates a new `AccumulatedObservation` instance.
    ///
    /// This function calculates the accumulated log of the price square root over a period of time
    /// and creates a new observation with the current timestamp and the calculated value. It
    /// handles two scenarios:
    /// 1) If no observations have been stored yet, it initializes the first observation with the
    /// current price square root log accumulation.
    /// 2) If there are existing observations, it calculates the new accumulated value based on the
    /// last observation and the time elapsed since then.
    ///
    /// # Returns
    ///
    /// An `AccumulatedObservation` instance containing the current timestamp and the accumulated
    /// log of the price square root.
    fn create_observation(&mut self) -> AccumulatedObservation {
        let now_minutes = Clock::time_in_minutes();

        let sub_observations = self.sub_observations.as_mut().unwrap();
        let minutes_since_last = now_minutes - sub_observations.last_updated.minutes();
        let finalized = sub_observations.finalize();

        // Case 1: If no observations have been stored yet, the first observation is generated and
        // returned.
        if self.observations_stored == 0 {
            let initial_observation = AccumulatedObservation {
                timestamp: now_minutes,
                price_sqrt_log_acc: accumulated_log(
                    dec!(0),
                    finalized,
                    sub_observations.price_sqrt_last,
                    minutes_since_last,
                ),
            };

            return initial_observation;
        }

        // Case 2: A new observation is generated based on the last stored observation, and returned
        let last_observation = self
            .observations
            .get(&self.last_observation_index.unwrap())
            .unwrap();

        let price_sqrt_log_acc = accumulated_log(
            last_observation.price_sqrt_log_acc,
            finalized,
            sub_observations.price_sqrt_last,
            minutes_since_last,
        );

        AccumulatedObservation {
            timestamp: now_minutes,
            price_sqrt_log_acc,
        }
    }

    /// Inserts a given `AccumulatedObservation` into the oracle's observation list.
    ///
    /// # Arguments
    ///
    /// * `observation`: The `AccumulatedObservation` instance to be inserted into the observation
    /// list.
    fn insert_observation(&mut self, observation: AccumulatedObservation) {
        // Update the `last_observation_index` to point to the newly inserted observation's index,
        // ensuring it wraps around
        // based on the `OBSERVATIONS_LIMIT` to mimic a circular queue and avoid out-of-bounds
        // errors.
        self.last_observation_index = match self.last_observation_index {
            // The index is set to zero on the first call
            None => Some(0),
            // The index is incremented by one, and if it exceeds the limit, it wraps around to zero
            Some(last_observation_index) => {
                Some((last_observation_index + 1) % self.observations_limit)
            }
        };
        self.observations
            .insert(self.last_observation_index.unwrap(), observation);

        // The `observations_stored` count is also incrementing, ensuring it
        // does not exceed the `OBSERVATIONS_LIMIT`.
        self.observations_stored = min(self.observations_stored + 1, self.observations_limit);
    }

    /// Retrieves an `AccumulatedObservation` for a given timestamp in seconds.
    /// The timestamp is automatically rounded to the minute.
    ///
    /// # Arguments
    ///
    /// * `seconds`: The timestamp in seconds for which an observation is sought.
    ///
    /// # Returns
    ///
    /// An `AccumulatedObservation` corresponding to the given timestamp.
    pub fn observation(&self, seconds: u64) -> AccumulatedObservation {
        let mut observation = self.observation_internal(seconds / 60);
        observation.timestamp *= 60;
        observation
    }

    /// Returns an `AccumulatedObservation` for a given timestamp.
    ///
    /// # Arguments
    /// * `target_minutes` - The target timestamp in minutes for which an observation is sought.
    ///
    /// # Scenarios
    /// 1) If an observation exists for the provided timestamp, it is returned.
    /// 2) If no observation matches the timestamp, but the timestamp is within the range captured
    /// by the oracle, an interpolated observation is generated from the two closest ones.
    /// 3) If the timestamp is more recent than the latest stored timestamp, but lesser or equal
    /// than the current timestamp,
    ///   a new observation is extrapolated.
    /// 4) Other timestamps will cause a panic.
    ///
    /// # Returns
    /// An `AccumulatedObservation` corresponding to the given timestamp.
    ///
    /// # Panics
    /// Panics if the `target_minutes` is not within the range of the oldest timestamp and the
    /// current time.
    fn observation_internal(&self, target_minutes: u64) -> AccumulatedObservation {
        // Unix minutes
        let now_minutes = Clock::time_in_minutes();

        // Assert that the target timestamp is in the allowed range
        // (oldest_timestamp <= target_minutes <= now)
        let oldest_timestamp = self
            .oldest_observation_timestamp_minutes()
            .expect("No observations exist yet.");
        assert!(
            target_minutes >= oldest_timestamp && target_minutes <= now_minutes,
            "Timestamp {} (rounded to the minute) not in range. The available range is [{}, {}]",
            target_minutes * 60,
            oldest_timestamp * 60,
            now_minutes * 60
        );

        let last_observation = self
            .observations
            .get(&self.last_observation_index.unwrap())
            .unwrap();

        if target_minutes == last_observation.timestamp {
            return last_observation.clone();
        }

        let sub_observations = self.sub_observations.as_ref().unwrap();

        if target_minutes > last_observation.timestamp {
            let minutes_since_last = target_minutes - last_observation.timestamp;

            let price_sqrt_log_acc = accumulated_log(
                last_observation.price_sqrt_log_acc,
                sub_observations.finalize_preview(),
                sub_observations.price_sqrt_last,
                minutes_since_last,
            );

            return AccumulatedObservation {
                timestamp: target_minutes,
                price_sqrt_log_acc,
            };
        }

        binary_search_and_interpolation(
            &self.observations,
            self.oldest_index().unwrap(),
            self.observations_stored,
            target_minutes,
        )
    }

    /// Calculates the geometric mean of the price square root over specified intervals.
    /// The timestamps are automatically rounded to the minute.
    ///
    /// # Arguments
    /// * `intervals_in_seconds` - A vector of tuples where each tuple contains two `u64` values
    /// representing the start and end of an interval in Unix seconds.
    ///
    /// # Returns
    /// A vector of `ObservationInterval` structs. Each `ObservationInterval` contains:
    /// * `start`: The start of the interval in Unix seconds.
    /// * `end`: The end of the interval in Unix seconds.
    /// * `price_sqrt`: The calculated geometric mean of the price square root for the interval.
    pub fn observation_intervals(
        &self,
        intervals_in_seconds: Vec<(u64, u64)>,
    ) -> Vec<ObservationInterval> {
        let mut averages = Vec::with_capacity(intervals_in_seconds.len());

        for (t_left_seconds, t_right_seconds) in intervals_in_seconds {
            let t_left_minutes = t_left_seconds / 60;
            let t_right_minutes = t_right_seconds / 60;

            assert!(
                t_left_minutes < t_right_minutes,
                "Provided intervals in seconds must be of the type [a, b], where a/60 < b/60,
                i.e. they must round down to different minutes. Interval [{}, {}] does not obey this
                condition.",
                t_left_seconds,
                t_right_seconds
            );

            let o_l = self.observation_internal(t_left_minutes);
            let o_r = self.observation_internal(t_right_minutes);

            let price_sqrt = geometric_mean(
                t_left_minutes,
                t_right_minutes,
                o_l.price_sqrt_log_acc,
                o_r.price_sqrt_log_acc,
            );

            averages.push(ObservationInterval {
                start: t_left_minutes * 60,
                end: t_right_minutes * 60,
                price_sqrt,
            });
        }

        averages
    }

    /// Returns the limit of observations that can be stored.
    ///
    /// # Returns
    ///
    /// A `u16` value representing the maximum number of observations that can be stored in the oracle.
    pub fn observations_limit(&self) -> u16 {
        self.observations_limit
    }

    /// Returns the number of observations currently stored.
    ///
    /// # Returns
    ///
    /// A `u16` value representing the number of observations currently stored in the oracle.
    pub fn observations_stored(&self) -> u16 {
        self.observations_stored
    }

    /// Returns the timestamp of the oldest observation in minutes, if any.
    ///
    /// # Returns
    ///
    /// An `Option<u64>` containing the timestamp of the oldest observation in minutes,
    /// or `None` if there are no observations.
    fn oldest_observation_timestamp_minutes(&self) -> Option<u64> {
        self.oldest_index()
            .and_then(|index| self.observations.get(&index))
            .map(|obs| obs.timestamp)
    }

    /// Returns the timestamp of the oldest observation in seconds, if any.
    ///
    /// # Returns
    ///
    /// An `Option<u64>` containing the timestamp of the oldest observation in seconds,
    /// or `None` if there are no observations.
    pub fn oldest_observation_timestamp(&self) -> Option<u64> {
        self.oldest_observation_timestamp_minutes()
            .map(|timestamp| timestamp * 60)
    }

    /// Returns the index of the last observation for testing purposes.
    ///
    /// # Returns
    ///
    /// An `Option<u16>` containing the index of the last observation,
    /// or `None` if there are no observations.
    pub fn last_observation_index(&self) -> Option<u16> {
        self.last_observation_index
    }

    /// Returns the index of the oldest observation.
    ///
    /// # Returns
    ///
    /// An `Option<u16>` containing the index of the oldest observation,
    /// or `None` if there are no observations.
    fn oldest_index(&self) -> Option<u16> {
        self.last_observation_index
            .map(|index| (index + 1) % self.observations_stored)
    }
}

/// Represents an accumulated observation at a specific timestamp.
///
/// This struct holds the timestamp of the observation and the accumulated
/// logarithmic value of the price square root up to this point.
#[derive(ScryptoSbor, Clone, Debug, PartialEq)]
pub struct AccumulatedObservation {
    /// The timestamp of the observation.
    pub timestamp: u64,
    /// The accumulated logarithmic value of the price square root.
    pub price_sqrt_log_acc: Decimal,
}

/// Represents an interval between two observations.
///
/// This struct defines an interval with a start and end timestamp, and the
/// calculated price square root for this interval.
#[derive(ScryptoSbor, Clone, Debug, PartialEq)]
pub struct ObservationInterval {
    /// The start timestamp of the interval.
    pub start: u64,
    /// The end timestamp of the interval.
    pub end: u64,
    /// The calculated price square root for the interval.
    pub price_sqrt: Decimal,
}

impl AccumulatedObservation {
    pub fn empty() -> Self {
        AccumulatedObservation {
            timestamp: 0,
            price_sqrt_log_acc: dec!(0),
        }
    }
}

/// Calculates the accumulated logarithmic value, which will be used later as one of the points to
/// calculate interval averages and returns it.
///
/// # Arguments
/// * `acc_value`: The accumulated value so far.
/// * `finalized`: The value registered during the last observation,
/// used to calculate the log value.
/// * `last_value`: The value received during the current timestamp.
/// * `minutes_since_last`: The number of minutes since the last observation.
///
/// # Returns
/// Returns the new accumulated logarithmic value as a `Decimal`.
pub fn accumulated_log(
    acc_value: Decimal,
    finalized: PreciseDecimal,
    last_value: PreciseDecimal,
    minutes_since_last: u64,
) -> Decimal {
    let finalized_log = finalized
        .ln()
        .unwrap()
        .checked_truncate(RoundingMode::ToNegativeInfinity)
        .unwrap();
    let last_value_log = last_value
        .ln()
        .unwrap()
        .checked_truncate(RoundingMode::ToNegativeInfinity)
        .unwrap();

    acc_value + finalized_log + (minutes_since_last - 1) * last_value_log
}

/// Performs binary search in the list of AccumulatedObservations in order to find the one matching
/// the `target_timestamp``
/// If no observation exists with the provided `target_timestamp`, then interpolation is performed
/// with the two closest ones. `
///
/// # Arguments
/// * `observations` - A reference to the KeyValueStore holding AccumulatedObservation instances,
/// indexed by a u16 key.
/// * `oldest_index` - The index within the store of the oldest observation.
/// * `observations_stored` - The total number of observations stored.
/// * `target_timestamp` - The specific timestamp for which an observation is sought.
///
/// # Returns
/// Returns an AccumulatedObservation instance. This will either be the observation that exactly
/// matches the `target_timestamp`, or an interpolated observation between the two closest
/// timestamps.
fn binary_search_and_interpolation(
    observations: &KeyValueStore<u16, AccumulatedObservation>,
    oldest_index: u16,
    observations_stored: u16,
    target_timestamp: u64,
) -> AccumulatedObservation {
    // The binary search is always called with target_timestamp within the range [left, right]
    let mut left = oldest_index;
    let mut right = left + observations_stored - 1;

    let (o_left, o_right) = loop {
        let mid = (left + right) / 2;
        let index_mid = mid % observations_stored;
        let observation_mid = observations.get(&index_mid).unwrap();

        if observation_mid.timestamp == target_timestamp {
            return observation_mid.clone();
        }

        // This situation occurs only when left and right are adjacent (index distance < 2).
        // If the index distance between left and right is greater than 1, then mid will be greater than left.
        // The case where mid equals left is already handled when mid is the target.
        if mid == left {
            let index_right = right % observations_stored;
            let observation_right = observations.get(&index_right).unwrap();

            // Unlike a regular binary search, we perform interpolation when only two elements remain,
            // and neither the left nor right element matches the target timestamp.
            // To prevent infinite loops, we return the observation_right if it is the target.
            // This situation occurs when left and right are adjacent, and right is the target.
            // For this to happen, the target must be the last observation (initially right), otherwise, we never reach this point,
            // because the right was the previous mid, and we always check if mid is the target first.
            // Only in the first iteration can mid be left and the target be right simultaneously.
            if observation_right.timestamp == target_timestamp {
                return observation_right.clone();
            }
            break (observation_mid, observation_right);
        }

        // In this binary search variant, we avoid adjusting mid by +1 or -1 because we will interpolate between the final two elements.
        // To prevent infinite loops, we check if observation_right's timestamp matches the target_timestamp when mid equals left.
        // At this stage, observation_mid.timestamp is guaranteed not to equal the target timestamp.
        // Until left and right are not adjacent and directly next to each other we will make progress in each iteration.
        if observation_mid.timestamp < target_timestamp {
            left = mid;
        } else {
            right = mid;
        }
    };

    // Interpolation
    let price_sqrt_log_acc = linear_interpolation(
        o_left.timestamp,
        o_right.timestamp,
        o_left.price_sqrt_log_acc,
        o_right.price_sqrt_log_acc,
        target_timestamp,
    );

    AccumulatedObservation {
        timestamp: target_timestamp,
        price_sqrt_log_acc,
    }
}

/// Performs linear interpolation between two points.
///
/// # Arguments
/// * `x_left` - The x-coordinate of the left point.
/// * `x_right` - The x-coordinate of the right point.
/// * `y_left` - The y-coordinate (value) of the left point.
/// * `y_right` - The y-coordinate (value) of the right point.
/// * `x_target` - The x-coordinate of the target point for which the y-coordinate (value) is
/// to be interpolated.
///
/// # Returns
/// * `Decimal` - The interpolated y-coordinate (value) of the target point.
pub fn linear_interpolation(
    x_left: u64,
    x_right: u64,
    y_left: Decimal,
    y_right: Decimal,
    x_target: u64,
) -> Decimal {
    let slope = (y_right - y_left) / (x_right - x_left);
    y_left + slope * (x_target - x_left)
}

/// Calculates the arithmetic mean between two points.
///
/// # Arguments
/// * `x_left` - The x-coordinate of the left point.
/// * `x_right` - The x-coordinate of the right point.
/// * `y_left` - The y-coordinate (value) of the left point.
/// * `y_right` - The y-coordinate (value) of the right point.
///
/// # Returns
/// * `Decimal` - The arithmetic mean between the two points.
pub fn arithmetic_mean(x_left: u64, x_right: u64, y_left: Decimal, y_right: Decimal) -> Decimal {
    (y_right - y_left) / (x_right - x_left)
}

/// Calculates the geometric mean between two points.
///
/// # Arguments
/// * `x_left` - The x-coordinate of the left point.
/// * `x_right` - The x-coordinate of the right point.
/// * `y_left` - The y-coordinate (value) of the left point.
/// * `y_right` - The y-coordinate (value) of the right point.
///
/// # Returns
/// * `Decimal` - The geometric mean of the slope between the two points.
pub fn geometric_mean(x_left: u64, x_right: u64, y_left: Decimal, y_right: Decimal) -> Decimal {
    let exponent = arithmetic_mean(x_left, x_right, y_left, y_right);
    exponent.exp().unwrap()
}
