/// Errors that can occur when using the MiCS-6814 driver.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum Error {
    /// The Rs/R0 ratio is out of the valid range for the requested gas.
    OutOfRange,
    /// The ADC voltage is invalid (negative, or exceeds Vcc).
    InvalidVoltage,
    /// The load resistance is zero or negative.
    InvalidLoadResistance,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::OutOfRange => write!(f, "Rs/R0 ratio out of sensor range"),
            Error::InvalidVoltage => write!(f, "ADC voltage invalid"),
            Error::InvalidLoadResistance => write!(f, "load resistance must be positive"),
        }
    }
}
