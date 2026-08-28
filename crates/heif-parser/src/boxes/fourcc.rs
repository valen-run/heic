//! Four-character box type identifier (`FourCC`).

use std::fmt;

/// Four-character box type identifier (FourCC).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FourCC(pub [u8; 4]);

impl FourCC {
    /// Creates a FourCC from a 4-byte slice.
    #[inline]
    pub const fn from_bytes(bytes: [u8; 4]) -> Self {
        Self(bytes)
    }

    /// Returns the underlying 4-byte array reference.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; 4] {
        &self.0
    }

    // Top-level containers & metadata boxes
    /// File type box `ftyp`.
    pub const FTYP: Self = Self(*b"ftyp");
    /// Meta box `meta`.
    pub const META: Self = Self(*b"meta");
    /// Handler reference box `hdlr`.
    pub const HDLR: Self = Self(*b"hdlr");
    /// Primary item box `pitm`.
    pub const PITM: Self = Self(*b"pitm");
    /// Item location box `iloc`.
    pub const ILOC: Self = Self(*b"iloc");
    /// Item information box `iinf`.
    pub const IINF: Self = Self(*b"iinf");
    /// Item info entry box `infe`.
    pub const INFE: Self = Self(*b"infe");
    /// Item reference box `iref`.
    pub const IREF: Self = Self(*b"iref");
    /// Media data box `mdat`.
    pub const MDAT: Self = Self(*b"mdat");
    /// Item data box `idat`.
    pub const IDAT: Self = Self(*b"idat");

    // Item properties
    /// Item properties box `iprp`.
    pub const IPRP: Self = Self(*b"iprp");
    /// Item property container box `ipco`.
    pub const IPCO: Self = Self(*b"ipco");
    /// Item property association box `ipma`.
    pub const IPMA: Self = Self(*b"ipma");
    /// Image spatial extents box `ispe`.
    pub const ISPE: Self = Self(*b"ispe");
    /// Image rotation box `irot`.
    pub const IROT: Self = Self(*b"irot");
    /// Image mirror box `imir`.
    pub const IMIR: Self = Self(*b"imir");
    /// Colour information box `colr`.
    pub const COLR: Self = Self(*b"colr");
    /// Auxiliary item property `auxC`.
    pub const AUXC: Self = Self(*b"auxC");
    /// HEVC configuration box `hvcC`.
    pub const HVCC: Self = Self(*b"hvcC");
    /// Pixel aspect ratio box `pasp`.
    pub const PASP: Self = Self(*b"pasp");
    /// Clean aperture box `clap`.
    pub const CLAP: Self = Self(*b"clap");

    // Handlers & Item Types
    /// Picture handler type `pict`.
    pub const PICT: Self = Self(*b"pict");
    /// HEVC/H.265 intra item type `hvc1`.
    pub const HVC1: Self = Self(*b"hvc1");
    /// Image grid derived item type `grid`.
    pub const GRID: Self = Self(*b"grid");
    /// EXIF metadata item type `Exif`.
    pub const EXIF: Self = Self(*b"Exif");
    /// MIME metadata item type `mime`.
    pub const MIME: Self = Self(*b"mime");

    // Reference Types
    /// Derived image reference `dimg`.
    pub const DIMG: Self = Self(*b"dimg");
    /// Auxiliary image reference `auxl`.
    pub const AUXL: Self = Self(*b"auxl");
    /// Content description reference `cdsc`.
    pub const CDSC: Self = Self(*b"cdsc");
    /// Thumbnail reference `thmb`.
    pub const THMB: Self = Self(*b"thmb");
}

impl fmt::Display for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = String::from_utf8_lossy(&self.0);
        write!(f, "{s}")
    }
}
