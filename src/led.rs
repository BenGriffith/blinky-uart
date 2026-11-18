pub enum LedState {
    On,
    Off,
}

impl LedState {
    pub fn as_bool(self) -> bool {
        matches!(self, LedState::On)
    }
}
