//! The tests of the Podman cleanup:
//! an unused unnamed old image is removed,
//! and one a container uses (in any state),
//! one a current source names,
//! one within the grace period,
//! and the newest of its repository stay.
//! A multi-tag image is asked for by its tags,
//! an image that went with another one in the same run counts as gone,
//! an image a container holds is kept rather than reported as a failure,
//! and the container list asks for external (Buildah working) containers.

use std::{cell::RefCell, collections::{HashMap, HashSet}, time::Duration};

use crate::lean_container_references::{collect_references, words_of};
use crate::lean_identity::Deadline;
use crate::lean_podman_cleanup::{Image, PodmanApi, PodmanOutcome, Volume, clean_podman, judge_images, judge_volumes};
use crate::lean_podman_real::{CONTAINER_LIST_ARGUMENTS, parse_images, parse_volumes};
use crate::lean_test_support::{DAY, NOW};
use crate::test_scratch::scratch;

/// The grace period of these tests.
const GRACE: i64 = 7 * DAY;

/// A Podman that holds fixtures and records what it was asked to remove.
struct Fake {
    /// The image rows.
    images: Vec<Image>,
    /// The containers' image ids.
    containers: Vec<String>,
    /// The dangling volumes.
    volumes: Vec<Volume>,
    /// The refusal Podman gives for a removal whose first reference is this key.
    refusals: HashMap<String, String>,
    /// The references each image removal was given.
    removal_calls: RefCell<Vec<Vec<String>>>,
    /// Volume names asked for.
    removed_volumes: RefCell<Vec<String>>,
    /// How long each removal takes,
    /// so a test can put the run's deadline inside the loop.
    pause: Duration,
}

impl Fake {
    /// A Podman with these image rows and no container.
    fn with_images(images: Vec<Image>) -> Fake {
        return Fake { images, containers: Vec::new(), volumes: Vec::new(), refusals: HashMap::new(), removal_calls: RefCell::new(Vec::new()), removed_volumes: RefCell::new(Vec::new()), pause: Duration::ZERO };
    }
}

impl PodmanApi for Fake {
    fn images(&self) -> Result<Vec<Image>, String> {
        return Ok(self.images.clone());
    }
    fn container_image_ids(&self) -> Result<Vec<String>, String> {
        return Ok(self.containers.clone());
    }
    fn dangling_volumes(&self) -> Result<Vec<Volume>, String> {
        return Ok(self.volumes.clone());
    }
    fn remove_image(&self, references: &[String]) -> Result<(), String> {
        self.removal_calls.borrow_mut().push(references.to_vec());
        std::thread::sleep(self.pause);
        return match references.first() {
            Some(first) => match self.refusals.get(first) {
                Some(refusal) => Err(refusal.clone()),
                None => Ok(()),
            },
            None => Ok(()),
        };
    }
    fn remove_volume(&self, name: &str) -> Result<(), String> {
        self.removed_volumes.borrow_mut().push(name.to_string());
        std::thread::sleep(self.pause);
        return Ok(());
    }
}

/// A deadline these tests are nowhere near reaching,
/// for the cases that are not about the deadline.
fn untouched() -> Deadline {
    return Deadline::after(Duration::from_secs(3_600));
}

/// An image row.
fn image(id: &str, repository: &str, tag: &str, idle: i64) -> Image {
    return Image { id: id.to_string(), repository: repository.to_string(), tag: tag.to_string(), created: NOW - idle };
}

/// The words of a piece of source text.
fn references(text: &str) -> HashSet<String> {
    let mut words: HashSet<String> = HashSet::new();
    words_of(text, &mut words);
    return words;
}

#[test]
fn an_unused_unnamed_old_image_is_removed_and_the_newest_of_its_repository_stays() {
    let images: Vec<Image> = vec![image("aaa111", "localhost/gate", "s1", 30 * DAY), image("bbb222", "localhost/gate", "s2", 20 * DAY), image("ccc333", "localhost/gate", "s3", 10 * DAY)];
    let (remove, keep) = judge_images(&images, &[], &references("localhost/gate:${tag}"), GRACE, NOW);
    assert_eq!(remove, vec![String::from("aaa111"), String::from("bbb222")]);
    assert!(keep.iter().any(|(id, why)| { return id == "ccc333" && why.contains("newest") }));
}

#[test]
fn an_image_a_container_uses_in_any_state_is_kept() {
    let images: Vec<Image> = vec![image("aaa111", "localhost/gate", "s1", 30 * DAY), image("zzz999", "localhost/gate", "s9", 1 * DAY)];
    let (remove, keep) = judge_images(&images, &[String::from("aaa111ffff")], &HashSet::new(), GRACE, NOW);
    assert!(remove.is_empty(), "{remove:?}");
    assert!(keep.iter().any(|(id, why)| { return id == "aaa111" && why.contains("container") }));
}

#[test]
fn a_tag_the_sources_spell_out_keeps_the_image_and_a_templated_one_does_not() {
    let images: Vec<Image> = vec![
        image("aaa111", "localhost/ide", "latest", 60 * DAY),
        image("bbb222", "localhost/ide", "dev", 50 * DAY),
        image("ccc333", "localhost/ide", "newer", 40 * DAY),
    ];
    let named: HashSet<String> = references("run: podman build -t localhost/ide:dev .\nbare: localhost/ide\n");
    let (remove, keep) = judge_images(&images, &[], &named, GRACE, NOW);
    assert!(remove.is_empty(), "latest by the bare repository, dev by name, newer as the newest: {remove:?} {keep:?}");
    let (remove_templated, _) = judge_images(&images, &[], &references("localhost/ide:${tag}"), GRACE, NOW);
    assert_eq!(remove_templated, vec![String::from("aaa111"), String::from("bbb222")]);
}

#[test]
fn an_image_within_the_grace_period_and_an_untagged_unused_old_one() {
    let images: Vec<Image> = vec![image("aaa111", "<none>", "<none>", 2 * DAY), image("bbb222", "<none>", "<none>", 30 * DAY)];
    let (remove, keep) = judge_images(&images, &[], &HashSet::new(), GRACE, NOW);
    assert_eq!(remove, vec![String::from("bbb222")]);
    assert!(keep.iter().any(|(id, why)| { return id == "aaa111" && why.contains("grace") }));
}

#[test]
fn volumes_a_task_names_or_that_are_young_are_kept_and_the_rest_go() {
    let volumes: Vec<Volume> = vec![
        Volume { name: String::from("ide-cargo"), created: NOW - 90 * DAY },
        Volume { name: String::from("0123abcd0123abcd"), created: NOW - 90 * DAY },
        Volume { name: String::from("fresh-volume"), created: NOW - DAY },
    ];
    let (remove, keep) = judge_volumes(&volumes, &references("-v ide-cargo:/cargo"), GRACE, NOW);
    assert_eq!(remove, vec![String::from("0123abcd0123abcd")]);
    assert_eq!(keep.len(), 2);
}

#[test]
fn a_cleanup_removes_each_judged_image_logs_each_proof_and_a_dry_run_asks_for_nothing() {
    let rows: Vec<Image> = (0..60).map(|index| { return image(&format!("id{index:04}"), "<none>", "<none>", 30 * DAY); }).collect();
    let fake: Fake = Fake::with_images(rows);
    let mut log: Vec<String> = Vec::new();
    let dry: PodmanOutcome = clean_podman(&fake, &HashSet::new(), GRACE, NOW, true, &untouched(), &mut log);
    assert_eq!(dry.images.len(), 60);
    assert!(fake.removal_calls.borrow().is_empty());
    let real: PodmanOutcome = clean_podman(&fake, &HashSet::new(), GRACE, NOW, false, &untouched(), &mut log);
    assert_eq!(real.images.len(), 60);
    assert_eq!(fake.removal_calls.borrow().len(), 60);
    assert!(log.iter().any(|line| { return line.starts_with("removing image id0000 (id0000): podman listed 60 image rows and 0 containers"); }));
}

#[test]
fn a_multi_tag_image_is_removed_through_every_tag_and_an_untagged_one_by_its_id() {
    // Measured 2026-11-08 in a throwaway Podman root (podman 6.1.3): `podman rmi <id>` refuses an image with more than one tag
    // ("unable to delete image ... by ID with more than one tag"), and `podman rmi <tag> <tag>` untags both and deletes it.
    let rows: Vec<Image> = vec![
        image("aaa111", "localhost/gate", "one", 30 * DAY),
        image("aaa111", "localhost/gate", "two", 30 * DAY),
        image("zzz999", "localhost/gate", "new", 2 * DAY),
        image("bbb222", "<none>", "<none>", 30 * DAY),
    ];
    let fake: Fake = Fake::with_images(rows);
    let mut log: Vec<String> = Vec::new();
    let outcome: PodmanOutcome = clean_podman(&fake, &HashSet::new(), GRACE, NOW, false, &untouched(), &mut log);
    assert_eq!(outcome.images, vec![String::from("aaa111"), String::from("bbb222")]);
    assert_eq!(*fake.removal_calls.borrow(), vec![vec![String::from("localhost/gate:one"), String::from("localhost/gate:two")], vec![String::from("bbb222")]]);
}

#[test]
fn an_image_that_went_with_another_one_counts_as_gone_and_one_a_container_holds_as_kept() {
    // Measured refusals of podman 6.1.3 (2026-11-08), verbatim from the first run's log (`~/temp/agent/lean/first-podman.out`).
    let rows: Vec<Image> = vec![image("ccc333", "<none>", "<none>", 30 * DAY), image("ddd444", "<none>", "<none>", 30 * DAY), image("eee555", "<none>", "<none>", 30 * DAY)];
    let mut fake: Fake = Fake::with_images(rows);
    fake.refusals.insert(String::from("ccc333"), String::from("Error: ccc333: image not known"));
    fake.refusals.insert(
        String::from("ddd444"),
        String::from("Error: image used by 1a305ceb3f6f19964bb840be1088592f0686be71c897eca0d77f73b75284a273: image is in use by a container: consider listing external containers and force-removing image"),
    );
    fake.refusals.insert(String::from("eee555"), String::from("Error: eee555: something this tool has no reading for"));
    let mut log: Vec<String> = Vec::new();
    let outcome: PodmanOutcome = clean_podman(&fake, &HashSet::new(), GRACE, NOW, false, &untouched(), &mut log);
    assert_eq!(outcome.gone_with_another, vec![String::from("ccc333")], "{} failures", outcome.failures.len());
    assert!(outcome.kept_images.iter().any(|(id, why)| { return id == "ddd444" && why.contains("container"); }), "{:?}", outcome.kept_images);
    assert_eq!(outcome.failures.len(), 1, "{:?}", outcome.failures);
    assert!(outcome.failures[0].contains("eee555"), "{:?}", outcome.failures);
}

#[test]
fn the_container_list_asks_podman_for_external_builders() {
    // A Buildah working container holds its image but does not appear in `podman ps --all`; measured 2026-11-08 in a throwaway root
    // (buildah 1.45.1, podman 6.1.3): `ps --external` lists it, `--all --external` lists it beside the ordinary containers.
    assert!(CONTAINER_LIST_ARGUMENTS.contains(&"--all"), "{CONTAINER_LIST_ARGUMENTS:?}");
    assert!(CONTAINER_LIST_ARGUMENTS.contains(&"--external"), "{CONTAINER_LIST_ARGUMENTS:?}");
}

#[test]
fn podmans_json_lists_parse_into_rows() {
    let images: &str = r#"[{"Id":"296e28a8520355e813","Created":1794110226,"Names":["localhost/git-policy-native-test:development"]},{"Id":"ec92d69233e49a7fc3","Created":1794110000,"Names":null}]"#;
    let rows: Vec<Image> = parse_images(images).expect("parses");
    assert_eq!(rows[0], Image { id: String::from("296e28a85203"), repository: String::from("localhost/git-policy-native-test"), tag: String::from("development"), created: 1_794_110_226 });
    assert_eq!(rows[1].repository, "<none>");
    let volumes: Vec<Volume> = parse_volumes(r#"[{"Name":"ide-cargo","CreatedAt":"2026-07-05T19:36:40.736210785-04:00"}]"#).expect("parses");
    assert_eq!(volumes[0].name, "ide-cargo");
    assert_eq!(volumes[0].created, 1_783_294_600);
}

#[test]
fn the_source_scan_reads_build_files_and_skips_dependency_and_documentation_directories() {
    let root = scratch();
    std::fs::create_dir_all(root.path().join("package").join("x")).expect("package");
    std::fs::create_dir_all(root.path().join("node_modules").join("y")).expect("modules");
    std::fs::create_dir_all(root.path().join("doc")).expect("doc");
    std::fs::write(root.path().join("package").join("x").join("mise.toml"), "run = \"podman run localhost/x:current -v x-cargo:/cargo\"").expect("mise");
    std::fs::write(root.path().join("node_modules").join("y").join("a.ts"), "localhost/dependency:old").expect("dependency");
    std::fs::write(root.path().join("doc").join("note.toml"), "localhost/doc:old").expect("doc file");
    let words: HashSet<String> = collect_references(root.path(), &Deadline::after(std::time::Duration::from_secs(30))).expect("finished");
    assert!(words.contains("localhost/x:current") && words.contains("x-cargo"));
    assert!(!words.contains("localhost/dependency:old") && !words.contains("localhost/doc:old"));
}

/// What:
/// `fn images_and_a_volume(count: usize) -> (Vec<Image>, Vec<Volume>)` is this many unused untagged old images and one dangling old volume.
/// Why:
/// The loop over removals is what the deadline bounds,
/// so the fixtures are long enough to see where it stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function imagesAndAVolume(count: number): [Image[], Volume[]]
/// ```
fn images_and_a_volume(count: usize) -> (Vec<Image>, Vec<Volume>) {
    let rows: Vec<Image> = (0..count).map(|index| { return image(&format!("id{index:04}"), "<none>", "<none>", 30 * DAY); }).collect();
    let volumes: Vec<Volume> = vec![Volume { name: String::from("dangling-old"), created: NOW - 30 * DAY }];
    return (rows, volumes);
}

/// A cleanup whose deadline has passed removes nothing and says it was cut:
/// no `podman rmi` and no `podman volume rm` is started after the run's bound.
#[test]
fn a_cleanup_with_no_time_left_removes_nothing_and_says_it_was_cut() {
    let (rows, volumes) = images_and_a_volume(3);
    let mut fake = Fake::with_images(rows);
    fake.volumes = volumes;
    let mut log: Vec<String> = Vec::new();
    let outcome: PodmanOutcome = clean_podman(&fake, &HashSet::new(), GRACE, NOW, false, &Deadline::after(Duration::ZERO), &mut log);
    assert!(outcome.images.is_empty(), "{outcome:?}");
    assert!(outcome.volumes.is_empty(), "{outcome:?}");
    assert!(outcome.cut, "the cleanup reports the bound it stopped at: {outcome:?}");
    assert!(fake.removal_calls.borrow().is_empty(), "no image removal is started after the deadline: {:?}", fake.removal_calls.borrow());
    assert!(fake.removed_volumes.borrow().is_empty(), "no volume removal is started after the deadline: {:?}", fake.removed_volumes.borrow());
}

/// A deadline that falls inside the loop stops it at the next removal:
/// what was removed is the front of the judged order,
/// what is left of it waits for the next run,
/// and the cleanup says it was cut rather than finished.
#[test]
fn a_deadline_that_passes_inside_the_loop_stops_it_at_the_next_removal() {
    let (rows, _volumes) = images_and_a_volume(60);
    let expected: Vec<String> = rows.iter().map(|row| { return row.id.clone(); }).collect();
    let mut fake = Fake::with_images(rows);
    fake.pause = Duration::from_millis(40);
    let mut log: Vec<String> = Vec::new();
    let outcome: PodmanOutcome = clean_podman(&fake, &HashSet::new(), GRACE, NOW, false, &Deadline::after(Duration::from_secs(1)), &mut log);
    assert!(outcome.cut, "a loop the deadline stopped says it was cut: {outcome:?}");
    assert!(!outcome.images.is_empty(), "at least the first removal happens before a one-second deadline: {outcome:?}");
    assert!(outcome.images.len() < expected.len(), "sixty removals of 40 ms cannot all fit in one second: {outcome:?}");
    assert_eq!(outcome.images, expected[..outcome.images.len()].to_vec(), "the loop stopped at a removal, it did not skip around");
}

/// A cleanup with time left finishes everything it judged and is not reported as cut.
#[test]
fn a_cleanup_with_time_left_removes_everything_and_is_not_cut() {
    let (rows, volumes) = images_and_a_volume(2);
    let mut fake = Fake::with_images(rows);
    fake.volumes = volumes;
    let mut log: Vec<String> = Vec::new();
    let outcome: PodmanOutcome = clean_podman(&fake, &HashSet::new(), GRACE, NOW, false, &untouched(), &mut log);
    assert_eq!(outcome.images, vec![String::from("id0000"), String::from("id0001")], "{outcome:?}");
    assert_eq!(outcome.volumes, vec![String::from("dangling-old")], "{outcome:?}");
    assert!(!outcome.cut, "{outcome:?}");
}

/// A dry run with no time left judges nothing and says it was cut,
/// so a report never claims what a real run would not have reached.
#[test]
fn a_dry_run_with_no_time_left_judges_nothing_and_says_it_was_cut() {
    let (rows, volumes) = images_and_a_volume(2);
    let mut fake = Fake::with_images(rows);
    fake.volumes = volumes;
    let mut log: Vec<String> = Vec::new();
    let outcome: PodmanOutcome = clean_podman(&fake, &HashSet::new(), GRACE, NOW, true, &Deadline::after(Duration::ZERO), &mut log);
    assert!(outcome.images.is_empty() && outcome.volumes.is_empty(), "{outcome:?}");
    assert!(outcome.cut, "{outcome:?}");
}
