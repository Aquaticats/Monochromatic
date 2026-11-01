// What:     Tests of `walk_order.rs` against the folder walk itself, pulled in by `walk_order.rs`.
// Why:      The comparison replaces a walk, so the walk is the oracle: on random trees with names that sort differently by
//           bytes, by case and by number, every listed file must be found at its index, every absent file must land
//           where the walk would put it, and every folder's run must be exactly the files under it.

use super::*;
use crate::refresh_test_support::{Rng, TempTree, pick_stem};

/// The name parts: they collide on case, digits and leading zeros, so natural order differs from byte order.
const STEMS: [&str; 16] = ["a", "A", "b", "B", "a2", "a10", "a02", "Track 2", "Track 10", "é", "ß", "日本", "1", "01", "z", "Z z"];

/// What: `random_name` is a relative file name of up to three folder levels, ending in an extension.
/// Why: the walk's order has to hold for any shape, including a file and a folder with similar names.
fn random_name(rng: &mut Rng, suffix: &str) -> String {
    let depth: usize = rng.below(4);
    let mut relative: String = String::new();
    for _ in 0..depth {
        relative.push_str(pick_stem(&STEMS, rng));
        relative.push('/');
    }
    let extension: &str = ["flac", "mp3", "opus", "txt"][rng.below(4)];
    relative.push_str(&format!("{}{suffix}.{extension}", pick_stem(&STEMS, rng)));
    return relative;
}

#[test]
fn every_listed_file_is_found_at_its_index_and_every_folder_has_its_run() {
    for seed in 1..=40_u64 {
        let tree = TempTree::new(&format!("walk-order-{seed}"));
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        for _ in 0..70 {
            let name: String = random_name(&mut rng, "");
            tree.try_write(&name, "x");
        }
        let tracks: Vec<PathBuf> = tree.walk();
        for (index, track) in tracks.iter().enumerate() {
            assert_eq!(locate(&tracks, &tree.root, track), Ok(index), "seed {seed}: {} is found at its index", track.display());
        }
        let mut folders: Vec<PathBuf> = tracks
            .iter()
            .flat_map(|track| return track.ancestors().skip(1).take_while(|ancestor| return *ancestor != tree.root).map(Path::to_path_buf).collect::<Vec<_>>())
            .collect();
        folders.sort();
        folders.dedup();
        for folder in &folders {
            let expected: Vec<usize> = tracks.iter().enumerate().filter(|(_, track)| return track.starts_with(folder)).map(|(index, _)| return index).collect();
            let run: Range<usize> = subtree_range(&tracks, &tree.root, folder);
            assert_eq!(run.clone().collect::<Vec<_>>(), expected, "seed {seed}: the run of {}", folder.display());
            let own: usize = tracks[run.clone()].iter().filter(|track| return track.parent() == Some(folder.as_path())).count();
            assert_eq!(direct_files(&tracks, folder, run).len(), own, "seed {seed}: the files directly in {}", folder.display());
        }
    }
}

#[test]
fn a_file_that_is_not_listed_lands_where_the_walk_would_put_it() {
    for seed in 1..=25_u64 {
        let tree = TempTree::new(&format!("walk-order-absent-{seed}"));
        let mut rng = Rng(seed.wrapping_mul(0xD6E8_FEB8_6659_FD93) | 1);
        for _ in 0..50 {
            let name: String = random_name(&mut rng, "");
            tree.try_write(&name, "x");
        }
        let before: Vec<PathBuf> = tree.walk();
        for probe_round in 0..12 {
            let relative: String = random_name(&mut rng, &format!(" probe {probe_round}")).replace(".txt", ".flac");
            let probe: PathBuf = tree.path(&relative);
            let Err(place) = locate(&before, &tree.root, &probe) else {
                panic!("seed {seed}: an absent file was found");
            };
            if !tree.try_write(&relative, "x") {
                continue;
            }
            let after: Vec<PathBuf> = tree.walk();
            assert_eq!(after.iter().position(|track| return *track == probe), Some(place), "seed {seed}: {relative} lands at its place");
            std::fs::remove_file(&probe).expect("the probe is removed");
        }
    }
}

#[test]
fn a_file_comes_before_a_folder_whose_name_sorts_earlier() {
    let root = Path::new("/lib");
    assert_eq!(walk_cmp(root, Path::new("/lib/z.flac"), Path::new("/lib/a/x.flac")), Ordering::Less, "own files precede subfolders");
    assert_eq!(walk_cmp(root, Path::new("/lib/a/x.flac"), Path::new("/lib/z.flac")), Ordering::Greater);
    assert_eq!(walk_cmp(root, Path::new("/lib/Track 2.flac"), Path::new("/lib/Track 10.flac")), Ordering::Less, "natural order");
    assert_eq!(walk_cmp(root, Path::new("/lib/a/x.flac"), Path::new("/lib/a/x.flac")), Ordering::Equal);
}

#[test]
fn a_path_outside_the_root_does_not_panic_and_has_no_run() {
    let tracks: Vec<PathBuf> = vec![PathBuf::from("/lib/a.flac"), PathBuf::from("/lib/b/c.flac")];
    assert!(subtree_range(&tracks, Path::new("/lib"), Path::new("/elsewhere")).is_empty());
    assert!(locate(&tracks, Path::new("/lib"), Path::new("/elsewhere/q.flac")).is_err());
}
