//mod player_script_tests;

#[allow(dead_code)]
pub struct StubClock {
    fixed_delta: f32,
}
impl ace::Clock for StubClock {
    fn time_delta(&self) -> f32 {
        self.fixed_delta
    }

    fn stop_frame_time(&self) {
        // Noop ; Stub clock time delta is fixed
    }
}
