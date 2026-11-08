//! The tests of the Podman cleanup:
//! an unused unnamed old image is removed,
//! and one a container uses (in any state),
//! one a current source names,
//! one within the grace period,
//! and the newest of its repository stay.

use std::{cell::RefCell, collections::HashSet};

use crate::lean_container_references::{collect_references, words_of};
use crate::lean_identity::Deadline;
use crate::lean_podman_cleanup::{Image, PodmanApi, PodmanOutcome, Volume, clean_podman, judge_images, judge_volumes};
use crate::lean_podman_real::{parse_images, parse_volumes};
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
    /// Image ids asked for.
    removed_images: RefCell<Vec<String>>,
    /// Volume names asked for.
    removed_volumes: RefCell<Vec<String>>,
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
    fn remove_images(&self, ids: &[String]) -> Result<(), String> {
        self.removed_images.borrow_mut().extend(ids.iter().cloned());
        return Ok(());
    }
    fn remove_volume(&self, name: &str) -> Result<(), String> {
        self.removed_volumes.borrow_mut().push(name.to_string());
        return Ok(());
    }
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
fn a_cleanup_removes_in_batches_logs_each_proof_and_a_dry_run_asks_for_nothing() {
    let rows: Vec<Image> = (0..60).map(|index| { return image(&format!("id{index:04}"), "<none>", "<none>", 30 * DAY); }).collect();
    let fake: Fake = Fake { images: rows, containers: Vec::new(), volumes: Vec::new(), removed_images: RefCell::new(Vec::new()), removed_volumes: RefCell::new(Vec::new()) };
    let mut log: Vec<String> = Vec::new();
    let dry: PodmanOutcome = clean_podman(&fake, &HashSet::new(), GRACE, NOW, true, &mut log);
    assert_eq!(dry.images.len(), 60);
    assert!(fake.removed_images.borrow().is_empty());
    let real: PodmanOutcome = clean_podman(&fake, &HashSet::new(), GRACE, NOW, false, &mut log);
    assert_eq!(real.images.len(), 60);
    assert_eq!(fake.removed_images.borrow().len(), 60);
    assert!(log.iter().any(|line| { return line.starts_with("removing image id0000: podman listed 60 image rows and 0 containers"); }));
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
