use crate::error::Error;
use crate::gas::Gas;
use libm::{log10f, powf};

/// Linear interpolation on a sorted slice of (x, y) pairs.
///
/// Inputs must already be in the desired space (e.g. log10-transformed).
/// The function performs plain linear interpolation — it does NOT apply
/// any log transformation internally.
///
/// Returns the interpolated y value, or `Error::OutOfRange` if `x` falls
/// outside the data bounds.
pub(crate) fn interpolate_sorted(points: &[(f32, f32)], x: f32) -> Result<f32, Error> {
    if points.len() < 2 {
        return Err(Error::OutOfRange);
    }

    let first = points[0].0;
    let last = points[points.len() - 1].0;

    if x < first - 0.001 || x > last + 0.001 {
        return Err(Error::OutOfRange);
    }

    // Clamp to endpoints
    if x <= first {
        return Ok(points[0].1);
    }
    if x >= last {
        return Ok(points[points.len() - 1].1);
    }

    // Find the bracketing segment
    let mut i = 0;
    while i < points.len() - 1 {
        if x >= points[i].0 && x <= points[i + 1].0 {
            let (x0, y0) = points[i];
            let (x1, y1) = points[i + 1];
            let t = (x - x0) / (x1 - x0);
            return Ok(y0 + t * (y1 - y0));
        }
        i += 1;
    }

    Err(Error::OutOfRange)
}

// ── Calibration curve data ──────────────────────────────────────────
// Each array stores (log10_ppm, log10_rs_r0) pairs, sorted by log10_ppm ascending.
// Values read from the datasheet graphs (page 1), 25°C, 50% RH.

// RED sensor
const CO_RED: &[(f32, f32)] = &[
    (0.0, 0.0),       // 1 ppm,   Rs/R0 = 1.0
    (1.0, -0.222),    // 10 ppm,  Rs/R0 = 0.6
    (2.0, -0.602),    // 100 ppm, Rs/R0 = 0.25
    (3.0, -1.046),    // 1000 ppm, Rs/R0 = 0.09
];

const ETHANOL_RED: &[(f32, f32)] = &[
    (1.0, -0.155),    // 10 ppm,  Rs/R0 = 0.7
    (2.0, -0.553),    // 100 ppm, Rs/R0 = 0.28
    (2.699, -0.854),  // 500 ppm, Rs/R0 = 0.14
];

const H2_RED: &[(f32, f32)] = &[
    (0.0, -0.097),    // 1 ppm,   Rs/R0 = 0.8
    (1.0, -0.398),    // 10 ppm,  Rs/R0 = 0.4
    (2.0, -0.824),    // 100 ppm, Rs/R0 = 0.15
    (3.0, -1.398),    // 1000 ppm, Rs/R0 = 0.04
];

const CH4_RED: &[(f32, f32)] = &[
    (3.0, 0.0),       // 1000 ppm,  Rs/R0 = 1.0
    (4.0, -0.155),    // 10000 ppm, Rs/R0 = 0.7
    (5.0, -0.398),    // 100000 ppm, Rs/R0 = 0.4
];

const C3H8_RED: &[(f32, f32)] = &[
    (3.0, -0.222),    // 1000 ppm,  Rs/R0 = 0.6
    (4.0, -0.602),    // 10000 ppm, Rs/R0 = 0.25
    (5.0, -1.046),    // 100000 ppm, Rs/R0 = 0.09
];

const C4H10_RED: &[(f32, f32)] = &[
    (3.0, -0.301),    // 1000 ppm,  Rs/R0 = 0.5
    (4.0, -0.824),    // 10000 ppm, Rs/R0 = 0.15
    (5.0, -1.398),    // 100000 ppm, Rs/R0 = 0.04
];

// OX sensor
const NO2_OX: &[(f32, f32)] = &[
    (-1.301, -0.046), // 0.05 ppm, Rs/R0 = 0.9
    (-1.0, 0.176),    // 0.1 ppm,  Rs/R0 = 1.5
    (0.0, 0.903),     // 1 ppm,    Rs/R0 = 8.0
    (1.0, 1.845),     // 10 ppm,   Rs/R0 = 70.0
];

// NH3 sensor
const NH3_NH3: &[(f32, f32)] = &[
    (0.0, -0.155),    // 1 ppm,   Rs/R0 = 0.7
    (1.0, -0.456),    // 10 ppm,  Rs/R0 = 0.35
    (2.0, -1.046),    // 100 ppm, Rs/R0 = 0.09
    (2.699, -1.523),  // 500 ppm, Rs/R0 = 0.03
];

/// Returns the calibration curve data for the given gas.
///
/// The returned slice contains `(log10_ppm, log10_rs_r0)` pairs.
pub(crate) fn curve_for(gas: Gas) -> &'static [(f32, f32)] {
    match gas {
        Gas::CarbonMonoxide => CO_RED,
        Gas::NitrogenDioxide => NO2_OX,
        Gas::Ammonia => NH3_NH3,
        Gas::Ethanol => ETHANOL_RED,
        Gas::Hydrogen => H2_RED,
        Gas::Methane => CH4_RED,
        Gas::Propane => C3H8_RED,
        Gas::Isobutane => C4H10_RED,
    }
}

/// Convert an Rs/R0 ratio to a concentration in ppm for the given gas.
///
/// Uses linear interpolation of the datasheet calibration curves in log10-log10
/// space. Returns `Error::OutOfRange` if the ratio is outside the curve bounds.
///
/// **Boundary behavior:** If the Rs/R0 ratio is slightly outside the
/// calibration curve (e.g. due to measurement noise or clean air readings
/// near the baseline), the function returns `OutOfRange`. Callers should
/// handle this — a common pattern is to clamp to 0 ppm when the ratio
/// indicates cleaner-than-calibration air.
pub fn rs_r0_to_ppm(gas: Gas, rs_r0: f32) -> Result<f32, Error> {
    if rs_r0 <= 0.0 {
        return Err(Error::OutOfRange);
    }

    let log_ratio = log10f(rs_r0);
    let curve = curve_for(gas);

    // The curves are stored as (log10_ppm, log10_rs_r0).
    // We need to look up log10_ppm given log10_rs_r0.
    // Build an inverted view: (log10_rs_r0, log10_ppm), sorted by log10_rs_r0.
    // For RED/NH3 sensors, Rs/R0 decreases with concentration (monotonically decreasing).
    // For OX sensor (NO2), Rs/R0 increases with concentration (monotonically increasing).

    let is_increasing = gas == Gas::NitrogenDioxide;

    // Build sorted-by-rs_r0 pairs
    // We need to sort by the y-value (log10_rs_r0) to use as x for inverse lookup.
    // Since the curves are monotonic, we can reverse or keep order.
    let mut inverted: [Option<(f32, f32)>; 8] = [None; 8];
    let len = curve.len();
    for i in 0..len {
        if is_increasing {
            // rs_r0 increases with ppm — already sorted ascending by log_rs
            inverted[i] = Some((curve[i].1, curve[i].0));
        } else {
            // rs_r0 decreases with ppm — reverse to get ascending log_rs
            inverted[i] = Some((curve[len - 1 - i].1, curve[len - 1 - i].0));
        }
    }

    // Collect into a fixed-size buffer for interpolation
    let mut buf: [(f32, f32); 8] = [(0.0, 0.0); 8];
    for i in 0..len {
        buf[i] = inverted[i].unwrap();
    }

    interpolate_sorted(&buf[..len], log_ratio).map(|log_ppm| powf(10.0, log_ppm))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_midpoint() {
        // Two points: (1.0, 0.0) and (3.0, 2.0) in log-log space
        // At x=2.0, linear interpolation gives y=1.0
        let points: &[(f32, f32)] = &[(1.0, 0.0), (3.0, 2.0)];
        let result = interpolate_sorted(points, 2.0);
        assert!((result.unwrap() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_interpolate_exact_point() {
        let points: &[(f32, f32)] = &[(1.0, 0.0), (3.0, 2.0)];
        let result = interpolate_sorted(points, 1.0);
        assert!((result.unwrap() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_interpolate_out_of_range() {
        let points: &[(f32, f32)] = &[(1.0, 0.0), (3.0, 2.0)];
        let result = interpolate_sorted(points, 0.5);
        assert_eq!(result, Err(crate::Error::OutOfRange));
    }

    #[test]
    fn test_co_at_known_point() {
        // At Rs/R0 = 0.25, datasheet says ~100 ppm CO
        let ppm = rs_r0_to_ppm(Gas::CarbonMonoxide, 0.25).unwrap();
        assert!((ppm - 100.0).abs() < 15.0, "expected ~100, got {ppm}");
    }

    #[test]
    fn test_no2_at_known_point() {
        // At Rs/R0 = 8.0, datasheet says ~1 ppm NO2
        let ppm = rs_r0_to_ppm(Gas::NitrogenDioxide, 8.0).unwrap();
        assert!((ppm - 1.0).abs() < 0.3, "expected ~1.0, got {ppm}");
    }

    #[test]
    fn test_nh3_at_known_point() {
        // At Rs/R0 = 0.35, datasheet says ~10 ppm NH3
        let ppm = rs_r0_to_ppm(Gas::Ammonia, 0.35).unwrap();
        assert!((ppm - 10.0).abs() < 3.0, "expected ~10, got {ppm}");
    }

    #[test]
    fn test_out_of_range_ratio() {
        // Rs/R0 = 0.0 should be an error
        assert_eq!(
            rs_r0_to_ppm(Gas::CarbonMonoxide, 0.0),
            Err(crate::Error::OutOfRange)
        );
    }
}
