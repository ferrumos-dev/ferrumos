use super::TickSnapshot;

#[test]
fn rejects_undefined_clock_parameters() {
    assert_eq!(TickSnapshot::new(1, 0, 100), None);
    assert_eq!(TickSnapshot::new(1, 100, 0), None);
}

#[test]
fn preserves_rational_frequency_and_rounds_down_fractional_milliseconds() {
    let snapshot = TickSnapshot::new(100, 1_193_182, 11_932).unwrap();
    assert_eq!(snapshot.ticks(), 100);
    assert_eq!(snapshot.frequency_numerator_hz(), 1_193_182);
    assert_eq!(snapshot.frequency_divisor(), 11_932);
    assert_eq!(snapshot.uptime_millis(), 1000);
    assert_eq!(snapshot.uptime_seconds(), 1);
    let fraction = TickSnapshot::new(1, 3, 1).unwrap();
    assert_eq!(fraction.uptime_millis(), 333);
    assert_eq!(fraction.uptime_seconds(), 0);
}

#[test]
fn zero_ticks_and_extreme_durations_never_wrap() {
    let zero = TickSnapshot::new(0, 1_193_182, 11_932).unwrap();
    assert_eq!(zero.uptime_millis(), 0);
    assert_eq!(zero.uptime_seconds(), 0);
    let saturated = TickSnapshot::new(u64::MAX, 1, u16::MAX).unwrap();
    assert_eq!(saturated.uptime_millis(), u64::MAX);
    assert_eq!(saturated.uptime_seconds(), u64::MAX);
    let seconds = TickSnapshot::new(u64::MAX, 1000, 1).unwrap();
    assert_eq!(seconds.uptime_millis(), u64::MAX);
    assert_eq!(seconds.uptime_seconds(), u64::MAX / 1000);
}
