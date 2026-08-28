//! Color space identification and profiles.

/// Color space identification and profiles.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub enum ColorSpace {
    /// Standard sRGB.
    #[default]
    Srgb,
    /// Display P3 wide gamut.
    DisplayP3,
    /// Rec. 2020 wide gamut.
    Rec2020,
    /// ITU-R BT.601 color matrix.
    Bt601,
    /// ITU-R BT.709 color matrix.
    Bt709,
    /// Raw embedded ICC profile data.
    IccProfile(Vec<u8>),
}
