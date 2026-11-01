// What:     Tests of `change_plan.rs`, pulled in by `change_plan.rs`.
// Why:      The planner replaces a walk of the library by a few lookups, so the walk is the oracle: after random creates,
//           deletes, replacements, folder moves and folder recreations, the queue edited from the named paths must equal
//           the walk of the folder. The named paths are what the watcher would name, including the cases where it names
//           less than changed (a folder created and filled before its watch exists, a folder moved away whole).

use super::*;
use crate::disk_view::RealDisk;
use crate::refresh_test_support::{MemoryDisk, Rng, TempTree, pick_stem};
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};

/// The name parts: they collide on case, digits and leading zeros, so natural order differs from byte order.
const STEMS: [&str; 12] = ["a", "A", "b", "a2", "a10", "a02", "Track 2", "Track 10", "é", "1", "01", "z"];

/// What: `World` is a library folder and the queue a controller would hold for it.
/// Why: each test edits the folder, names what the watcher would name, and checks the queue follows.
struct World {
    tree: TempTree,
    outside: TempTree,
    queue: Vec<PathBuf>,
}

/// Methods of `World`.
impl World {
    /// What: `new` makes the folder and takes the first walk as the queue.
    /// Why: the queue starts as the scan of the folder, as after an open.
    fn new(tag: &str, files: &[&str]) -> World {
        let tree = TempTree::new(tag);
        for file in files {
            tree.write(file, "first");
        }
        let queue = tree.walk();
        return World { tree, outside: TempTree::new(&format!("{tag}-outside")), queue };
    }

    /// What: `named` is a change set of the given paths.
    /// Why: tests name paths by relative strings.
    fn named(&self, entries: &[(&str, bool)]) -> DirtyPaths {
        let mut set = DirtyPaths::default();
        for (relative, structural) in entries {
            set.insert(self.tree.path(relative), *structural);
        }
        return set;
    }

    /// What: `refresh` plans the named paths, applies the plan to the queue and returns the plan.
    /// Why: the step every test repeats.
    fn refresh(&mut self, dirty: &DirtyPaths) -> Plan {
        let plan = plan_changes(&self.queue, &self.tree.root, dirty, &RealDisk);
        self.queue = apply_plan(std::mem::take(&mut self.queue), &self.tree.root, &plan);
        return plan;
    }

    /// What: `assert_follows_the_disk` compares the queue with the walk of the folder.
    /// Why: the walk is the definition of the queue.
    fn assert_follows_the_disk(&self, context: &str) {
        assert_eq!(self.queue, self.tree.walk(), "{context}");
    }
}

#[test]
fn a_replaced_file_changes_no_list_and_is_the_only_file_to_look_at() {
    let mut world = World::new("plan-replace", &["a/one.flac", "a/two.flac", "b/three.flac"]);
    world.tree.write("a/two.flac", "second and longer");
    let plan = world.refresh(&world.named(&[("a/two.flac", false)]));
    assert!(!plan.changes_the_list());
    assert_eq!(plan.touched, vec![world.tree.path("a/two.flac")]);
    world.assert_follows_the_disk("nothing moved");
}

#[test]
fn a_new_file_is_added_in_its_place_and_a_deleted_one_is_removed() {
    let mut world = World::new("plan-add-remove", &["a/one.flac", "a/three.flac", "b/x.flac"]);
    world.tree.write("a/two.flac", "new");
    let plan = world.refresh(&world.named(&[("a/two.flac", true)]));
    assert_eq!(plan.added, vec![world.tree.path("a/two.flac")]);
    world.assert_follows_the_disk("added between its neighbours");
    std::fs::remove_file(world.tree.path("a/one.flac")).expect("removed");
    let removal = world.refresh(&world.named(&[("a/one.flac", true)]));
    assert_eq!(removal.removed, vec![0..1]);
    world.assert_follows_the_disk("removed");
}

#[test]
fn the_same_path_named_many_times_and_paths_outside_the_folder_do_no_harm() {
    let mut world = World::new("plan-duplicates", &["a/one.flac"]);
    world.tree.write("a/two.flac", "new");
    let mut dirty = world.named(&[("a/two.flac", true), ("a/two.flac", false), ("a/two.flac", true)]);
    dirty.insert(PathBuf::from("/definitely/not/in/the/library.flac"), true);
    dirty.insert(world.outside.path("x.flac"), true);
    let plan = world.refresh(&dirty);
    assert_eq!(plan.added.len(), 1, "once");
    world.assert_follows_the_disk("one file added once");
}

#[test]
fn a_folder_deleted_and_recreated_with_other_files_is_read_again() {
    let mut world = World::new("plan-recreate", &["album/old1.flac", "album/old2.flac", "album/deep/old3.flac", "other/keep.flac"]);
    std::fs::remove_dir_all(world.tree.path("album")).expect("removed");
    world.tree.write("album/new1.flac", "n");
    world.tree.write("album/old2.flac", "replaced by a longer one");
    world.tree.write("album/fresh/new2.flac", "n");
    let plan = world.refresh(&world.named(&[("album", true)]));
    assert!(plan.touched.contains(&world.tree.path("album/old2.flac")), "a file with the same name may hold new content");
    world.assert_follows_the_disk("the folder holds what it holds now");
}

#[test]
fn a_folder_removed_and_recreated_is_found_even_when_only_the_folder_is_named() {
    let mut world = World::new("plan-recreate-only-folder", &["album/a.flac", "album/sub/b.flac"]);
    std::fs::remove_dir_all(world.tree.path("album")).expect("removed");
    std::fs::create_dir_all(world.tree.path("album")).expect("created");
    world.refresh(&world.named(&[("album", true)]));
    world.assert_follows_the_disk("the recreated folder is empty");
}

#[test]
fn a_folder_moved_out_whole_is_removed_though_only_the_folder_is_named() {
    let mut world = World::new("plan-move-out", &["a/one.flac", "gone/x.flac", "gone/deep/y.flac", "z/last.flac"]);
    std::fs::rename(world.tree.path("gone"), world.outside.path("gone")).expect("moved out");
    let plan = world.refresh(&world.named(&[("gone", true)]));
    assert_eq!(plan.removed.iter().map(Range::len).sum::<usize>(), 2);
    world.assert_follows_the_disk("the whole run left");
}

#[test]
fn a_folder_removed_with_all_its_files_named_too_is_one_run_not_overlapping_runs() {
    let mut world = World::new("plan-one-run", &["a/one.flac", "gone/x.flac", "gone/y.flac", "gone/deep/z.flac", "z/last.flac"]);
    std::fs::remove_dir_all(world.tree.path("gone")).expect("removed");
    let dirty = world.named(&[("gone", true), ("gone/x.flac", true), ("gone/y.flac", true), ("gone/deep", true), ("gone/deep/z.flac", true)]);
    let plan = world.refresh(&dirty);
    assert_eq!(plan.removed, vec![1..4], "the folder and its files are one run");
    world.assert_follows_the_disk("the whole folder left");
}

#[test]
fn a_folder_created_and_filled_before_its_watch_existed_is_read_from_the_folder_event_alone() {
    let mut world = World::new("plan-new-folder", &["a/one.flac"]);
    world.tree.write("fresh/inner/song.flac", "x");
    world.tree.write("fresh/top.flac", "x");
    world.refresh(&world.named(&[("fresh", true)]));
    world.assert_follows_the_disk("the new folder's tree");
}

#[test]
fn a_folder_event_that_only_touched_the_folder_still_finds_entries_that_appeared_without_events() {
    let mut world = World::new("plan-folder-diff", &["a/one.flac", "a/sub/two.flac", "a/gone.flac"]);
    world.tree.write("a/three.flac", "x");
    world.tree.write("a/newsub/four.flac", "x");
    std::fs::remove_file(world.tree.path("a/gone.flac")).expect("removed");
    std::fs::remove_dir_all(world.tree.path("a/sub")).expect("removed");
    world.refresh(&world.named(&[("a", false)]));
    world.assert_follows_the_disk("the folder's own entries were compared");
}

#[test]
fn a_file_replaced_by_a_folder_of_the_same_name_and_back() {
    let mut world = World::new("plan-file-folder", &["a/x.flac", "a/y.flac"]);
    std::fs::remove_file(world.tree.path("a/x.flac")).expect("removed");
    world.tree.write("a/x.flac/inside.flac", "x");
    world.refresh(&world.named(&[("a/x.flac", true)]));
    world.assert_follows_the_disk("the file became a folder");
    std::fs::remove_dir_all(world.tree.path("a/x.flac")).expect("removed");
    world.tree.write("a/x.flac", "file again");
    world.refresh(&world.named(&[("a/x.flac", true)]));
    world.assert_follows_the_disk("and a file again");
}

#[test]
fn a_file_renamed_to_something_that_is_not_audio_leaves_the_list() {
    let mut world = World::new("plan-rename-away", &["a/x.flac", "a/y.flac"]);
    std::fs::rename(world.tree.path("a/x.flac"), world.tree.path("a/x.flac.part")).expect("renamed");
    world.refresh(&world.named(&[("a/x.flac", true), ("a/x.flac.part", true)]));
    world.assert_follows_the_disk("the partial file is not listed");
    std::fs::rename(world.tree.path("a/x.flac.part"), world.tree.path("a/x.flac")).expect("renamed back");
    world.refresh(&world.named(&[("a/x.flac", true), ("a/x.flac.part", true)]));
    world.assert_follows_the_disk("and is listed when it is finished");
}

#[test]
fn random_edits_named_the_way_the_watcher_names_them_keep_the_queue_equal_to_the_walk() {
    for seed in 1..=45_u64 {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let mut world = World::new(&format!("plan-random-{seed}"), &["a/one.flac", "a/b/two.mp3", "z.flac"]);
        for round in 0..10 {
            let mut dirty = DirtyPaths::default();
            for _ in 0..(1 + rng.below(4)) {
                random_edit(&world, &mut rng, &mut dirty, round);
            }
            world.refresh(&dirty);
            world.assert_follows_the_disk(&format!("seed {seed}, round {round}"));
        }
    }
}

/// What: `random_name` is a relative file name of one to three levels.
/// Why: random edits collide on names on purpose.
fn random_name(rng: &mut Rng) -> String {
    let depth: usize = rng.below(3);
    let mut relative: String = String::new();
    for _ in 0..depth {
        relative.push_str(pick_stem(&STEMS, rng));
        relative.push('/');
    }
    let extension: &str = ["flac", "mp3", "opus", "txt"][rng.below(4)];
    relative.push_str(&format!("{}.{extension}", pick_stem(&STEMS, rng)));
    return relative;
}

/// What: `random_edit` changes the folder in one random way and adds the paths the watcher would name to `dirty`.
/// Why: the watcher names a created and filled folder by the folder alone, a moved-away folder by itself alone, and a
/// removed folder by every entry, so each case is made here.
fn random_edit(world: &World, rng: &mut Rng, dirty: &mut DirtyPaths, round: usize) {
    let listed: Vec<PathBuf> = world.tree.walk();
    let tree = &world.tree;
    match rng.below(9) {
        0 => {
            let name = random_name(rng);
            let path = tree.path(&name);
            let first_new_ancestor: Option<PathBuf> = path.ancestors().skip(1).take_while(|ancestor| return *ancestor != tree.root).filter(|ancestor| return !ancestor.exists()).last().map(Path::to_path_buf);
            if tree.try_write(&name, "created") {
                dirty.insert(first_new_ancestor.unwrap_or(path), true);
            }
        }
        1 | 2 => {
            if let Some(path) = listed.get(rng.below(listed.len().max(1))) {
                std::fs::write(path, format!("replaced in round {round} with a different length {}", rng.below(1000))).expect("replaced");
                dirty.insert(path.clone(), false);
            }
        }
        3 => {
            if let Some(path) = listed.get(rng.below(listed.len().max(1))) {
                std::fs::remove_file(path).expect("removed");
                dirty.insert(path.clone(), true);
            }
        }
        4 => {
            let name = format!("{}/{}", pick_stem(&STEMS, rng), random_name(rng));
            let path = tree.path(&name);
            let first_new_ancestor: Option<PathBuf> = path.ancestors().skip(1).take_while(|ancestor| return *ancestor != tree.root).filter(|ancestor| return !ancestor.exists()).last().map(Path::to_path_buf);
            let sibling = format!("{}/sibling{round}.flac", name.rsplit_once('/').map_or("", |(parent, _)| return parent));
            if tree.try_write(&name, "x") && tree.try_write(&sibling, "x") {
                if let Some(folder) = first_new_ancestor {
                    dirty.insert(folder, true);
                } else {
                    dirty.insert(path, true);
                    dirty.insert(tree.path(&sibling), true);
                }
            }
        }
        5 => {
            let folders: BTreeSet<PathBuf> = listed.iter().filter_map(|track| return track.parent().map(Path::to_path_buf)).filter(|folder| return *folder != tree.root).collect();
            let folder_list: Vec<&PathBuf> = folders.iter().collect();
            if let Some(folder) = folder_list.get(rng.below(folder_list.len().max(1))) {
                let mut everything: Vec<PathBuf> = vec![(*folder).clone()];
                everything.extend(listed.iter().filter(|track| return track.starts_with(folder)).cloned());
                std::fs::remove_dir_all(folder).expect("removed");
                for path in everything {
                    dirty.insert(path, true);
                }
            }
        }
        6 => {
            let folders: BTreeSet<PathBuf> = listed.iter().filter_map(|track| return track.parent().map(Path::to_path_buf)).filter(|folder| return *folder != tree.root).collect();
            let folder_list: Vec<&PathBuf> = folders.iter().collect();
            if let Some(folder) = folder_list.get(rng.below(folder_list.len().max(1))) {
                let target = world.outside.path(&format!("moved-{round}-{}", rng.below(1_000_000)));
                std::fs::rename(folder, target).expect("moved out");
                dirty.insert((*folder).clone(), true);
            }
        }
        7 => {
            let folders: BTreeSet<PathBuf> = listed.iter().filter_map(|track| return track.parent().map(Path::to_path_buf)).filter(|folder| return *folder != tree.root).collect();
            let folder_list: Vec<&PathBuf> = folders.iter().collect();
            if let Some(folder) = folder_list.get(rng.below(folder_list.len().max(1))) {
                std::fs::remove_dir_all(folder).expect("removed");
                std::fs::create_dir_all(folder).expect("recreated");
                std::fs::write(folder.join(format!("recreated{round}.flac")), "x").expect("filled");
                dirty.insert((*folder).clone(), true);
            }
        }
        _ => {
            if let Some(path) = listed.get(rng.below(listed.len().max(1))) {
                let target = path.with_file_name(format!("renamed {round} {}.flac", rng.below(1000)));
                std::fs::rename(path, &target).expect("renamed");
                dirty.insert(path.clone(), true);
                dirty.insert(target, true);
            }
        }
    }
}

/// What: `pretend_disk` is a disk that holds exactly the given files.
/// Why: the million-track tests ask the planner about a few paths only, so the pretend disk need hold only those.
fn pretend_disk(files: impl IntoIterator<Item = PathBuf>) -> MemoryDisk {
    return MemoryDisk { files: files.into_iter().collect(), asked: RefCell::new(HashMap::new()), root: PathBuf::from(crate::refresh_scale_support::ROOT) };
}

#[test]
fn a_replaced_file_in_a_million_track_queue_asks_the_disk_once_and_changes_no_list() {
    let library = crate::refresh_scale_support::million_library();
    let root = PathBuf::from(crate::refresh_scale_support::ROOT);
    let target = library[library.len() / 2].clone();
    let disk = pretend_disk([target.clone()]);
    let dirty = DirtyPaths::single(target.clone(), false);
    let plan = plan_changes(&library, &root, &dirty, &disk);
    assert_eq!(plan.touched, vec![target]);
    assert!(!plan.changes_the_list());
    let asked: HashMap<&'static str, usize> = disk.asked.borrow().clone();
    assert_eq!(asked.get("kind"), Some(&1));
    assert_eq!(asked.get("walk"), None, "no walk");
    assert_eq!(asked.get("entries"), None, "no folder read");
}

#[test]
fn a_hundred_thousand_named_paths_are_planned_and_applied_in_bounded_time() {
    let library = crate::refresh_scale_support::million_library();
    let root = PathBuf::from(crate::refresh_scale_support::ROOT);
    let mut dirty = DirtyPaths::default();
    let mut kept: BTreeSet<PathBuf> = BTreeSet::new();
    let mut expected_removed = 0_usize;
    for index in (0..library.len()).step_by(10) {
        if index % 20 == 0 {
            expected_removed += 1;
        } else {
            kept.insert(library[index].clone());
        }
        dirty.insert(library[index].clone(), true);
    }
    let disk = pretend_disk(kept);
    let started = std::time::Instant::now();
    let plan = plan_changes(&library, &root, &dirty, &disk);
    let planned = started.elapsed();
    let next = apply_plan(library.clone(), &root, &plan);
    println!("REFRESH plan of {} named paths at {} tracks: planned in {:.0} ms, applied in {:.0} ms", dirty.len(), library.len(), planned.as_secs_f64() * 1000.0, (started.elapsed() - planned).as_secs_f64() * 1000.0);
    assert_eq!(next.len(), library.len() - expected_removed);
    assert_eq!(plan.touched.len(), dirty.len() - expected_removed);
}
