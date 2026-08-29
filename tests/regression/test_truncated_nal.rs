//! Regression tests for truncated NAL units and incomplete bitstream slices.

use crate::fixtures::build_truncated_slice_payload;
use valen_heic_core::Limits;
use valen_heic_decoder::{DecodeOptions, HeicDecoder, PureRustHevcDecoder};

#[test]
fn test_truncated_slice_payload_decode_error() {
    let truncated_slice = build_truncated_slice_payload();
    let decoder = PureRustHevcDecoder::new();
    let options = DecodeOptions {
        limits: Limits::default(),
        ..Default::default()
    };

    let result = decoder.decode_item(&truncated_slice, &options);
    assert!(result.is_err());
}

#[test]
fn test_empty_nal_payload_decode_error() {
    let empty: &[u8] = &[];
    let decoder = PureRustHevcDecoder::new();
    let options = DecodeOptions::default();

    let result = decoder.decode_item(empty, &options);
    assert!(result.is_err());
}
