// Cause-chain tests for `PlayerError`. This file is a child of error.rs.

// Bring parent module declarations into the test module.
use super::*;
use std::error::Error as _;

/// A wrapped I/O error is the source of the `Io` variant, so a chain walker reaches it.
#[test]
fn the_io_variant_reports_the_wrapped_io_error_as_its_source() {
    let error = PlayerError::from(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "no access"));
    let source = error.source().expect("the Io variant has a source");
    assert_eq!(source.to_string(), "no access");
    let inner = source.downcast_ref::<std::io::Error>().expect("the source is the wrapped io error");
    assert_eq!(inner.kind(), std::io::ErrorKind::PermissionDenied);
}

/// A wrapped symphonia error is the source of the `Decode` variant.
#[test]
fn the_decode_variant_reports_the_wrapped_symphonia_error_as_its_source() {
    let error = PlayerError::from(symphonia::core::errors::Error::DecodeError("bad frame"));
    let source = error.source().expect("the Decode variant has a source");
    assert_eq!(source.to_string(), "malformed stream: bad frame");
    assert!(source.downcast_ref::<symphonia::core::errors::Error>().is_some());
}

/// A wrapped opus error is the source of the `Opus` variant. The error comes from a decoder the library rejects.
#[test]
fn the_opus_variant_reports_the_wrapped_opus_error_as_its_source() {
    let opus_error = opus::Decoder::new(12345, opus::Channels::Mono).expect_err("12345 Hz is not an Opus rate");
    let message = opus_error.to_string();
    let error = PlayerError::from(opus_error);
    let source = error.source().expect("the Opus variant has a source");
    assert_eq!(source.to_string(), message);
    assert!(source.downcast_ref::<opus::Error>().is_some());
}

/// The two message variants wrap no error value, so they have no source.
#[test]
fn the_message_variants_have_no_source() {
    assert!(PlayerError::Unsupported("a codec".to_string()).source().is_none());
    assert!(PlayerError::Audio("no device".to_string()).source().is_none());
}

/// An `anyhow` report of a wrapped error lists the wrapped error as its cause.
#[test]
fn an_anyhow_report_lists_the_wrapped_error_in_its_cause_chain() {
    let report = anyhow::Error::new(PlayerError::from(std::io::Error::other("disk gone")));
    let chain: Vec<String> = report.chain().map(ToString::to_string).collect();
    assert_eq!(chain, vec!["i/o error: disk gone".to_string(), "disk gone".to_string()]);
}
