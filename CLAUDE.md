# mics-6814

Platform-agnostic, `no_std` Rust driver for the MiCS-6814 triple gas sensor (CO, NO2, NH3).

## Project Overview

This is a standalone library crate (not a binary). It converts raw ADC readings from the MiCS-6814's three independent sensing elements into gas concentration estimates (ppm) using calibration curves from the datasheet.

**Sensor elements:**
- **RED** — reducing gases: CO (1-1000 ppm), Ethanol, H2, CH4, C3H8, C4H10
- **OX** — oxidising gases: NO2 (0.05-10 ppm)
- **NH3** — ammonia: NH3 (1-500 ppm)

## Architecture

embedded-hal 1.0 removed ADC traits, so this driver does NOT abstract over an ADC peripheral. Instead it accepts raw resistance ratios (Rs/R0) and converts them to ppm via log-log interpolation of the datasheet curves.

The user is responsible for:
1. Reading the ADC voltage for each channel
2. Computing Rs from the voltage divider: `Rs = Rload * (Vcc - Vadc) / Vadc`
3. Calibrating R0 in clean air
4. Passing `Rs/R0` to this driver

## Crate Structure

```
src/
  lib.rs           — MiCs6814 driver struct, public API
  calibration.rs   — Rs/R0 → ppm lookup tables and log-log interpolation
  error.rs         — driver error types
  gas.rs           — Gas enum and detection range metadata
```

## Conventions

- `no_std` — no allocator, no standard library. Use `libm` for math.
- Dual license: MIT OR Apache-2.0
- Use `defmt` behind a feature flag for embedded logging
- All public types must derive `Debug`, `Clone`, `Copy` where sensible
- Calibration data points are `const` arrays extracted from the datasheet curves
- Use `f32` throughout (embedded targets rarely have f64 hardware)
- Test with `#[cfg(test)]` standard unit tests (tests run on host, not target)

## Build & Test

```bash
cargo build                        # build library
cargo test                         # run unit tests
cargo build --features defmt-03    # build with defmt logging
cargo clippy -- -D warnings        # lint
cargo doc --no-deps --open         # generate docs
```

## Datasheet Reference

`docs/MiCS-6814.pdf` — SGX Sensortech datasheet (1143 rev 8). Key data:
- Page 1: Rs/R0 vs concentration curves for all three sensors
- Page 2: Sensor resistance ranges (R0) and sensitivity factors
- Page 3: Measurement circuit with load resistors, heater specs
