# Pitchlake coprocessor
Contains all the necessary functions,benchmarks, guest programs to run the pitchlake zkvm process

## File structure
```
common
├── src
│   ├── fixed_point
│   ├── floating_point
│   ├── test
│
risc0-methods
├── methods-*
```

`common` - consist of commonly shared rust functions in floating point and fixed point （to run `test_compare_calculate_twap` we will need to have `data.csv` in the `common` directory)

`risc0-methods` - consist of guest programs to run various sub functions
(to run `twap_error_bound_floating` we will need to have `data.csv` in the `twap_error_bound_floating` directory)

## Running each methods
To run each methods, we will need to `cd` into the methods that we are interested in running (eg. risc0-methods/proof-composition-twap-maxreturn-reserveprice-floating-hashing)

with command

```
RISC0_DEV_MODE=1 cargo run
```


or from `risc0-methods` folder

```
RISC0_DEV_MODE=1 cargo run -p <proof-composition-twap-maxreturn-reserveprice-floating-hashing>
```

## Journal
The output of the journal will look like:
```
pub struct ProofCompositionOutput {
    pub data_8_months_hash: [u32; 8],
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub reserve_price: f64,
    pub floating_point_tolerance: f64,
    pub reserve_price_tolerance: f64,
    pub twap_tolerance: f64,
    pub twap_result: f64,
    pub max_return: f64,
}
```