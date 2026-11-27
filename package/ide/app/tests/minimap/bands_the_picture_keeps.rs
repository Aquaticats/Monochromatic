//! The bands of the picture a window keeps:
//!  those in view and one on each side,
//!  a cap that keeps those from the top of the track,
//!  the lead a fling reaches on the side it heads toward,
//!  and the whole band a heading turns on.

/// The geometry and its constants.
use ide_app::minimap::{
    BAND,
    Geometry,
    Heading,
    LEAD,
    band_count,
};
/// The vertical mapping the minimap shows.
use ide_app::row_map::CODE_ROW;

/// The bands to keep are the ones the track shows and one on each side,
/// held to the text,
/// and the same bands for a scroll that stays within a band.
#[test]
fn the_bands_to_keep_surround_the_part_of_the_picture_in_view() {
    let step = 24.0 * 33.0;
    let content = 50_000.0 * CODE_ROW;
    let geometry = Geometry::new(content, 600.0);
    let total = band_count(content, step);
    assert_eq!(total, (content / step).ceil() as usize);
    for share in [0.0_f32, 0.001, 0.4, 0.999, 1.0] {
        let offset = geometry.max_offset() * share;
        let bands = geometry.bands_for(offset, step, usize::MAX);
        let shown_from = geometry.slide(offset) / geometry.scale();
        let shown_to = (geometry.slide(offset) + geometry.track()) / geometry.scale();
        assert!(bands.end <= total && bands.start < bands.end);
        // A whole band of margin lies on each side of the part in view, but at the ends of the text.
        assert!(
            bands.start == 0 || (bands.start as f32 + 1.0) * step <= shown_from,
            "at {share} the bands start at {} after the part in view at {shown_from} less a band",
            bands.start
        );
        assert!(
            bands.end == total || bands.end as f32 * step >= shown_to + step,
            "at {share} the bands end at {} before the part in view ends at {shown_to} plus a band",
            bands.end
        );
        // The picture's 600 px are 3,600 px of text, five bands of this height, with one more on each side.
        assert!(bands.len() <= 8, "{} bands for one view", bands.len());
    }
    // A scroll within a band's height of the picture's slide names the same bands.
    let at = geometry.max_offset() * 0.5;
    let same = geometry.bands_for(at + 1.0, step, usize::MAX);
    assert_eq!(geometry.bands_for(at, step, usize::MAX), same);
    // A short text is one window of all its bands.
    let short = Geometry::new(10.0 * CODE_ROW, 600.0);
    assert_eq!(short.bands_for(0.0, step, usize::MAX), 0..1);
    assert_eq!(
        Geometry::new(0.0, 600.0).bands_for(0.0, step, usize::MAX),
        0..0
    );
}

/// A texture holds only so many bands:
/// a window capped to that many starts at the band the track starts in,
/// keeps the bands below it,
/// and never names more than the cap.
#[test]
fn a_cap_on_the_bands_keeps_those_from_the_top_of_the_track() {
    let step = 768.0;
    let content = 50_000.0 * CODE_ROW;
    // A view 12,000 px tall: its track is 12,000 px, 15.6 bands of the text's 768 px at a sixth.
    let geometry = Geometry::new(content, 12_000.0);
    let offset = geometry.max_offset() * 0.4;
    let free = geometry.bands_for(offset, step, usize::MAX);
    assert!(free.len() > 20, "the free window is {} bands", free.len());
    for cap in [1_usize, 8, 16, 64] {
        let capped = geometry.bands_for(offset, step, cap);
        assert!(
            capped.len() <= cap,
            "{} bands past the cap {cap}",
            capped.len()
        );
        assert!(capped.start >= free.start && capped.end <= free.end);
        if cap < free.len() {
            let shown_from = geometry.slide(offset) / geometry.scale();
            assert!(
                capped.start as f32 * step <= shown_from
                    && shown_from < (capped.start as f32 + 1.0) * step,
                "the capped window {capped:?} does not start in the band the track starts in ({shown_from})"
            );
        }
    }
}

/// A view heading down holds `LEAD` bands past the track and one before it,
/// a view heading up the other way round,
/// and a view at rest one on each side:
/// the window of a fling reaches the bands the next picture needs.
#[test]
fn the_window_of_a_fling_reaches_ahead_on_the_side_it_heads_toward() {
    let step = BAND;
    let content = 50_000.0 * CODE_ROW;
    let geometry = Geometry::new(content, 600.0);
    let offset = geometry.max_offset() * 0.4;
    let still = geometry.bands_for(offset, step, usize::MAX);
    assert_eq!(
        geometry.bands_toward(
            offset,
            step,
            usize::MAX,
            Heading::Still {
                at: 7.0
            }
        ),
        still,
        "a view at rest keeps one band on each side"
    );
    let down = geometry.bands_toward(
        offset,
        step,
        usize::MAX,
        Heading::Down {
            at: 0.0,
        },
    );
    let up = geometry.bands_toward(
        offset,
        step,
        usize::MAX,
        Heading::Up {
            at: 0.0,
        },
    );
    assert_eq!(
        down.start, still.start,
        "heading down keeps one band behind"
    );
    assert_eq!(
        down.end,
        still.end - 1 + LEAD,
        "heading down holds the lead after the track"
    );
    assert_eq!(up.end, still.end, "heading up keeps one band behind");
    assert_eq!(
        up.start,
        still.start + 1 - LEAD,
        "heading up holds the lead before the track"
    );
    // The window never leaves the text: at its ends the lead is cut short.
    let total = band_count(content, step);
    let last = geometry.bands_toward(
        geometry.max_offset(),
        step,
        usize::MAX,
        Heading::Down {
            at: 0.0,
        },
    );
    assert_eq!(last.end, total);
    let first = geometry.bands_toward(
        0.0,
        step,
        usize::MAX,
        Heading::Up {
            at: 0.0,
        },
    );
    assert_eq!(first.start, 0);
    // A cap on the bands still wins over the lead.
    let capped = geometry.bands_toward(
        offset,
        step,
        3,
        Heading::Down {
            at: 0.0,
        },
    );
    assert!(capped.len() <= 3, "{} bands past the cap", capped.len());
}

/// A heading is confirmed by a move of a whole band from where it was decided,
/// kept through a creep of less than a band either way,
/// and turned by a whole band against it.
#[test]
fn a_heading_follows_the_track_by_whole_bands() {
    let step = BAND;
    let still = Heading::Still {
        at: 1_000.0,
    };
    assert_eq!(
        still.follow(1_000.0 + step - 1.0, step),
        still,
        "less than a band changes nothing"
    );
    assert_eq!(still.follow(1_000.0 - step + 1.0, step), still);
    let down = still.follow(1_000.0 + step, step);
    assert_eq!(
        down,
        Heading::Down {
            at: 1_000.0 + step
        }
    );
    // A creep back within a band keeps the heading and the place where it was decided.
    assert_eq!(down.follow(1_000.0 + step - 100.0, step), down);
    assert_eq!(down.follow(1_000.0 + 2.0 * step - 1.0, step), down);
    // The next band confirms it again, one step further.
    assert_eq!(
        down.follow(1_000.0 + 2.0 * step, step),
        Heading::Down {
            at: 1_000.0 + 2.0 * step
        }
    );
    // A whole band back turns it.
    assert_eq!(
        down.follow(1_000.0, step),
        Heading::Up {
            at: 1_000.0
        }
    );
    assert_eq!(
        Heading::Still {
            at: 0.0
        }
        .margins(),
        (1, 1)
    );
    assert_eq!(down.margins(), (1, LEAD));
    assert_eq!(
        Heading::Up {
            at: 0.0
        }
        .margins(),
        (LEAD, 1)
    );
}
