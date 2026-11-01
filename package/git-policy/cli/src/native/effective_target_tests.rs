//! What:
//!  Controls for worktree target classification and the tool-cache allowlist.
//! Why:
//!  An exemption must match whole path segments through symbolic links and must
//!      never widen to sibling directories.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(classifyEffectiveTarget(linkedIdentity, [])).toBe('linked-worktree');
//! ```
#![cfg(unix)]

/// Import the classification under test and shared fixtures.
use super::{
    EffectiveTarget, classify_effective_target, default_allowed_worktree_dirs,
    is_allowed_worktree_dir, mise_data_dir, uv_cache_dir,
};
use crate::test_support::{fixture, remove};
use crate::worktree_identity::WorktreeIdentity;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Build an environment list from text pairs.
fn environment(pairs: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
    let mut result: Vec<(OsString, OsString)> = Vec::<(OsString, OsString)>::new();
    for (name, value) in pairs {
        result.push((OsString::from(name), OsString::from(value)));
    }
    return result;
}

/// uv's own precedence applies:
///  explicit directory,
///  then XDG cache,
///  then the home cache;
///  empty is unset.
#[test]
fn uv_cache_directory_follows_uv_precedence() {
    let home: Option<&Path> = Some(Path::new("/home/fixture"));
    for (pairs, expected) in [
        (
            vec![("UV_CACHE_DIR", "/explicit"), ("XDG_CACHE_HOME", "/xdg")],
            Some(PathBuf::from("/explicit")),
        ),
        (
            vec![("UV_CACHE_DIR", ""), ("XDG_CACHE_HOME", "/xdg")],
            Some(PathBuf::from("/xdg/uv")),
        ),
        (
            vec![("XDG_CACHE_HOME", "/xdg")],
            Some(PathBuf::from("/xdg/uv")),
        ),
        (
            vec![("UV_CACHE_DIR", ""), ("XDG_CACHE_HOME", "")],
            Some(PathBuf::from("/home/fixture/.cache/uv")),
        ),
        (vec![], Some(PathBuf::from("/home/fixture/.cache/uv"))),
    ] {
        assert_eq!(
            uv_cache_dir(environment(pairs.as_slice()).as_slice(), home),
            expected,
            "{pairs:?}"
        );
    }
    assert_eq!(uv_cache_dir(&[], None), None);
    assert_eq!(
        uv_cache_dir(
            environment(&[("UV_CACHE_DIR", "/explicit")]).as_slice(),
            None
        ),
        Some(PathBuf::from("/explicit"))
    );
    assert_eq!(
        default_allowed_worktree_dirs(&[], home),
        vec![
            PathBuf::from("/home/fixture/.cache/uv"),
            PathBuf::from("/home/fixture/.local/share/mise"),
        ]
    );
    assert_eq!(
        default_allowed_worktree_dirs(&[], None),
        Vec::<PathBuf>::new()
    );
}

/// mise's own precedence applies:
///  explicit directory,
///  then the XDG data directory,
///  then the home data directory;
///  empty is unset.
#[test]
fn mise_data_directory_follows_mise_precedence() {
    let home: Option<&Path> = Some(Path::new("/home/fixture"));
    for (pairs, expected) in [
        (
            vec![("MISE_DATA_DIR", "/explicit"), ("XDG_DATA_HOME", "/xdg")],
            Some(PathBuf::from("/explicit")),
        ),
        (
            vec![("MISE_DATA_DIR", ""), ("XDG_DATA_HOME", "/xdg")],
            Some(PathBuf::from("/xdg/mise")),
        ),
        (
            vec![("XDG_DATA_HOME", "/xdg")],
            Some(PathBuf::from("/xdg/mise")),
        ),
        (
            vec![("MISE_DATA_DIR", ""), ("XDG_DATA_HOME", "")],
            Some(PathBuf::from("/home/fixture/.local/share/mise")),
        ),
        (
            vec![],
            Some(PathBuf::from("/home/fixture/.local/share/mise")),
        ),
        // The uv variables are not mise's.
        (
            vec![("UV_CACHE_DIR", "/uv"), ("XDG_CACHE_HOME", "/cache")],
            Some(PathBuf::from("/home/fixture/.local/share/mise")),
        ),
    ] {
        assert_eq!(
            mise_data_dir(environment(pairs.as_slice()).as_slice(), home),
            expected,
            "{pairs:?}"
        );
    }
    assert_eq!(mise_data_dir(&[], None), None);
    assert_eq!(
        mise_data_dir(
            environment(&[("MISE_DATA_DIR", "/explicit")]).as_slice(),
            None
        ),
        Some(PathBuf::from("/explicit"))
    );
    assert_eq!(
        mise_data_dir(environment(&[("XDG_DATA_HOME", "/xdg")]).as_slice(), None),
        Some(PathBuf::from("/xdg/mise"))
    );
    // Both tools' directories are allowlisted together, each from its own variables.
    assert_eq!(
        default_allowed_worktree_dirs(
            environment(&[("UV_CACHE_DIR", "/uv"), ("MISE_DATA_DIR", "/mise")]).as_slice(),
            home
        ),
        vec![PathBuf::from("/uv"), PathBuf::from("/mise")]
    );
}

/// Containment is by whole segments through resolved links;
///  missing allowed directories drop out.
#[test]
fn allowlist_matches_whole_segments_through_links() {
    let root: PathBuf = fixture("allowlist");
    let cache: PathBuf = root.join("cache/uv");
    std::fs::create_dir_all(cache.join("git/checkout/.git")).expect("cache layout");
    std::fs::create_dir_all(root.join("cache/uv-other/.git")).expect("sibling layout");
    std::os::unix::fs::symlink(root.join("cache"), root.join("cache-link")).expect("symlink");
    let through_link: Vec<PathBuf> = vec![root.join("missing"), root.join("cache-link/uv")];
    for (candidate, expected) in [
        (cache.join("git/checkout/.git"), true),
        (cache.clone(), true),
        (root.join("cache/uv-other/.git"), false),
        (root.join("cache"), false),
        (root.clone(), false),
    ] {
        assert_eq!(
            is_allowed_worktree_dir(candidate.as_path(), through_link.as_slice()),
            expected,
            "{candidate:?}"
        );
    }
    assert!(!is_allowed_worktree_dir(cache.as_path(), &[]));
    assert!(!is_allowed_worktree_dir(
        cache.as_path(),
        &[root.join("missing")]
    ));
    remove(root.as_path());
}

/// Each identity maps to its target,
///  and an allowlisted Git directory exempts main and linked worktrees.
#[test]
fn identities_map_to_targets() {
    let root: PathBuf = fixture("targets");
    let cache: PathBuf = root.join("uv");
    std::fs::create_dir_all(cache.join("clone/.git/worktrees/w")).expect("cache layout");
    let allowed: Vec<PathBuf> = vec![cache.clone()];
    let main: WorktreeIdentity = WorktreeIdentity::MainWorktree {
        common_dir: PathBuf::from("/r/.git"),
        git_dir: PathBuf::from("/r/.git"),
        worktree_root: PathBuf::from("/r"),
    };
    let linked: WorktreeIdentity = WorktreeIdentity::LinkedWorktree {
        common_dir: PathBuf::from("/r/.git"),
        git_dir: PathBuf::from("/r/.git/worktrees/w"),
        worktree_root: PathBuf::from("/w"),
    };
    let cached_main: WorktreeIdentity = WorktreeIdentity::MainWorktree {
        common_dir: cache.join("clone/.git"),
        git_dir: cache.join("clone/.git"),
        worktree_root: cache.join("clone"),
    };
    let cached_linked: WorktreeIdentity = WorktreeIdentity::LinkedWorktree {
        common_dir: cache.join("clone/.git"),
        git_dir: cache.join("clone/.git/worktrees/w"),
        worktree_root: PathBuf::from("/elsewhere"),
    };
    let bare: WorktreeIdentity = WorktreeIdentity::BareRepository {
        common_dir: cache.join("clone/.git"),
        git_dir: cache.join("clone/.git"),
    };
    for (identity, expected) in [
        (
            &WorktreeIdentity::OutsideWorktree,
            EffectiveTarget::OutsideWorktree,
        ),
        (&bare, EffectiveTarget::OutsideWorktree),
        (&main, EffectiveTarget::MainWorktree),
        (&linked, EffectiveTarget::LinkedWorktree),
        (&cached_main, EffectiveTarget::Allowlisted),
        (&cached_linked, EffectiveTarget::Allowlisted),
    ] {
        assert_eq!(
            classify_effective_target(identity, allowed.as_slice()),
            expected,
            "{identity:?}"
        );
    }
    // Without an allowlist the cached repositories are ordinary worktrees.
    assert_eq!(
        classify_effective_target(&cached_main, &[]),
        EffectiveTarget::MainWorktree
    );
    assert_eq!(
        classify_effective_target(&cached_linked, &[]),
        EffectiveTarget::LinkedWorktree
    );
    remove(root.as_path());
}
