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
benchmark-risc0
├── methods-*
│
data-gathering
├── reserve-price-original
├── reserve-price-modified
│
main
├── host
├── methods-*
├── db-*
```

`common` - consist of commonly shared rust functions in floating point and fixed point （to run `test_compare_calculate_twap` we will need to have `data.csv` in the `common` directory)

`benchmark-risc0` - consist of guest programs to run various sub functions
(to run `twap_error_bound_floating` we will need to have `data.csv` in the `twap_error_bound_floating` directory)

`data-gathering` - contains code to gather data from the original implementation and the modified implementation

`main` - contains the original unoptimzed implementation as well as optimized twap and volatility calculation
