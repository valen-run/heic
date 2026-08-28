//! Comprehensive WebAssembly pipeline tests and interface validations.

use valen_heic_core::{HeicError, ImageDimensions, Limits, OutputFormat, PixelFormat};
use valen_heic_wasm::options::{WasmConvertOptions, WasmDecodeOptions};
use valen_heic_wasm::pipeline::{convert_image, decode_raw};
use valen_heic_wasm::{is_heif, wasm_detect};
use valen_image_encoder::{encode_jpeg, encode_png, encode_webp, PngOptions, WebpOptions};
use valen_image_processing::{flatten_alpha, PixelBuffer};

fn make_box(fourcc: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let size = (8 + payload.len()) as u32;
    let mut b = Vec::with_capacity(size as usize);
    b.extend_from_slice(&size.to_be_bytes());
    b.extend_from_slice(fourcc);
    b.extend_from_slice(payload);
    b
}

fn make_full_box(fourcc: &[u8; 4], version: u8, flags: u32, payload: &[u8]) -> Vec<u8> {
    let size = (12 + payload.len()) as u32;
    let mut b = Vec::with_capacity(size as usize);
    b.extend_from_slice(&size.to_be_bytes());
    b.extend_from_slice(fourcc);
    b.push(version);
    b.push(((flags >> 16) & 0xFF) as u8);
    b.push(((flags >> 8) & 0xFF) as u8);
    b.push((flags & 0xFF) as u8);
    b.extend_from_slice(payload);
    b
}

fn build_synthetic_heic() -> Vec<u8> {
    // 1. ftyp
    let mut ftyp_payload = Vec::new();
    ftyp_payload.extend_from_slice(b"heic");
    ftyp_payload.extend_from_slice(&[0, 0, 0, 0]);
    ftyp_payload.extend_from_slice(b"mif1");
    let ftyp = make_box(b"ftyp", &ftyp_payload);

    // 2. meta box components
    let mut hdlr_payload = Vec::new();
    hdlr_payload.extend_from_slice(&[0; 4]);
    hdlr_payload.extend_from_slice(b"pict");
    hdlr_payload.extend_from_slice(&[0; 12]);
    let hdlr = make_full_box(b"hdlr", 0, 0, &hdlr_payload);

    let pitm = make_full_box(b"pitm", 0, 0, &1u16.to_be_bytes());

    let infe = make_full_box(b"infe", 2, 0, &[0, 1, 0, 0, b'h', b'v', b'c', b'1', 0]);

    let mut iinf_payload = Vec::new();
    iinf_payload.extend_from_slice(&1u16.to_be_bytes());
    iinf_payload.extend_from_slice(&infe);
    let iinf = make_full_box(b"iinf", 0, 0, &iinf_payload);

    // ispe: 1920x1080
    let mut ispe_payload = Vec::new();
    ispe_payload.extend_from_slice(&1920u32.to_be_bytes());
    ispe_payload.extend_from_slice(&1080u32.to_be_bytes());
    let ispe = make_full_box(b"ispe", 0, 0, &ispe_payload);

    let ipco = make_box(b"ipco", &ispe);

    let mut ipma_payload = Vec::new();
    ipma_payload.extend_from_slice(&1u32.to_be_bytes());
    ipma_payload.extend_from_slice(&1u16.to_be_bytes());
    ipma_payload.push(1);
    ipma_payload.push(1);
    let ipma = make_full_box(b"ipma", 0, 0, &ipma_payload);

    let mut iprp_payload = Vec::new();
    iprp_payload.extend_from_slice(&ipco);
    iprp_payload.extend_from_slice(&ipma);
    let iprp = make_box(b"iprp", &iprp_payload);

    // iloc
    let mut iloc_payload = Vec::new();
    iloc_payload.push(0x44);
    iloc_payload.push(0x00);
    iloc_payload.extend_from_slice(&1u16.to_be_bytes());
    iloc_payload.extend_from_slice(&1u16.to_be_bytes());
    iloc_payload.extend_from_slice(&0u16.to_be_bytes());
    iloc_payload.extend_from_slice(&1u16.to_be_bytes());
    let offset_placeholder_pos = iloc_payload.len();
    iloc_payload.extend_from_slice(&0u32.to_be_bytes());
    iloc_payload.extend_from_slice(&12u32.to_be_bytes());

    let mut meta_content = Vec::new();
    meta_content.extend_from_slice(&hdlr);
    meta_content.extend_from_slice(&pitm);
    meta_content.extend_from_slice(&iinf);
    meta_content.extend_from_slice(&iprp);

    let meta_total_size = 12 + meta_content.len() + 12 + iloc_payload.len();
    let mdat_payload_offset = ftyp.len() + meta_total_size + 8;

    iloc_payload[offset_placeholder_pos..offset_placeholder_pos + 4]
        .copy_from_slice(&(mdat_payload_offset as u32).to_be_bytes());
    let iloc = make_full_box(b"iloc", 0, 0, &iloc_payload);

    meta_content.extend_from_slice(&iloc);
    let meta = make_full_box(b"meta", 0, 0, &meta_content);

    // 3. mdat
    let mut mdat_payload = Vec::new();
    mdat_payload.extend_from_slice(&8u32.to_be_bytes());
    mdat_payload.extend_from_slice(&[0x26, 0x01, 0xAF, 0x00, 0x00, 0x00, 0x00, 0x00]);
    let mdat = make_box(b"mdat", &mdat_payload);

    let mut file_bytes = Vec::new();
    file_bytes.extend_from_slice(&ftyp);
    file_bytes.extend_from_slice(&meta);
    file_bytes.extend_from_slice(&mdat);
    file_bytes
}

#[test]
fn test_is_heif_fast_brand_detection() {
    let valid_data = build_synthetic_heic();
    assert!(is_heif(&valid_data));
    assert!(wasm_detect(&valid_data));

    assert!(!is_heif(&[]));
    assert!(!is_heif(b"GIF89a123456"));
    assert!(!is_heif(&[0, 0, 0, 8, b'f', b't', b'y', b'p']));
}

#[test]
fn test_limits_rejection_in_wasm_pipeline() {
    let data = build_synthetic_heic();
    let convert_opts = WasmConvertOptions {
        format: OutputFormat::Jpeg,
        quality: 85,
        bg_color: [255, 255, 255],
        apply_orientation: true,
        limits: Limits::none().with_max_file_size(10), // Too small
    };

    let res = convert_image(&data, &convert_opts);
    assert!(res.is_err());

    let decode_opts = WasmDecodeOptions {
        pixel_format: PixelFormat::Rgba8,
        apply_orientation: true,
        limits: Limits::none().with_max_file_size(10),
    };
    let res_dec = decode_raw(&data, &decode_opts);
    assert!(res_dec.is_err());
}

#[test]
fn test_pixel_buffer_pipeline_encoders() {
    let mut buf = PixelBuffer::new(ImageDimensions::new(16, 16), PixelFormat::Rgba8);
    for y in 0..16 {
        for x in 0..16 {
            buf.set_pixel(x, y, &[(x * 16) as u8, (y * 16) as u8, 128, 200])
                .unwrap();
        }
    }

    // 1. JPEG with alpha flattening
    let limits = Limits::default();
    let rgb_buf = flatten_alpha(&buf, [255, 255, 255], &limits).unwrap();
    assert_eq!(rgb_buf.format, PixelFormat::Rgb8);
    let jpeg = encode_jpeg(&rgb_buf, 85).unwrap();
    assert!(jpeg.starts_with(&[0xFF, 0xD8]));
    assert!(jpeg.ends_with(&[0xFF, 0xD9]));

    // 2. PNG
    let png = encode_png(&buf, &PngOptions::default()).unwrap();
    assert!(png.starts_with(&[0x89, 0x50, 0x4E, 0x47]));

    // 3. WebP
    let webp = encode_webp(&buf, &WebpOptions::default()).unwrap();
    assert!(webp.starts_with(b"RIFF"));
    assert_eq!(&webp[8..12], b"WEBP");
}

#[test]
fn test_error_taxonomy_codes() {
    assert_eq!(
        HeicError::InvalidInput("test".into()).error_code(),
        "INVALID_INPUT"
    );
    assert_eq!(
        HeicError::InvalidContainer("test".into()).error_code(),
        "INVALID_CONTAINER"
    );
    assert_eq!(
        HeicError::UnsupportedBrand("test".into()).error_code(),
        "UNSUPPORTED_BRAND"
    );
    assert_eq!(
        HeicError::UnsupportedFormat("test".into()).error_code(),
        "UNSUPPORTED_FORMAT"
    );
    assert_eq!(
        HeicError::UnsupportedCodec("test".into()).error_code(),
        "UNSUPPORTED_CODEC"
    );
    assert_eq!(
        HeicError::LimitInputBytes {
            actual: 100,
            max: 50
        }
        .error_code(),
        "LIMIT_INPUT_BYTES"
    );
    assert_eq!(
        HeicError::LimitDimensions {
            width: 100,
            height: 100,
            max_width: Some(50),
            max_height: Some(50),
        }
        .error_code(),
        "LIMIT_DIMENSIONS"
    );
    assert_eq!(
        HeicError::LimitPixels {
            count: 100,
            max: 50
        }
        .error_code(),
        "LIMIT_PIXELS"
    );
    assert_eq!(
        HeicError::LimitMemory {
            requested: 100,
            max: 50
        }
        .error_code(),
        "LIMIT_MEMORY"
    );
    assert_eq!(
        HeicError::MalformedInput("test".into()).error_code(),
        "MALFORMED_INPUT"
    );
    assert_eq!(
        HeicError::DecodeFailed("test".into()).error_code(),
        "DECODE_FAILED"
    );
    assert_eq!(
        HeicError::EncodeFailed("test".into()).error_code(),
        "ENCODE_FAILED"
    );
}
