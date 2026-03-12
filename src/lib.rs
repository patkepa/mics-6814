#![no_std]
#![doc = include_str!("../README.md")]

pub mod calibration;
pub mod error;
pub mod gas;

pub use calibration::rs_r0_to_ppm;
pub use error::Error;
pub use gas::{Channel, Gas};

use calibration::rs_r0_to_ppm as convert;

/// Raw resistance readings from the three sensor channels.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct ChannelReading {
    /// Rs/R0 ratio from the RED (reducing) sensor channel.
    pub red: f32,
    /// Rs/R0 ratio from the OX (oxidising) sensor channel.
    pub ox: f32,
    /// Rs/R0 ratio from the NH3 sensor channel.
    pub nh3: f32,
}

/// Gas concentration reading in parts per million.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct GasReading {
    /// The gas that was measured.
    pub gas: Gas,
    /// Estimated concentration in ppm.
    pub ppm: f32,
}

/// Convert an ADC voltage reading to an Rs/R0 ratio.
///
/// Assumes the datasheet reference circuit (page 3): load resistor on the high
/// side (VCC → Rload → ADC pin → Rs → GND), giving:
///
/// `Rs = Rload * Vadc / (Vcc - Vadc)`
///
/// # Arguments
/// - `v_adc` — measured voltage at the ADC pin (volts)
/// - `v_cc` — supply voltage (volts), typically 3.3 or 5.0
/// - `r_load` — load resistor value in ohms (minimum 820 Ω per datasheet)
/// - `r0` — baseline resistance in clean air (ohms), obtained during calibration
pub fn voltage_to_rs_r0(v_adc: f32, v_cc: f32, r_load: f32, r0: f32) -> Result<f32, Error> {
    if v_adc <= 0.0 || v_adc >= v_cc {
        return Err(Error::InvalidVoltage);
    }
    if r_load <= 0.0 || r0 <= 0.0 {
        return Err(Error::InvalidLoadResistance);
    }
    let rs = r_load * v_adc / (v_cc - v_adc);
    Ok(rs / r0)
}

/// Estimate the concentration of a specific gas from a channel reading.
///
/// The `reading` must contain the Rs/R0 ratio for the correct channel
/// (determined by `gas.channel()`).
pub fn measure(reading: &ChannelReading, gas: Gas) -> Result<GasReading, Error> {
    let rs_r0 = match gas.channel() {
        Channel::Red => reading.red,
        Channel::Ox => reading.ox,
        Channel::Nh3 => reading.nh3,
    };
    let ppm = convert(gas, rs_r0)?;
    Ok(GasReading { gas, ppm })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voltage_to_rs_r0() {
        // Midpoint voltage → Rs = Rload → ratio = Rload/R0 = 1.0
        let ratio = voltage_to_rs_r0(1.65, 3.3, 10_000.0, 10_000.0).unwrap();
        assert!((ratio - 1.0).abs() < 0.01, "expected 1.0, got {ratio}");
    }

    #[test]
    fn test_voltage_to_rs_r0_zero_voltage() {
        let result = voltage_to_rs_r0(0.0, 3.3, 10_000.0, 10_000.0);
        assert_eq!(result, Err(Error::InvalidVoltage));
    }

    #[test]
    fn test_voltage_equals_vcc() {
        // Vadc = Vcc means denominator is zero — reject
        let result = voltage_to_rs_r0(3.3, 3.3, 10_000.0, 10_000.0);
        assert_eq!(result, Err(Error::InvalidVoltage));
    }

    #[test]
    fn test_voltage_exceeds_vcc() {
        let result = voltage_to_rs_r0(3.5, 3.3, 10_000.0, 10_000.0);
        assert_eq!(result, Err(Error::InvalidVoltage));
    }

    #[test]
    fn test_full_pipeline_co() {
        // Datasheet circuit: VCC → Rload → ADC → Rs → GND
        // Vadc = Vcc * Rs / (Rload + Rs)
        // Want Rs/R0 = 0.25 with R0=500kΩ → Rs = 125kΩ
        // Rload = 56kΩ, Vcc = 3.3V
        // Vadc = 3.3 * 125000 / (56000 + 125000) = 2.279V
        let rs_target = 0.25 * 500_000.0; // 125kΩ
        let v_adc = 3.3 * rs_target / (56_000.0 + rs_target);
        let ratio = voltage_to_rs_r0(v_adc, 3.3, 56_000.0, 500_000.0).unwrap();
        let reading = ChannelReading {
            red: ratio,
            ox: 1.0,
            nh3: 1.0,
        };
        let result = measure(&reading, Gas::CarbonMonoxide).unwrap();
        assert!(
            (result.ppm - 100.0).abs() < 20.0,
            "expected ~100 ppm CO, got {}",
            result.ppm
        );
    }
}
