//! Reference sample container and smoothing filters (ITU-T H.265 Section 8.4.4.2.3).

/// Boundary and neighboring reference samples for an intra prediction block of size `N x N`.
#[derive(Debug, Clone)]
pub struct IntraReferences {
    /// Reference samples: index 0 is top-left `(-1, -1)`,
    /// indices `1..=2*N` are top and top-right `(0..2N-1, -1)`.
    pub top: Vec<u16>,
    /// Reference samples: index 0 is top-left `(-1, -1)`,
    /// indices `1..=2*N` are left and bottom-left `(-1, 0..2N-1)`.
    pub left: Vec<u16>,
    /// Block size `N` (e.g. 4, 8, 16, 32).
    pub size: usize,
}

impl IntraReferences {
    /// Creates a new reference sample container for block size `N` initialized with DC default.
    pub fn new(size: usize, default_val: u16) -> Self {
        Self {
            top: vec![default_val; 2 * size + 1],
            left: vec![default_val; 2 * size + 1],
            size,
        }
    }

    /// Prepares smoothed reference samples according to Section 8.4.4.2.3.
    pub fn filter_references(&self, mode: u8, is_luma: bool, bit_depth: u8) -> Self {
        let n = self.size;
        let mut filtered_top = self.top.clone();
        let mut filtered_left = self.left.clone();

        if !is_luma || n == 4 {
            return self.clone();
        }

        // Intra smoothing filter check
        let smoothing_needed = match n {
            8 => mode == 0 || mode == 2 || ((18..=34).contains(&mode) && ((mode - 18) & 3) == 0),
            16 => mode != 9 && mode != 10 && mode != 11 && mode != 25 && mode != 26 && mode != 27,
            32 => mode != 10 && mode != 26,
            _ => false,
        };

        if !smoothing_needed {
            return self.clone();
        }

        // Strong intra smoothing check for 32x32 blocks
        if n == 32 {
            let top_left = self.top[0] as i32;
            let top_end = self.top[2 * n] as i32;
            let left_end = self.left[2 * n] as i32;
            let threshold = 1 << (bit_depth - 5);

            let cond1 = (top_left - 2 * (self.top[n] as i32) + top_end).abs() < threshold;
            let cond2 = (top_left - 2 * (self.left[n] as i32) + left_end).abs() < threshold;

            if cond1 && cond2 {
                for i in 1..=2 * n {
                    filtered_top[i] =
                        (((2 * n - i) as i32 * top_left + i as i32 * top_end + 32) >> 6) as u16;
                    filtered_left[i] =
                        (((2 * n - i) as i32 * top_left + i as i32 * left_end + 32) >> 6) as u16;
                }
                return Self {
                    top: filtered_top,
                    left: filtered_left,
                    size: n,
                };
            }
        }

        // Standard 3-tap filter [1, 2, 1] / 4
        filtered_top[0] =
            ((self.left[1] as u32 + 2 * self.top[0] as u32 + self.top[1] as u32 + 2) >> 2) as u16;
        filtered_left[0] = filtered_top[0];

        for i in 1..2 * n {
            filtered_top[i] =
                ((self.top[i - 1] as u32 + 2 * self.top[i] as u32 + self.top[i + 1] as u32 + 2)
                    >> 2) as u16;
            filtered_left[i] =
                ((self.left[i - 1] as u32 + 2 * self.left[i] as u32 + self.left[i + 1] as u32 + 2)
                    >> 2) as u16;
        }

        Self {
            top: filtered_top,
            left: filtered_left,
            size: n,
        }
    }
}
