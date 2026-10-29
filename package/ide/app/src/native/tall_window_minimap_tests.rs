//! The minimap of a tall window stays a picture of the text,
//!  the window's own shaped lines drawn small,
//!  and never turns into blocks.
//!  The textures it is drawn into never pass the device's limit,
//!  and a window taller than the limit gets several that stack.
//! Nothing here waits on the window thread:
//! every wait advances timers and lets the workers answer.

/// The reader fixture.
use super::find_tests::Reader;
/// The reader fixture,
/// the wrapped text,
/// the window sizing,
/// and the reads of the minimap's texture.
use super::{
    minimap_tests::{bands_of, picture_still},
    sidebar_tests::{HEADER, TEXT_LEFT, minimap_beside},
    tile_reads::{draw_minimap, minimap_pixels, same_pixels, same_texel},
    wrap_tests::{long_text, open, settled},
};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::Waiting;
/// The window's frame,
///  read back from the GPU,
///  and how often the window was given a new list of the minimap's textures.
use crate::native::{gpu_snapshot::snapshot, gpu_tile::SLICE_LISTS_SET};
/// The device's texture limit,
///  and lowering it for a test.
use ide_app::shaped_text::{set_texture_dimension, texture_dimension};
/// Toolkit events.
use slint::{
    ComponentHandle, LogicalSize, Rgba8Pixel,
    platform::{WindowEvent, update_timers_and_animations},
};
/// Counters are atomic.
use std::sync::atomic::Ordering;
/// A window `height` logical pixels tall over four hundred wrapped lines,
/// once its first picture is on screen,
/// and the line measurements have arrived when `measured` asks for them.
/// A window as tall as twelve thousand pixels makes a tile the raster limit cuts,
/// whose measurement pass is not what these tests are about.
fn tall(height: f32, measured: bool) -> (tempfile::TempDir, Reader) {
    let (project, opened) = open(&long_text(400));
    let window = &opened.window;
    window.window().dispatch_event(WindowEvent::Resized {
        size: LogicalSize::new(TEXT_LEFT + 1330.0 + minimap_beside(1330.0), height),
    });
    update_timers_and_animations();
    if measured {
        settled(&opened);
    }
    {
        let mut waiting = Waiting::slow();
        loop {
            {
                let current = opened.source.borrow();
                let track = window.get_minimap_track();
                if current.minimap.shown.is_some()
                    && current.minimap.posted == current.minimap.shown
                    && current.minimap.images_set > 0
                    && track > 1000.0
                {
                    break;
                }
            }
            waiting.step("the minimap picture of the tall window did not arrive");
        }
    }
    draw_minimap(window);
    // The picture of the window before it was resized is also posted and shown;
    // under load the test could read it before the picture of the tall window arrived.
    {
        let mut waiting = Waiting::slow();
        while window.get_minimap_picture_height() <= 1000.0 {
            waiting.step("the texture of the tall window was not drawn");
            draw_minimap(window);
        }
    }
    return (project, opened);
}

/// What:
///  Count the rows of `pixels` (`width` to a row) that are inked,
///  and among them the rows whose coverage rises and falls along the row like the glyphs of a line do.
/// Why:
///  The picture is the glyphs' coverage,
///  so it is graded;
///  a block would be one flat run of a single coverage,
///  while glyphs make the coverage rise and fall with every word and gap.
fn textured_rows(pixels: &[slint::Rgba8Pixel], width: usize) -> (usize, usize) {
    let mut inked_rows = 0;
    let mut glyph_rows = 0;
    for row in pixels.chunks(width) {
        let mut strongest = 0;
        for pixel in row {
            strongest = strongest.max(pixel.a);
        }
        // Rows between lines carry only the faint tails of descenders and ascenders.
        if strongest < 40 {
            continue;
        }
        inked_rows += 1;
        // Count alternations between clearly inked and clearly lighter pixels.
        let high = strongest / 2;
        let mut alternations = 0;
        let mut inked = row[0].a > high;
        for pixel in &row[1..] {
            let now = if inked {
                pixel.a > high / 2
            } else {
                pixel.a > high
            };
            if now != inked {
                alternations += 1;
                inked = now;
            }
        }
        if alternations >= 4 {
            glyph_rows += 1;
        }
    }
    return (inked_rows, glyph_rows);
}

/// A tall window shows a minimap whose picture covers its track,
/// whose rows hold glyph texture,
/// and whose texture stays within the device's limit.
#[test]
fn the_minimap_of_a_tall_window_is_a_picture_of_glyphs_not_blocks() {
    for (height, measured) in [(3500.0, true), (12000.0, false)] {
        let (_project, opened) = tall(height, measured);
        let window = &opened.window;
        let image = minimap_pixels(window, &opened.source).expect("picture pixels");
        assert!(
            image.height() > 1000,
            "the picture is only {} rows tall",
            image.height()
        );
        let (inked_rows, glyph_rows) = textured_rows(image.as_slice(), image.width() as usize);
        assert!(
            inked_rows > 100,
            "the picture of a {height} window holds only {inked_rows} inked rows"
        );
        assert!(
            glyph_rows * 10 >= inked_rows * 9,
            "only {glyph_rows} of {inked_rows} inked rows rise and fall like glyphs at {height}"
        );
        let sizes = opened.source.borrow().gpu.minimap_texture_sizes();
        assert!(!sizes.is_empty(), "no texture was drawn");
        for (width, rows) in &sizes {
            assert!(
                *rows <= texture_dimension() && *width <= texture_dimension(),
                "a texture of {width}x{rows} passes the device's limit"
            );
        }
        let stats = opened
            .source
            .borrow()
            .minimap
            .last_stats
            .expect("build statistics");
        assert_eq!(stats.skipped, 0, "no line was left out at {height}");
        window.hide().expect("close minimap window");
    }
}

/// Positive control for the texture count:
/// a flat block of coverage is not counted as glyphs,
/// and alternating coverage is.
#[test]
fn the_texture_count_tells_a_block_from_glyphs() {
    let flat = vec![
        slint::Rgba8Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 120,
        };
        40
    ];
    assert_eq!(textured_rows(&flat, 40), (1, 0));
    let mut striped = flat.clone();
    for (index, pixel) in striped.iter_mut().enumerate() {
        if index % 4 < 2 {
            pixel.a = 20;
        }
    }
    assert_eq!(textured_rows(&striped, 40), (1, 1));
}

/// Lines of about two hundred and thirty characters wrap to a second row in a window this wide,
/// and cost far more than the lines of ordinary code.
fn long_file(lines: usize) -> String {
    let mut text = String::new();
    for number in 0..lines {
        text.push_str(&format!(
            "fn item_{number}(value: u32) -> u32 {{ value + {number} }} // trailing {}\n",
            "words ".repeat(30)
        ));
    }
    return text;
}

/// Wait for the first picture of the reader,
/// then resize its window to `height` and wait for the picture of that
/// height,
/// drawn.
fn resized_to(opened: &Reader, height: f32, track: f32) {
    let window = &opened.window;
    {
        let mut waiting = Waiting::slow();
        while opened.source.borrow().minimap.last_stats.is_none() {
            waiting.step("the first picture of the long file did not arrive");
        }
    }
    let images = opened.source.borrow().minimap.images_set;
    window.window().dispatch_event(WindowEvent::Resized {
        size: LogicalSize::new(TEXT_LEFT + 1330.0 + minimap_beside(1330.0), height),
    });
    update_timers_and_animations();
    {
        let mut waiting = Waiting::slow();
        loop {
            {
                let current = opened.source.borrow();
                // The picture is for the minimap's size as it now stands, not for the size before the resize.
                let size = current.minimap.size;
                let sized = match current.minimap.shown {
                    Some(shown) => shown.size == (size.0.to_bits(), size.1.to_bits()),
                    None => false,
                };
                if current.minimap.images_set > images
                    && current.minimap.posted == current.minimap.shown
                    && window.get_minimap_track() > track
                    && sized
                {
                    break;
                }
            }
            waiting.step("the picture of the tall window did not arrive");
        }
    }
    // The picture checked is the one that stood still while the line measurements finished.
    picture_still(opened);
    draw_minimap(window);
}

/// A tall window over a long file draws only the lines near the visible part of the picture,
/// all within the work budget,
/// and leaves none out.
#[test]
fn a_tall_window_over_a_long_file_draws_a_window_of_bands() {
    let (_project, opened) = open(&long_file(6_000));
    let window = &opened.window;
    resized_to(&opened, 3500.0, 3000.0);
    let stats = opened
        .source
        .borrow()
        .minimap
        .last_stats
        .expect("build statistics");
    assert_eq!(stats.skipped, 0, "no line was left out");
    assert_eq!(stats.shaped + stats.reused, stats.lines);
    assert!(
        stats.units * 4 < bands_of(&opened),
        "{} of {} bands were built for one view",
        stats.units,
        bands_of(&opened)
    );
    window.hide().expect("close minimap window");
}

/// A window taller than the device's texture limit gets a picture that covers its whole track:
/// no error is shown,
/// and the picture reaches the end of the track,
/// not the end of one texture.
#[test]
fn a_picture_taller_than_the_device_limit_covers_the_whole_track() {
    let (_project, opened) = open(&long_file(6_000));
    let window = &opened.window;
    // The window is 12,000 px tall at scale 1: its track is 12,000 px, past the harness device's 8192.
    resized_to(&opened, 12_000.0, 9_000.0);
    let limit = texture_dimension();
    assert_eq!(
        limit, 8192,
        "the native tests run on a device capped to 8192 px"
    );
    let track = window.get_minimap_track() * window.window().scale_factor();
    assert!(track > limit as f32, "the track is {track} px");
    let image = minimap_pixels(window, &opened.source).expect("a texture");
    assert_eq!(
        window.get_minimap_picture_top(),
        0.0,
        "the view is scrolled to the top"
    );
    assert!(
        window.get_minimap_picture_height() + 1.0 >= track,
        "the picture is {} rows of a track of {track}",
        window.get_minimap_picture_height()
    );
    // The rows are in several textures, each within the limit, that together are the rows the window places.
    let sizes = opened.source.borrow().gpu.minimap_texture_sizes();
    assert!(sizes.len() >= 2, "the picture is one texture: {sizes:?}");
    for (width, rows) in &sizes {
        assert!(
            *rows <= limit && *width <= limit,
            "a texture of {width}x{rows} passes the limit {limit}"
        );
    }
    assert_eq!(
        sizes.iter().map(|(_, rows)| return *rows).sum::<u32>(),
        image.height(),
        "the textures together are the rows that were read back"
    );
    assert_eq!(
        window.get_minimap_picture_height(),
        image.height() as f32,
        "the markup places the rows that were read back"
    );
    assert_eq!(
        window.get_error_message().to_string(),
        "",
        "the tall picture is not an error"
    );
    // The picture is several renders, and none of them came out blank.
    let chunks = opened.source.borrow().gpu.minimap_chunks_rendered();
    let wanted = u64::from(image.height().div_ceil(ide_app::minimap::CHUNK_ROWS));
    assert!(
        wanted > 1 && chunks >= wanted,
        "{chunks} chunks rendered for a texture of {wanted} chunks"
    );
    let width = image.width() as usize;
    let pixels = image.as_slice();
    let mut inked_by_chunk = Vec::new();
    for chunk in 0..wanted as usize {
        let first = chunk * ide_app::minimap::CHUNK_ROWS as usize * width;
        let last = ((chunk + 1) * ide_app::minimap::CHUNK_ROWS as usize * width).min(pixels.len());
        inked_by_chunk.push(
            pixels[first..last]
                .iter()
                .filter(|pixel| return pixel.a > 0)
                .count(),
        );
    }
    assert!(
        inked_by_chunk.iter().all(|inked| return *inked > 0),
        "a chunk of the slice is blank: inked pixels by chunk {inked_by_chunk:?}, texture {}x{}, {chunks} chunks rendered, {} slices, statistics {:?}",
        image.width(),
        image.height(),
        opened.source.borrow().gpu.minimaps_rendered(),
        opened.source.borrow().minimap.last_stats
    );
    window.hide().expect("close minimap window");
}

/// The pixels of the minimap's column in the window's frame,
/// row by row from the top:
/// the textures as the window composites them,
///  with the box over them.
fn minimap_column(window: &crate::native::AppWindow) -> (Vec<Rgba8Pixel>, usize) {
    let frame = snapshot(window.window());
    let columns = frame.width() as usize;
    let width = (window.get_minimap_width() * window.window().scale_factor()).round() as usize;
    let mut column = Vec::new();
    for row in frame.as_slice().chunks(columns) {
        column.extend_from_slice(&row[columns - width..]);
    }
    return (column, width);
}

/// Lower the device limit the shaped text records to `limit` and wait for the picture of the same bands,
///  cut for it.
fn restacked(opened: &Reader, limit: u32) {
    let window = &opened.window;
    set_texture_dimension(limit);
    // The window asks for a picture after a render of its own; a change of the limit alone starts none.
    super::minimap::note(window, &opened.source);
    let mut waiting = Waiting::slow();
    loop {
        draw_minimap(window);
        {
            let current = opened.source.borrow();
            let cut = match current.minimap.shown {
                Some(shown) => shown.texture_limit == limit,
                None => false,
            };
            if cut && current.minimap.posted == current.minimap.shown {
                break;
            }
        }
        waiting.step("the picture cut for the lowered texture limit did not arrive");
    }
    draw_minimap(window);
}

/// Whether `stacked` is `whole` shifted down by one row of `width` texels,
///  which no picture of text equals.
fn shifted_equals(whole: &[Rgba8Pixel], stacked: &[Rgba8Pixel], width: usize) -> bool {
    return same_pixels(&whole[width..], &stacked[..stacked.len() - width]);
}

/// A picture that fits one texture,
///  drawn again as three,
///  as four,
///  and as seven or more textures that stack,
/// is the same picture:
/// every texel read back equals the texel of the one texture,
/// the rows on both sides of each seam are inked and equal,
/// and the window composites the stack into the same column of pixels the one texture gave.
/// A shift of the stack by a single row is seen by both comparisons.
#[test]
fn textures_that_stack_show_the_picture_one_texture_shows() {
    let (_project, opened) = tall(3500.0, true);
    let window = &opened.window;
    // The frame the harness reads is as big as the window the adapter was told to be,
    // which a resize event alone does not change, and the frame is wanted whole.
    window.window().set_size(LogicalSize::new(
        TEXT_LEFT + 1330.0 + minimap_beside(1330.0),
        3500.0,
    ));
    update_timers_and_animations();
    picture_still(&opened);
    draw_minimap(window);
    let whole = minimap_pixels(window, &opened.source).expect("the picture in one texture");
    let width = whole.width() as usize;
    assert_eq!(
        opened.source.borrow().gpu.minimap_texture_sizes().len(),
        1,
        "the picture of a window that fits is one texture"
    );
    assert!(
        whole.height() > 3000,
        "the picture is {} rows",
        whole.height()
    );
    let (column, column_width) = minimap_column(window);
    let bands = opened
        .source
        .borrow()
        .minimap
        .shown
        .map(|key| return key.window);
    let top = window.get_minimap_picture_top();
    for limit in [1536_u32, 1024, 512] {
        restacked(&opened, limit);
        let current = opened.source.borrow().minimap.shown;
        assert_eq!(
            current.map(|key| return key.window),
            bands,
            "the same bands are drawn at limit {limit}"
        );
        assert_eq!(
            window.get_minimap_picture_top(),
            top,
            "the picture starts on the same row at limit {limit}"
        );
        let sizes = opened.source.borrow().gpu.minimap_texture_sizes();
        let per_texture = limit / ide_app::minimap::CHUNK_ROWS * ide_app::minimap::CHUNK_ROWS;
        assert_eq!(
            sizes.len() as u32,
            whole.height().div_ceil(per_texture),
            "textures at limit {limit}: {sizes:?}"
        );
        assert!(
            sizes.len() >= 3,
            "only {} textures at limit {limit}",
            sizes.len()
        );
        for (index, (texture_width, rows)) in sizes.iter().enumerate() {
            assert_eq!(*texture_width as usize, width);
            assert!(*rows <= limit, "a texture of {rows} rows passes {limit}");
            assert!(
                index + 1 == sizes.len() || *rows == per_texture,
                "only the last texture is short: {sizes:?}"
            );
        }
        assert_eq!(
            window.get_minimap_picture_height(),
            whole.height() as f32,
            "the stack is as tall as the one texture was at limit {limit}"
        );
        let stacked = minimap_pixels(window, &opened.source).expect("the picture in a stack");
        assert_eq!(stacked.height(), whole.height());
        // Every texel of the stack is the texel of the one texture.
        assert!(
            same_pixels(whole.as_slice(), stacked.as_slice()),
            "the stack of {} textures differs from the one texture at limit {limit}",
            sizes.len()
        );
        // The rows on both sides of each seam are inked, and so the equality above is not of blank rows.
        let mut seam = 0;
        for (_, rows) in &sizes[..sizes.len() - 1] {
            seam += *rows as usize;
            let around = &stacked.as_slice()[(seam - 8) * width..(seam + 8) * width];
            let inked = around.iter().filter(|pixel| return pixel.a > 0).count();
            assert!(inked > 0, "the rows around the seam at {seam} are blank");
            let same_around = whole.as_slice()[(seam - 8) * width..(seam + 8) * width]
                .iter()
                .zip(around)
                .all(|(one, other)| return same_texel(*one, *other));
            assert!(same_around, "the rows around the seam at {seam} differ");
        }
        // Positive control: a stack that sat one row lower would not be equal.
        assert!(
            !shifted_equals(whole.as_slice(), stacked.as_slice(), width),
            "a shift of one row went unseen"
        );
        // The window composites the stack into the column the one texture gave:
        // the rows on both sides of each seam, which stand below the window's header, are inked in it.
        let (stack_column, stack_width) = minimap_column(window);
        assert_eq!(stack_width, column_width);
        let background = column[0];
        let mut frame_seam = HEADER as usize;
        let mut frame_seams_seen = 0;
        for (_, rows) in &sizes[..sizes.len() - 1] {
            frame_seam += *rows as usize;
            // A seam below the bottom of the frame is not in the composited column.
            if frame_seam + 8 > stack_column.len() / column_width {
                break;
            }
            frame_seams_seen += 1;
            for side in [frame_seam - 8..frame_seam, frame_seam..frame_seam + 8] {
                let inked = stack_column[side.start * column_width..side.end * column_width]
                    .iter()
                    .filter(|pixel| {
                        return pixel.r != background.r
                            || pixel.g != background.g
                            || pixel.b != background.b;
                    })
                    .count();
                assert!(inked > 0, "the composited column is blank at rows {side:?}");
            }
        }
        assert!(
            frame_seams_seen >= 2,
            "only {frame_seams_seen} seams are in the frame at limit {limit}"
        );
        assert!(
            same_pixels(&column, &stack_column),
            "the composited column differs at limit {limit}"
        );
        assert!(
            !same_pixels(
                &column[column_width..],
                &stack_column[..stack_column.len() - column_width]
            ),
            "a shift of one row of the composited column went unseen"
        );
        assert_eq!(window.get_error_message().to_string(), "");
    }
    set_texture_dimension(8192);
    window.hide().expect("close minimap window");
}

/// A picture of the same shape as the one on screen,
/// asked for again by a change of the minimap's size of one pixel,
/// is drawn into the same textures and shown by changing the rows of the list the window already holds:
/// the markup is not given a new list,
/// which would make it drop and make every image again inside a frame of a scroll.
#[test]
fn a_picture_of_the_same_shape_changes_the_rows_of_the_list_and_never_replaces_it() {
    let (_project, opened) = tall(3500.0, true);
    let window = &opened.window;
    picture_still(&opened);
    draw_minimap(window);
    let lists = SLICE_LISTS_SET.load(Ordering::SeqCst);
    let (drawn, images, sizes) = {
        let current = opened.source.borrow();
        (
            current.gpu.minimaps_rendered(),
            current.minimap.images_set,
            current.gpu.minimap_texture_sizes(),
        )
    };
    let bands = opened
        .source
        .borrow()
        .minimap
        .shown
        .map(|key| return key.window);
    window.window().dispatch_event(WindowEvent::Resized {
        size: LogicalSize::new(TEXT_LEFT + 1330.0 + minimap_beside(1330.0), 3501.0),
    });
    update_timers_and_animations();
    {
        let mut waiting = Waiting::slow();
        loop {
            draw_minimap(window);
            {
                let current = opened.source.borrow();
                if current.minimap.images_set > images
                    && current.minimap.posted == current.minimap.shown
                    && current.gpu.minimaps_rendered() > drawn
                {
                    break;
                }
            }
            waiting.step("the picture of the resized window did not arrive");
        }
    }
    let current = opened.source.borrow();
    assert_eq!(
        current.minimap.shown.map(|key| return key.window),
        bands,
        "the resize named other bands, so the picture is not of the same shape"
    );
    assert_eq!(
        current.gpu.minimap_texture_sizes(),
        sizes,
        "the textures were made again"
    );
    assert_eq!(
        SLICE_LISTS_SET.load(Ordering::SeqCst),
        lists,
        "the window was given a new list of textures for a picture of the same shape"
    );
    drop(current);
    window.hide().expect("close minimap window");
}
