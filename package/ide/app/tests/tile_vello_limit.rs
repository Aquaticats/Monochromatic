//! The limit of one texture of a tile,
//! the floor of Vello's per-tile command list,
//! and Vello's bin count.
//!
//! Vello 0.11 gives every 16 by 16 pixel tile of the picture 64 `u32` of its command list (`ptcl`) before it stores
//!  any command (`vello_shaders-0.11.0/shader/shared/ptcl.wgsl:6`),
//!  and the list has a fixed size of 8,388,608 elements (`vello_encoding-0.11.0/src/config.rs:406`).
//! That is 131,072 tiles,
//!  33.5 million pixels,
//!  128 MiB of `Rgba8Unorm`.
//! A texture past it renders wrong with no failure bit set,
//!  as a picture 1113 pixels wide and 16384 tall did on a list of 4,194,304 elements:
//!  290,992 bytes differed from the reference by up to 184 levels,
//!  and the renderer reported that nothing had failed,
//!  as `doc/planning/ide-vello-buffer-sizes.md` records in its section on silent corruption.
//! The 64 MiB rule of `MAX_RASTER_BYTES` keeps every tile of the application below that floor,
//!  and these tests keep the rule and the floor in step:
//!  raising the rule,
//!  or shrinking Vello's list,
//!  needs a list size computed from the target.
//! No window is opened.

/// The rule and the texture side limit under test.
use ide_app::shaped_text::{
    MAX_BINS, MAX_RASTER_BYTES, set_texture_dimension, texture_fits, tile_texture_rows,
};

/// Elements of Vello 0.11's command list at its default size.
const VELLO_PTCL_ELEMENTS: u64 = 1 << 23;
/// Elements of the command list every tile uses before any command.
const PTCL_FLOOR_PER_TILE: u64 = 64;
/// Pixels on a side of one tile.
const TILE_SIDE: u64 = 16;
/// Bytes of one texel of the tile texture (`Rgba8Unorm`).
const TEXEL_BYTES: u64 = 4;
/// Pixels on a side of one bin of Vello's coarse stage:
/// 16 tiles of 16 pixels.
const BIN_SIDE: u64 = 256;

/// The number of bins Vello 0.11 counts for a picture `columns` by `rows` pixels
/// (`vello-0.11.0/src/render.rs`:
/// the product of the coarse stage's workgroup counts,
/// which are
/// `ceil(width_in_tiles / 16)` and `ceil(height_in_tiles / 16)`,
/// warns above 256).
fn bins(columns: u64, rows: u64) -> u64 {
    let tiles_across = columns.div_ceil(TILE_SIDE);
    let tiles_down = rows.div_ceil(TILE_SIDE);
    return tiles_across.div_ceil(16) * tiles_down.div_ceil(16);
}

/// The number of tiles Vello lays over a picture `columns` by `rows` pixels.
fn tiles(columns: u64, rows: u64) -> u64 {
    return columns.div_ceil(TILE_SIDE) * rows.div_ceil(TILE_SIDE);
}

/// The tallest texture `columns` wide that the code accepts,
/// or nothing when none is.
fn tallest(columns: u64) -> Option<u64> {
    if !texture_fits(columns, 1) {
        return None;
    }
    let mut low: u64 = 1;
    let mut high: u64 = 1 << 20;
    while low < high {
        let middle = (low + high).div_ceil(2);
        if texture_fits(columns, middle) {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    return Some(low);
}

/// The byte rule is at most half of the texture the command list's floor can serve,
/// so a texture twice as large as the rule allows is the first that corrupts at Vello's default list size.
#[test]
fn the_byte_rule_is_at_most_half_of_what_the_command_list_floor_serves() {
    let floor_tiles = VELLO_PTCL_ELEMENTS / PTCL_FLOOR_PER_TILE;
    let floor_bytes = floor_tiles * TILE_SIDE * TILE_SIDE * TEXEL_BYTES;
    assert_eq!(floor_bytes, 128 * 1024 * 1024);
    assert!(
        MAX_RASTER_BYTES * 2 <= floor_bytes,
        "a rule of {MAX_RASTER_BYTES} bytes against a floor of {floor_bytes}"
    );
}

/// No texture the code accepts,
/// at any width,
/// on a device with any of the texture side limits in use or plausible,
/// takes more than three quarters of the command list for its per-tile floor,
/// counting the tiles that a size which is not a multiple of 16 rounds up to.
#[test]
fn no_accepted_tile_uses_more_than_three_quarters_of_the_command_list_floor() {
    let allowed = VELLO_PTCL_ELEMENTS / PTCL_FLOOR_PER_TILE * 3 / 4;
    let mut largest: u64 = 0;
    for side in [8192_u32, 16_384, 32_768] {
        set_texture_dimension(side);
        let mut columns: u64 = 1;
        while columns <= u64::from(side) {
            let rows =
                tallest(columns).expect("a one pixel tall texture fits any width up to the side");
            assert!(texture_fits(columns, rows));
            assert!(!texture_fits(columns, rows + 1));
            assert_eq!(tile_texture_rows(columns), rows);
            let used = tiles(columns, rows);
            largest = largest.max(used);
            assert!(
                used <= allowed,
                "side {side}: a texture {columns} by {rows} has {used} tiles against {allowed} allowed"
            );
            // Step through widths densely at the small end and by a growing stride after it.
            columns += columns.div_ceil(40).max(1);
        }
    }
    // Positive control: a width on the bin grid reaches the rule's own size (65,536 tiles, 64 MiB),
    // so the sweep does not skip the tall ones.
    set_texture_dimension(32_768);
    let on_grid = tallest(1024).expect("a texture 1024 wide fits");
    assert_eq!(tiles(1024, on_grid), 65_536);
    assert!(
        largest * TILE_SIDE * TILE_SIDE * TEXEL_BYTES >= MAX_RASTER_BYTES / 2,
        "the sweep reached only {largest} tiles"
    );
    // Positive control: a texture twice the rule's size would not be allowed.
    assert!(tiles(1024, 16_384 * 2) > allowed);
}

/// No texture the code accepts,
/// at any width and on any side limit in use or plausible,
/// covers more than 256 bins by Vello's own formula,
/// so the renderer never logs "Trying to paint too large image" for a tile.
#[test]
fn no_accepted_texture_covers_more_bins_than_vello_counts_before_it_warns() {
    for side in [8192_u32, 16_384, 32_768] {
        set_texture_dimension(side);
        let mut columns: u64 = 1;
        while columns <= u64::from(side) {
            let rows =
                tallest(columns).expect("a one pixel tall texture fits any width up to the side");
            assert!(
                bins(columns, rows) <= MAX_BINS,
                "side {side}: a texture {columns} by {rows} covers {} bins",
                bins(columns, rows)
            );
            // The next row would be the first to pass the limit, unless the side stops it.
            if rows < u64::from(side) {
                assert!(
                    bins(columns, rows + 1) > MAX_BINS || rows + 1 > u64::from(side),
                    "side {side}: a texture {columns} by {rows} is not the tallest of its bin count"
                );
            }
            columns += 1 + columns / 97;
        }
    }
}

/// The tile of the scale 2 measurement:
/// 1358 columns.
/// The byte rule alone allowed 12,288 rows (66,748,416 bytes),
/// which is 6 by 48 = 288 bins and made Vello log its warning at every render;
/// on the bin grid the most is 42 bins,
/// 10,752 rows.
#[test]
fn the_texture_of_the_scale_two_measurement_is_on_the_bin_grid() {
    set_texture_dimension(16_384);
    // Positive control: the old rule's size counts more bins than Vello accepts.
    assert_eq!(1358 * 12_288 * TEXEL_BYTES, 66_748_416);
    assert_eq!(bins(1358, 12_288), 288);
    assert!(!texture_fits(1358, 12_288));
    assert!(texture_fits(1358, 10_752));
    assert_eq!(bins(1358, 10_752), 252);
    assert_eq!(tile_texture_rows(1358), 42 * BIN_SIDE);
}
