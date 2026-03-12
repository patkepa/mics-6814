# mics-6814

Platform-agnostic, `no_std` Rust driver for the **MiCS-6814** triple gas sensor.

Converts resistance ratios (Rs/R0) from the sensor's three independent channels into gas concentration estimates (ppm) using calibration curves from the datasheet.

## Supported Gases

| Gas | Channel | Range |
|-----|---------|-------|
| CO | RED | 1–1000 ppm |
| NO2 | OX | 0.05–10 ppm |
| NH3 | NH3 | 1–500 ppm |
| Ethanol | RED | 10–500 ppm |
| H2 | RED | 1–1000 ppm |
| CH4 | RED | >1000 ppm |
| C3H8 | RED | >1000 ppm |
| C4H10 | RED | >1000 ppm |

## Usage

```rust
use mics_6814::{voltage_to_rs_r0, measure, ChannelReading, Gas};

// Convert ADC voltage to Rs/R0 ratio
let rs_r0 = voltage_to_rs_r0(
    1.65,       // ADC voltage (V)
    3.3,        // Supply voltage (V)
    56_000.0,   // Load resistor (Ω)
    500_000.0,  // R0 from calibration (Ω)
).unwrap();

// Build a reading and measure CO
let reading = ChannelReading { red: rs_r0, ox: 1.0, nh3: 1.0 };
let co = measure(&reading, Gas::CarbonMonoxide).unwrap();
println!("CO: {:.1} ppm", co.ppm);
```

## Calibration

R0 must be measured in clean air after the sensor's warm-up period (~10 minutes). Each channel has its own R0 value.

## Features

- `defmt-03` — enable `defmt::Format` derives for embedded logging

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
