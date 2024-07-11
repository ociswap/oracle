# Ociswap Price Oracle

The price oracle described in this document aims to provide to provide reliable and secure price feeds to various decentralized applications (DApps) running on the Radix DLT network. By utilizing this oracle, DApps can access accurate and up-to-date price information, which is critical for functions such as asset trading, collateral calculations, and financial derivatives.

This Price Oracle has been developed in a way to maximize both robustness and efficiency, providing reliable decentralized price information while minimizing network fees overheads  It is constructed in such a way to be resistant to potential vulnerabilities, such as Sybil attacks and price manipulations at very short time scales, thereby fostering the integrity of the price data it provides.

In the associated Rust code, a Rust Object implementation of the oracle, which we import as a module and integrated directly into our pools, in order to prevent network fees overheads associated with cross-component calls, thereby minimizing overall gas fees.



# Design

The oracle is triggered whenever a swap transaction is performed, and periodically stores an "observation", containing the **accumulated value of the logarithm of the square root of the price** (`price_sqrt_log_acc`), and the respective timestamp (corresponding to the beginning of the minute). This accumulation starts roughly with the start of the trading of the pool.

This allows the user to calculate the **geometric mean** of the square root of the price (`price_sqrt`)  of the pool between any two minute timestamps, provided that they are in the temporal range that is recorded in the oracle.

### General Characteristics:
- **Up ot 65535 stored observations**. Once this number is reached, the oldest observations start being overwritten. This gives a worst-case history range for the oracle of up to ~45.5 days into the past, given the scenario in which transactions take place every minute and, as such, one observation is stored per minute. Naturally, if there are idle minutes, then the oracle will be able to provide price values in time ranges of more than 45.5 days into the past.
- **Observations are stored on a per minute basis, assuming a swap transaction is made**. If no transaction takes place for several minutes, then only one observation will be stored once a transaction finally takes place. This observation will correspond to the beginning of the minute in which a transaction was finally triggered.
- **Transactions within the same minute are averaged**. As aforementioned, an observation is only store when a swap takes place in a different minute compared to the last swap. How are then same-minute swaps accounted for? The oracle performs a time-weighted average of the price of the pool within the said minute. This naturally means that, for same-second transactions, only the `price_sqrt` of the last transaction is considered.

### Mechanism

Each observation contains a field called `price_sqrt_log_acc`. This field represents the accumulation of the logarithms of the time-weighted average price for each minute, since the start of the trading of the pool until a given timestamp. It can be calculated with the following formula>

```
price_sqrt_log_acc(t) = price_sqrt_log_acc(t-a) + log(avg_price_sqrt(t-a)) + (a-1)*log(last_price_sqrt(t-a))
```

Where
- `t` is the unix timestamp in minutes, for which we wish to calculate the accumulation
- `a` is the number of minutes that have passed since the last observation was stored
- `price_sqrt_log_acc(t)` is the value accumulated till unix minute `t`, and included in the corresponding observation
- `price_sqrt_log_acc(t-a)` is the value accumulated till the last stored observation
- `log` represents the natural logarithm
- `avg_price_sqrt(t-a)` is the average price of the minute starting in `t-a`, i.e. the last active minute
- `last_price_sqrt(t)` is the last observed `price_sqrt` in the last active minute

A new observation is generated and stored for every new minute where a swap takes place. If no swap takes place in the 60s leading up to a given minute, then no observation is stored for that minute.

The oracle allows estimating `price_sqrt` for any subinterval (rounded to the minute) that is contained interval in the time interval kept in storage by the oracle. This is done via the following steps:

1) Binary search is used to find the two observations corresponding to the bounds of the interval.
    - For each of the bounds, if no corresponding observation can be found in the oracle, then these are interpolated from the two closest stored observations.
2) The geometric mean of the price square root is then calculated for the interval, using the formula:

```
price_sqrt_avg\[a,b\] = exp((y_b - y_a)/(b - x))
```


Where:
- `price_sqrt_avg[a,b]` is the average of the square root of the price in the interval `[a,b]`
- `y_a` and `y_b` are the `price_sqrt_log_acc` values of the observations at timestamps `a` and `b`
- `a` and `b` are the bounds of the interval being considered

# User interfaces
The Oracle module provides several interfaces to interact with and retrieve data from the price oracle. Below are the key interfaces and their explanations:

- `observation(seconds: u64) -> AccumulatedObservation`: This interface retrieves an `AccumulatedObservation` for a given timestamp in seconds. The timestamp is automatically rounded to the minute, and the respective observation (either directly or via interpolation) is returned, assuming the timestamp is in range.

- `observation_intervals(intervals_in_seconds: Vec<(u64, u64)>) -> Vec<ObservationInterval>`: This interface calculates the geometric mean of the price square root over specified intervals. It takes a vector of tuples, each representing the start and end of an interval in Unix seconds, and returns a vector of `ObservationInterval` structs. Each struct contains the start and end of the interval in Unix seconds and the calculated geometric mean of the price square root for the interval. Similarly to `observation`, the inserted timestamps are rounded to the minute.

- `observations_limit() -> u16`: This interface returns the limit of observations that can be stored within the oracle, setting its capacity. Once the limit is reached, the oldest observations are overwritted as needed.

- `observations_stored() -> u16`: This interface returns the number of observations currently stored in the oracle. Once the limit is reached, the oldest observations are overwritted as needed.

- `oldest_observation_at() -> Option<u64>`: This interface returns the timestamp of the oldest observation, if any. It is useful for determining the earliest point in time for which the oracle has data.



# Security considerations

The oracle is designed to be resistant to potential vulnerabilities, such as Sybil attacks and price manipulations at very short time scales.

The oracle mitigates the risk of price manipulation through Sybil attacks by employing a time-weighted average price (TWAP) rather than a simple average of prices after each swap. In a Sybil attack, an attacker could attempt to manipulate the price by executing a large number of swaps in a short period. However, since the oracle calculates the average price based on time rather than the number of transactions, the impact of such an attack is significantly reduced.

For instance, if an attacker were to execute 1000 swaps within a minute in an attempt to manipulate the price, the oracle's use of a time-weighted average means that the price would not be unduly influenced by the volume of transactions within that minute. Instead, the price calculation would reflect the price over the entire minute, making it much more difficult and costly for an attacker to have a significant impact on the price through Sybil attacks.

This approach ensures that the oracle remains robust against attempts to manipulate prices through a high volume of transactions in a short time frame, thereby preserving the integrity and reliability of the price feeds it provides.

However, it is important to note that it is not invulnerable, and that the safety depends also on the user. As such, there are good practice that the user can take in order to maximize the reliability of the oracle price feeds, listed below.


### Rules of thumb

1. The larger the interval within which the price is average, the better
    - Averaging across a longer time period means that the result becomes resilient to tampering, since disrupting the average would likely mean disrupting for price of the pool for greather lengths of time across that period, and consequently, the costs to the attacker. Disrupting the price for a given time length, mas the attacker not only likelly incur in slippage, but also leaves the attacker vulnerable to arbitrage.
2. The larger the liquidity the pool has, the better
    - Larger liquidity means it is more costly for the attacker to perform price disruptions, since the attacker will have to perform bigger swaps, incurring in slippage.
3. The greater the activity the pool has from a diverse number of participants, the better.
    - The greater the activity the pool has, the higher the degree to which the price of the assets in the pool are in sync with its value in other pools and markets. This is essential to get an accurate reading of the price. Simultaneously, it also exposes eventual attackers to arbitrage, greatly increasing the cost of the attack.