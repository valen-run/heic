//! Probability context model definition and state initialization (Section 9.3.2.2).

/// A single CABAC probability context model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ContextModel {
    /// 6-bit probability state index (0..=63).
    pub state: u8,
    /// Most Probable Symbol (0 or 1).
    pub val_mps: u8,
}

impl ContextModel {
    /// Creates a context initialized with state and MPS.
    pub const fn new(state: u8, val_mps: u8) -> Self {
        Self { state, val_mps }
    }

    /// Initializes context according to ITU-T H.265 Section 9.3.2.2.
    pub fn init(&mut self, init_value: u8, slice_qp: i32) {
        let slope = (init_value as i32 >> 4) * 5 - 45;
        let offset = ((init_value as i32 & 15) << 3) - 16;
        let pre_state = ((slope * slice_qp) >> 4) + offset;
        let state_val = pre_state.clamp(1, 126);

        if state_val >= 64 {
            self.val_mps = 1;
            self.state = (state_val - 64) as u8;
        } else {
            self.val_mps = 0;
            self.state = (63 - state_val) as u8;
        }
    }
}
