//! Project-wide constants. Centralizes magic numbers so they are not
//! duplicated across modules.

/// Waveform-style bufferbloat thresholds: (max latency increase in ms, grade).
/// The first row whose threshold the measured value is <= to wins.
/// `f64::INFINITY` serves as the catch-all for F.
pub const BUFFERBLOAT_THRESHOLDS: &[(f64, &str)] = &[
    (5.0, "A+"),
    (30.0, "A"),
    (60.0, "B"),
    (200.0, "C"),
    (400.0, "D"),
    (f64::INFINITY, "F"),
];

/// Stability thresholds: (max coefficient of variation in percent, grade).
/// First matching row wins. Calibrated against typical observed CV ranges
/// (wired fiber ~2-4%, decent Wi-Fi ~8-15%, congested ~20%+).
pub const STABILITY_THRESHOLDS: &[(f64, &str)] = &[
    (5.0, "A"),
    (10.0, "B"),
    (20.0, "C"),
    (35.0, "D"),
    (f64::INFINITY, "F"),
];

/// Sentinel grade used in `ConnectionQuality` when one half (bufferbloat
/// or stability) cannot be computed but the other can. Single-character
/// hyphen, never the em dash.
pub const GRADE_UNAVAILABLE: &str = "-";

/// Minimum number of throughput samples required to compute a CV
/// (stddev of fewer than 3 samples is meaningless).
pub const MIN_STABILITY_SAMPLES: usize = 3;
