//! NAL unit parser, RBSP bit-level reader, parameter sets (VPS/SPS/PPS), and slice headers.

pub mod bit_reader;
pub mod pps;
pub mod slice;
pub mod sps;
pub mod unit;

pub use bit_reader::BitReader;
pub use pps::Pps;
pub use slice::{SliceHeader, SliceType};
pub use sps::Sps;
pub use unit::{remove_emulation_prevention_bytes, NalUnit, NalUnitType};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emulation_prevention_byte_removal() {
        let raw = [0x00, 0x00, 0x03, 0x01, 0x00, 0x00, 0x03, 0x02];
        let rbsp = remove_emulation_prevention_bytes(&raw);
        assert_eq!(rbsp, vec![0x00, 0x00, 0x01, 0x00, 0x00, 0x02]);
    }

    #[test]
    fn test_bit_reader_exp_golomb() {
        // ue(0) = 1 (1 bit)
        // ue(1) = 010 (3 bits)
        // ue(2) = 011 (3 bits) -> se(2) = -1
        let mut r = BitReader::new(&[0b10100110]);
        assert_eq!(r.read_ue().unwrap(), 0);
        assert_eq!(r.read_ue().unwrap(), 1);
        assert_eq!(r.read_se().unwrap(), -1);
    }
}
