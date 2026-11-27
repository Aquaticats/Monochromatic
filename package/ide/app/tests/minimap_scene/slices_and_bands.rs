//! The picture cut into slices and bands:
//!  consecutive slices meeting without a gap or an overlap,
//!  and the most bands that fit the texture limit.

use ide_app::minimap::{
    BAND,
    Geometry,
    band_count,
    most_bands,
    picture_rows,
    slice_rows,
};

/// The slices of consecutive windows meet without a gap or an overlap,
/// the first starts at the picture's first row and the last ends at its last,
/// and the whole picture is as tall as the text at the fixed scale.
#[test]
fn consecutive_slices_meet_and_cover_the_picture() {
    for scale in [1.0_f32, 1.25, 1.5, 2.0, 3.0] {
        let height = 7_310.5_f32;
        let ratio = f64::from(Geometry::new(height, 660.0).scale());
        let count = band_count(height, BAND);
        let total = picture_rows(height, scale, ratio);
        assert!(count > 8);
        let mut at = 0;
        for cut in [0, 3, 3, 4, 9.min(count - 1), count] {
            let rows = slice_rows(&(at..cut.max(at)), height, scale, ratio);
            at = cut.max(at);
            assert!(rows.start <= rows.end, "a slice runs backwards at {scale}");
        }
        let first = slice_rows(&(0..3), height, scale, ratio);
        let second = slice_rows(&(3..count), height, scale, ratio);
        assert_eq!(first.start, 0);
        assert_eq!(
            first.end, second.start,
            "the slices share an edge at scale {scale}"
        );
        assert_eq!(
            second.end, total,
            "the last slice ends at the picture's end"
        );
        assert_eq!(
            slice_rows(&(0..count), height, scale, ratio),
            0..total,
            "all the bands make up the whole picture"
        );
        assert!(
            (total as f64 - f64::from(height) * f64::from(scale) / 6.0).abs() < 1.0,
            "the picture is a sixth of the text"
        );
    }
}

/// A window of the most bands one texture can hold is a slice no taller than the texture,
/// at every scale and for the guaranteed and the larger limits,
/// and two bands more are not (the cap wastes less than a band).
#[test]
fn the_most_bands_fit_the_texture_limit_and_one_more_does_not() {
    let height = 400_000.0_f32;
    let ratio = f64::from(Geometry::new(height, 660.0).scale());
    for limit in [2048_u32, 4096, 8192, 16384] {
        for scale in [1.0_f32, 1.25, 1.5, 2.0, 3.0] {
            let most = most_bands(limit, scale, ratio);
            let rows = slice_rows(&(10..10 + most), height, scale, ratio);
            assert!(
                rows.len() <= limit as usize,
                "{} rows pass the limit {limit} at scale {scale}",
                rows.len()
            );
            let more = slice_rows(&(10..12 + most), height, scale, ratio);
            assert!(
                more.len() > limit as usize || most == 1,
                "two more bands still fit {limit} at scale {scale}"
            );
        }
    }
}
