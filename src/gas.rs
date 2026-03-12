/// The three independent sensing channels on the MiCS-6814.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum Channel {
    /// Reducing gas sensor — CO, Ethanol, H2, CH4, C3H8, C4H10, NH3
    Red,
    /// Oxidising gas sensor — NO2, NO, H2
    Ox,
    /// NH3 sensor — NH3, Ethanol, H2, C3H8, C4H10
    Nh3,
}

/// Gases detectable by the MiCS-6814.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum Gas {
    /// Carbon monoxide (1–1000 ppm), RED channel
    CarbonMonoxide,
    /// Nitrogen dioxide (0.05–10 ppm), OX channel
    NitrogenDioxide,
    /// Ammonia (1–500 ppm), NH3 channel
    Ammonia,
    /// Ethanol (10–500 ppm), RED channel
    Ethanol,
    /// Hydrogen (1–1000 ppm), RED channel
    Hydrogen,
    /// Methane (>1000 ppm), RED channel
    Methane,
    /// Propane (>1000 ppm), RED channel
    Propane,
    /// Isobutane (>1000 ppm), RED channel
    Isobutane,
}

impl Gas {
    /// Returns which sensor channel this gas is measured on.
    pub fn channel(self) -> Channel {
        match self {
            Gas::CarbonMonoxide => Channel::Red,
            Gas::NitrogenDioxide => Channel::Ox,
            Gas::Ammonia => Channel::Nh3,
            Gas::Ethanol => Channel::Red,
            Gas::Hydrogen => Channel::Red,
            Gas::Methane => Channel::Red,
            Gas::Propane => Channel::Red,
            Gas::Isobutane => Channel::Red,
        }
    }
}
