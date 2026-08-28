//! Colour profile and auxiliary channel property types.

use valen_heic_core::ColorSpace;

/// Colour information parsed from `colr`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorProperty {
    /// NCLX colour profile description.
    Nclx {
        /// Colour primaries index.
        colour_primaries: u16,
        /// Transfer characteristics index.
        transfer_characteristics: u16,
        /// Matrix coefficients index.
        matrix_coefficients: u16,
        /// Full range video flag.
        full_range_flag: bool,
    },
    /// Embedded raw ICC profile data (`rICC` or `prof`).
    IccProfile(Vec<u8>),
}

impl ColorProperty {
    /// Converts to core [`ColorSpace`].
    pub fn to_color_space(&self) -> ColorSpace {
        match self {
            Self::Nclx {
                colour_primaries, ..
            } => match colour_primaries {
                1 => ColorSpace::Srgb,
                12 => ColorSpace::DisplayP3,
                9 => ColorSpace::Rec2020,
                _ => ColorSpace::Srgb,
            },
            Self::IccProfile(bytes) => ColorSpace::IccProfile(bytes.clone()),
        }
    }
}

/// Auxiliary image type parsed from `auxC`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuxiliaryProperty {
    /// URN identifier of auxiliary stream (e.g. `urn:mpeg:hevc:2015:auxid:1` for alpha channel).
    pub aux_type: String,
    /// Sub-type configuration data.
    pub sub_type: Vec<u8>,
}

impl AuxiliaryProperty {
    /// Returns `true` if this auxiliary property identifies an alpha transparency mask.
    pub fn is_alpha(&self) -> bool {
        self.aux_type == "urn:mpeg:hevc:2015:auxid:1"
            || self.aux_type.ends_with(":auxid:1")
            || self.aux_type.eq_ignore_ascii_case("alpha")
    }
}
