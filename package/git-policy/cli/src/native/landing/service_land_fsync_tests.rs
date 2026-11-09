//! Controls for the flushes Git's own children make while a landing imports its commits:
//! native Git writes the objects of a
//!  commit as loose objects and flushes none by default,
//! so the import flushes nothing the repository's `core.fsync` does not
//!  ask for.
//! The flushes are counted from Git's trace2 `fsync` counters,
//! whatever child makes them.

use super::tests::{World, request, world};
use super::*;
use crate::test_support::{git, git_text, remove};

/// A landing of one changed file in a repository configured with `settings`,
/// and the flushes its Git children reported.
/// Why the controls below take tens of seconds:
///  the landing's own phases are the cost, never the counting of the flushes.
/// Measured on 2026-11-09 with
///  this host at a load average of 62:
/// one landing took 107.9 s,
/// of which the index build and install took 57.8 s,
/// the journal write 20.6 s,
/// the objects 8.6 s and the index lock 5.5 s,
/// while draining the flushes the landing queued took 0.114 ms
///  and the three Git checks that follow it 5 to 20 ms.
/// Those phases are the same writes and the same flushes a native commit makes,
///  `core.fsync` deciding which,
/// so a commit on such a host waits for the same disk.
fn landing_flushes(name: &str, settings: &[(&str, &str)]) -> (u64, World) {
    let mut world: World = world(name);
    for (key, value) in settings {
        git(world.repo.as_path(), &["config", key, value]);
    }
    let trace: PathBuf = world.root.join("trace.json");
    world.context.git.overlay.push((std::ffi::OsString::from("GIT_TRACE2_EVENT"), trace.clone().into_os_string()));
    let landed: CommitRequest = request(&world, "landed", "landed.txt", "landed\n");
    let outcome = land_batch(&world.context, 1, &[landed]).expect("landed");
    assert!(matches!(outcome.as_slice(), [Outcome::Landed { .. }]), "{outcome:?}");
    crate::service_durable::drain();
    let events: String = std::fs::read_to_string(&trace).unwrap_or_default();
    let flushes: u64 = events
        .lines()
        .filter(|line| { return line.contains("\"event\":\"counter\"") && line.contains("\"category\":\"fsync\""); })
        .filter_map(|line| { return line.split("\"count\":").nth(1)?.trim_end_matches('}').parse::<u64>().ok(); })
        .sum();
    return (flushes, world);
}

/// The commit is on the branch with its content,
/// and the repository is whole.
fn assert_landed_whole(world: &World) {
    assert_eq!(git_text(world.repo.as_path(), &["show", "HEAD:landed.txt"]), "landed");
    assert_eq!(git_text(world.repo.as_path(), &["rev-list", "--objects", "--all", "--missing=print"]).lines().filter(|line| { return line.starts_with('?'); }).count(), 0, "no object is missing");
    git(world.repo.as_path(), &["fsck", "--strict", "--no-dangling"]);
}

/// A small batch is imported as loose objects and the temporary pack is deleted,
/// so the flush of that pack protects nothing:
/// under Git's default `core.fsync` a landing's Git children flush nothing,
/// as a native commit flushes nothing.
#[test]
fn a_landing_flushes_no_pack_the_import_deletes() {
    let (flushes, world) = landing_flushes("service-land-fsync-default", &[]);
    assert_eq!(flushes, 0, "the import's pack is a temporary file");
    assert_landed_whole(&world);
    remove(world.root.as_path());
}

/// A person who asks for flushed objects keeps them:
/// a repository that names loose objects,
/// `added`,
/// `all` or the deprecated
/// object-file switch gets the flushes Git makes for it.
/// These are the positive controls of the control above.
#[test]
fn a_repository_that_flushes_loose_objects_keeps_the_flushes_of_the_import() {
    let cases: [(&str, [(&str, &str); 1]); 4] = [
        ("loose-object", [("core.fsync", "loose-object")]),
        ("added", [("core.fsync", "added")]),
        ("all", [("core.fsync", "all")]),
        ("object-files", [("core.fsyncObjectFiles", "true")]),
    ];
    for (label, settings) in cases {
        let (flushes, world) = landing_flushes(format!("service-land-fsync-{label}").as_str(), &settings);
        assert!(flushes > 0, "{label}: the import flushes what the repository asks for");
        assert_landed_whole(&world);
        remove(world.root.as_path());
    }
}

/// A batch above the unpack limit keeps its pack:
/// with the default `core.fsync` it is not flushed,
/// as native Git flushes no loose
/// object,
/// and when the repository flushes loose objects it is.
#[test]
fn a_kept_pack_is_flushed_only_where_loose_objects_are() {
    let (plain, plain_world) = landing_flushes("service-land-fsync-kept-default", &[("fastimport.unpackLimit", "0")]);
    assert_eq!(plain, 0, "a kept pack follows the repository, which asks for no flush of objects");
    let packs: usize = std::fs::read_dir(plain_world.repo.join(".git/objects/pack")).expect("pack directory").flatten().filter(|entry| { return entry.file_name().to_string_lossy().ends_with(".pack"); }).count();
    assert_eq!(packs, 1, "the import kept its pack");
    assert_landed_whole(&plain_world);
    remove(plain_world.root.as_path());
    let (flushed, flushed_world) = landing_flushes("service-land-fsync-kept-loose", &[("fastimport.unpackLimit", "0"), ("core.fsync", "loose-object")]);
    assert!(flushed > 0, "a repository that flushes loose objects flushes the kept pack");
    assert_landed_whole(&flushed_world);
    remove(flushed_world.root.as_path());
}

/// Where native Git would harden the objects of a commit (loose objects named in `core.fsync`,
/// or the deprecated object-file
/// switch),
/// a pack that the import keeps instead of unpacking is hardened too,
/// with its index:
/// the person's choice covers the
/// same objects whichever way they are stored.
/// Each case keeps its pack (`fastimport.unpackLimit=0`) and names loose objects
/// without naming the packs,
/// which is where Git's own importer would leave the kept pack unflushed.
#[test]
fn a_kept_pack_gets_the_flush_native_gives_the_same_objects() {
    let cases: [(&str, [(&str, &str); 2]); 3] = [
        ("loose-only", [("fastimport.unpackLimit", "0"), ("core.fsync", "loose-object,none")]),
        ("objects-only", [("fastimport.unpackLimit", "0"), ("core.fsync", "objects,none")]),
        ("object-files", [("fastimport.unpackLimit", "0"), ("core.fsyncObjectFiles", "true")]),
    ];
    for (label, settings) in cases {
        let mut all: Vec<(&str, &str)> = settings.to_vec();
        if label == "object-files" {
            all.push(("core.fsync", "none"));
        }
        let (flushes, world) = landing_flushes(format!("service-land-fsync-kept-{label}").as_str(), all.as_slice());
        let packs: usize = std::fs::read_dir(world.repo.join(".git/objects/pack")).expect("pack directory").flatten().filter(|entry| { return entry.file_name().to_string_lossy().ends_with(".pack"); }).count();
        assert_eq!(packs, 1, "{label}: the import kept its pack");
        assert!(flushes >= 2, "{label}: the kept pack and its index are flushed ({flushes} flushes)");
        assert_landed_whole(&world);
        remove(world.root.as_path());
    }
}
