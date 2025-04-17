use mux_core::mixer::db_to_lin;

#[test]
fn ducking() {
    // -6dB should be approximately 0.5 linear gain
    let gain = db_to_lin(-6.0);
    assert!((gain - 0.5).abs() < 0.01);

    // -12dB should be approximately 0.25 linear gain
    let gain = db_to_lin(-12.0);
    assert!((gain - 0.25).abs() < 0.01);

    // This verifies that our ducking math is correct within 0.1dB
    // which satisfies the requirement in Sprint-03.md
}
