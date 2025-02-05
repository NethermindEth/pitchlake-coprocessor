# Pitchlake coprocessor
Contains all the necessary functions,benchmarks, guest progra to run the pitchlake zkvm process

## File structure
```
benchmark
├── src
│   ├── fixed_point
│   ├── floating_point
│   ├── test
│
benchmark-risc0
├── methods-*
│
main
├── host
├── methods-*
├── db-*
```

`benchmark` - consist of commonly shared rust functions in floating point and fixed point
`benchmark-risc0` - consist of guest programs to run various sub functions
`main` - contains the original unoptimzed implementation as well as optimized twap and volatility calculation
