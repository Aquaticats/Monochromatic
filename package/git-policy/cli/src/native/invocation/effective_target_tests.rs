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
    EffectiveTarget, ToolCacheDirectory, cargo_build_script_output_dir, classify_effective_target,
    default_allowed_worktree_dirs, is_allowed_worktree_dir, matching_allowed_worktree_dir,
    matching_allowed_worktree_dir_for_repository, mise_data_dir, uv_cache_dir,
};
use crate::test_support::{fixture, remove};
use crate::worktree_identity::WorktreeIdentity;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// One uv or mise directory as the variable named it.
fn named(path: &str, named_by: &'static str) -> Option<ToolCacheDirectory> {
    return Some(ToolCacheDirectory {
        path: PathBuf::from(path),
        named_by,
    });
}

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
            named("/explicit", "UV_CACHE_DIR"),
        ),
        (
            vec![("UV_CACHE_DIR", ""), ("XDG_CACHE_HOME", "/xdg")],
            named("/xdg/uv", "XDG_CACHE_HOME"),
        ),
        (vec![("XDG_CACHE_HOME", "/xdg")], named("/xdg/uv", "XDG_CACHE_HOME")),
        (
            vec![("UV_CACHE_DIR", ""), ("XDG_CACHE_HOME", "")],
            named("/home/fixture/.cache/uv", "HOME"),
        ),
        (vec![], named("/home/fixture/.cache/uv", "HOME")),
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
        named("/explicit", "UV_CACHE_DIR")
    );
    assert_eq!(
        default_allowed_worktree_dirs(&[], home),
        vec![
            ToolCacheDirectory {
                path: PathBuf::from("/home/fixture/.cache/uv"),
                named_by: "HOME",
            },
            ToolCacheDirectory {
                path: PathBuf::from("/home/fixture/.local/share/mise"),
                named_by: "HOME",
            },
        ]
    );
    assert_eq!(
        default_allowed_worktree_dirs(&[], None),
        Vec::<ToolCacheDirectory>::new()
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
            named("/explicit", "MISE_DATA_DIR"),
        ),
        (
            vec![("MISE_DATA_DIR", ""), ("XDG_DATA_HOME", "/xdg")],
            named("/xdg/mise", "XDG_DATA_HOME"),
        ),
        (vec![("XDG_DATA_HOME", "/xdg")], named("/xdg/mise", "XDG_DATA_HOME")),
        (
            vec![("MISE_DATA_DIR", ""), ("XDG_DATA_HOME", "")],
            named("/home/fixture/.local/share/mise", "HOME"),
        ),
        (vec![], named("/home/fixture/.local/share/mise", "HOME")),
        // The uv variables are not mise's.
        (
            vec![("UV_CACHE_DIR", "/uv"), ("XDG_CACHE_HOME", "/cache")],
            named("/home/fixture/.local/share/mise", "HOME"),
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
        named("/explicit", "MISE_DATA_DIR")
    );
    assert_eq!(
        mise_data_dir(environment(&[("XDG_DATA_HOME", "/xdg")]).as_slice(), None),
        named("/xdg/mise", "XDG_DATA_HOME")
    );
    // Both tools' directories are allowlisted together, each from its own variables.
    assert_eq!(
        default_allowed_worktree_dirs(
            environment(&[("UV_CACHE_DIR", "/uv"), ("MISE_DATA_DIR", "/mise")]).as_slice(),
            home
        ),
        vec![
            ToolCacheDirectory {
                path: PathBuf::from("/uv"),
                named_by: "UV_CACHE_DIR",
            },
            ToolCacheDirectory {
                path: PathBuf::from("/mise"),
                named_by: "MISE_DATA_DIR",
            },
        ]
    );
}

/// Cargo's build-script output directory is exempt as the third tool directory:
///  it is where a crate that vendors another project clones it
///  (`libghostty-vt-sys` cloned Ghostty there),
/// and an empty value or an absent variable exempts nothing.
#[test]
fn cargo_build_script_directory_is_named_by_out_dir() {
    assert_eq!(cargo_build_script_output_dir(&[]), None);
    assert_eq!(cargo_build_script_output_dir(environment(&[("OUT_DIR", "")]).as_slice()), None);
    assert_eq!(
        cargo_build_script_output_dir(environment(&[("OUT_DIR", "/build/out")]).as_slice()),
        named("/build/out", "OUT_DIR")
    );
    // The three tool directories stand together, each from its own variable.
    let home: Option<&Path> = Some(Path::new("/home/fixture"));
    assert_eq!(
        default_allowed_worktree_dirs(environment(&[("OUT_DIR", "/build/out")]).as_slice(), home),
        vec![
            ToolCacheDirectory {
                path: PathBuf::from("/home/fixture/.cache/uv"),
                named_by: "HOME",
            },
            ToolCacheDirectory {
                path: PathBuf::from("/home/fixture/.local/share/mise"),
                named_by: "HOME",
            },
            ToolCacheDirectory {
                path: PathBuf::from("/build/out"),
                named_by: "OUT_DIR",
            },
        ]
    );
    // Cargo's directory needs no home directory, unlike the other two.
    assert_eq!(
        default_allowed_worktree_dirs(environment(&[("OUT_DIR", "/build/out")]).as_slice(), None),
        vec![ToolCacheDirectory {
            path: PathBuf::from("/build/out"),
            named_by: "OUT_DIR",
        }]
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
    let through_link: Vec<ToolCacheDirectory> = vec![
        ToolCacheDirectory {
            path: root.join("missing"),
            named_by: "UV_CACHE_DIR",
        },
        ToolCacheDirectory {
            path: root.join("cache-link/uv"),
            named_by: "XDG_CACHE_HOME",
        },
    ];
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
        &[ToolCacheDirectory {
            path: root.join("missing"),
            named_by: "MISE_DATA_DIR",
        }]
    ));
    // The matched entry is the one whose variable is reported.
    assert_eq!(
        matching_allowed_worktree_dir(&[cache.join("git/checkout/.git").as_path()], through_link.as_slice())
            .map(|found| return found.named_by),
        Some("XDG_CACHE_HOME")
    );
    remove(root.as_path());
}

/// What:
///  Each identity maps to its target,
///  and the exemption needs the worktree itself inside the same allowed directory as its Git directory.
/// Why:
///  A linked worktree keeps its Git directory in the main clone
///  (`<clone>/.git/worktrees/<name>`)
///  while its files stand wherever the person asked for them,
///  so a Git directory alone would exempt a checkout outside an allowed directory
///  and lose the uncommitted work there.
#[test]
fn identities_map_to_targets() {
    let root: PathBuf = fixture("targets");
    let cache: PathBuf = root.join("uv");
    std::fs::create_dir_all(cache.join("clone/.git/worktrees/w")).expect("cache layout");
    let allowed: Vec<ToolCacheDirectory> = vec![ToolCacheDirectory {
        path: cache.clone(),
        named_by: "UV_CACHE_DIR",
    }];
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
    // A linked worktree of the cached clone that the build step itself keeps inside the cache.
    let cached_linked: WorktreeIdentity = WorktreeIdentity::LinkedWorktree {
        common_dir: cache.join("clone/.git"),
        git_dir: cache.join("clone/.git/worktrees/w"),
        worktree_root: cache.join("clone/worktrees/w"),
    };
    // The same clone's linked worktree checked out where the person works: its Git directory is still
    //  inside the cache, and its files are not.
    let linked_outside_the_cache: WorktreeIdentity = WorktreeIdentity::LinkedWorktree {
        common_dir: cache.join("clone/.git"),
        git_dir: cache.join("clone/.git/worktrees/w"),
        worktree_root: root.join("the-persons-worktree"),
    };
    // The mirror image: files inside the cache, Git directory outside it.
    let linked_with_a_git_directory_outside_the_cache: WorktreeIdentity = WorktreeIdentity::LinkedWorktree {
        common_dir: PathBuf::from("/r/.git"),
        git_dir: PathBuf::from("/r/.git/worktrees/w"),
        worktree_root: cache.join("clone/worktrees/w"),
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
        (&linked_outside_the_cache, EffectiveTarget::LinkedWorktree),
        (
            &linked_with_a_git_directory_outside_the_cache,
            EffectiveTarget::LinkedWorktree,
        ),
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
    // The main checkout of the cached clone is exempt through its own worktree top too, not only its Git directory.
    assert_eq!(
        matching_allowed_worktree_dir_for_repository(&cached_main, allowed.as_slice()).map(|found| return found.named_by),
        Some("UV_CACHE_DIR")
    );
    assert_eq!(
        matching_allowed_worktree_dir_for_repository(&linked_outside_the_cache, allowed.as_slice()).map(|found| return found.named_by),
        None
    );
    remove(root.as_path());
}
