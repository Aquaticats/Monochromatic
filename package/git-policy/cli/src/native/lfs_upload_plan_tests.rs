//! Controls for what the Git LFS upload sends.

use super::*;
use std::path::PathBuf;

/// An argument list from words.
fn words(items: &[&str]) -> Vec<OsString> {
    return items.iter().map(OsString::from).collect();
}

/// A planner whose Git answers come from `answers` (a query's first word, then its answer) and which counts its queries.
fn plan_in(
    directory: &Path,
    global_prefix: &[&str],
    region: &[&str],
    environment: &[(OsString, OsString)],
    answers: &[(&str, &str)],
) -> (Option<UploadPlan>, usize) {
    let mut asked: usize = 0;
    let mut query = |arguments: &[&str]| -> Option<String> {
        asked += 1;
        return answers
            .iter()
            .find(|(first, _)| {
                return *first == arguments[0];
            })
            .map(|(_, answer)| {
                return (*answer).to_owned();
            });
    };
    let planned = plan_upload(words(global_prefix).as_slice(), words(region).as_slice(), environment, directory, &mut query);
    return (planned, asked);
}

/// The same, in a directory that does not matter to the case.
fn plan_with(
    global_prefix: &[&str],
    region: &[&str],
    environment: &[(OsString, OsString)],
    answers: &[(&str, &str)],
) -> (Option<UploadPlan>, usize) {
    return plan_in(Path::new("/work"), global_prefix, region, environment, answers);
}

/// A push that names its repository and refs needs no Git query, and uploads exactly those refs to that remote.
#[test]
fn a_push_that_names_its_remote_and_refs_uploads_those_refs_without_asking_git() {
    let (planned, asked) = plan_with(&[], &["origin", "main", "topic:refs/heads/other"], &[], &[]);
    let plan: UploadPlan = planned.expect("a plan");
    assert_eq!(asked, 0);
    assert_eq!(plan.remote, OsString::from("origin"));
    assert_eq!(plan.refs, words(&["main", "topic"]));
    assert!(!plan.everything && !plan.submodules);
    let mut expected: Vec<OsString> = words(&["push"]);
    expected.push(plan.remote.clone());
    expected.extend(plan.refs.iter().cloned());
    assert_eq!(plan.arguments(), expected);
}

/// A forced refspec uploads its source, a deletion uploads nothing, and `tag <name>` is the tag's ref.
#[test]
fn forced_deleting_and_tag_refspecs_read_as_git_reads_them() {
    let (forced, _) = plan_with(&[], &["origin", "+HEAD:refs/heads/x"], &[], &[]);
    assert_eq!(forced.expect("a plan").refs, words(&["HEAD"]));
    let (deleting, _) = plan_with(&[], &["origin", ":gone"], &[], &[("symbolic-ref", "refs/heads/work")]);
    assert_eq!(deleting, None, "a refspec that only deletes sends no objects");
    let (deleted, _) = plan_with(&[], &["--delete", "origin", "gone"], &[], &[]);
    assert_eq!(deleted, None);
    let (tagged, _) = plan_with(&[], &["origin", "tag", "v1"], &[], &[]);
    assert_eq!(tagged.expect("a plan").refs, words(&["refs/tags/v1"]));
}

/// `--all` and `--tags` and glob refspecs upload the refs they stand for, never everything; `--mirror` uploads everything.
#[test]
fn patterns_are_expanded_to_the_refs_they_stand_for() {
    let heads: &str = "refs/heads/main\nrefs/heads/other";
    let (branches, _) = plan_with(&[], &["--all", "origin"], &[], &[("for-each-ref", heads)]);
    assert_eq!(branches.expect("a plan").refs, words(&["refs/heads/main", "refs/heads/other"]));
    let (tagged, _) = plan_with(&[], &["--tags", "origin"], &[], &[("for-each-ref", "refs/tags/v1")]);
    let tags: UploadPlan = tagged.expect("a plan");
    assert_eq!((tags.refs, tags.everything), (words(&["refs/tags/v1"]), false));
    let (glob, _) = plan_with(&[], &["origin", "refs/heads/*:refs/heads/*"], &[], &[("for-each-ref", heads)]);
    assert_eq!(glob.expect("a plan").refs, words(&["refs/heads/main", "refs/heads/other"]));
    let (no_tags, _) = plan_with(&[], &["--tags", "origin"], &[], &[("for-each-ref", "")]);
    assert_eq!(no_tags, None, "no tag means nothing to upload");
    let (mirrored, _) = plan_with(&[], &["--mirror", "origin"], &[], &[]);
    let mirror: UploadPlan = mirrored.expect("a plan");
    assert!(mirror.everything);
    assert_eq!(mirror.arguments(), words(&["push", "--all", "origin"]));
}

/// A bare push takes the remote and the branch from Git: the push remote, then the default push remote, then the
///  branch's remote, then `origin`; a branch without a name is `HEAD`; `push.default=matching` uploads everything.
#[test]
fn a_bare_push_resolves_its_remote_and_branch_the_way_git_does() {
    let resolve = |lines: &str| -> Option<UploadPlan> {
        return plan_with(&[], &[], &[], &[("symbolic-ref", "refs/heads/work"), ("config", lines)]).0;
    };
    let all: &str = "branch.work.remote upstream\nremote.pushdefault shared\nbranch.work.pushremote mine";
    assert_eq!(resolve(all).expect("a plan").remote, OsString::from("mine"));
    let without_push_remote: &str = "branch.work.remote upstream\nremote.pushdefault shared";
    assert_eq!(resolve(without_push_remote).expect("a plan").remote, OsString::from("shared"));
    assert_eq!(resolve("branch.work.remote upstream").expect("a plan").remote, OsString::from("upstream"));
    assert_eq!(resolve("branch.other.remote elsewhere").expect("a plan").remote, OsString::from("origin"));
    let bare: UploadPlan = resolve("").expect("a plan");
    assert_eq!(bare.refs, words(&["refs/heads/work"]));
    let detached = plan_with(&[], &[], &[], &[("config", "")]).0.expect("a plan");
    assert_eq!(detached.refs, words(&["HEAD"]));
    let matching: UploadPlan = resolve("push.default matching").expect("a plan");
    assert!(matching.everything);
}

/// A branch named like a tag is read from the full ref, so its configuration is found.
#[test]
fn the_branch_is_read_from_the_full_ref() {
    let answers = [("symbolic-ref", "refs/heads/main"), ("config", "branch.main.pushremote mine")];
    let planned = plan_with(&[], &[], &[], &answers).0.expect("a plan");
    assert_eq!((planned.remote, planned.refs), (OsString::from("mine"), words(&["refs/heads/main"])));
    let not_a_branch = plan_with(&[], &[], &[], &[("symbolic-ref", "refs/tags/main"), ("config", "")]).0.expect("a plan");
    assert_eq!(not_a_branch.refs, words(&["HEAD"]), "a symbolic ref that is not a branch is not trusted as one");
}

/// `remote.<name>.push` is what a push with no refspec sends, glob included.
#[test]
fn the_configured_push_refspecs_of_the_remote_are_what_a_bare_push_sends() {
    let config: &str = "remote.origin.push refs/heads/*:refs/heads/*\nremote.other.push ignored:refs/heads/x";
    let answers = [("symbolic-ref", "refs/heads/main"), ("config", config), ("for-each-ref", "refs/heads/main\nrefs/heads/other")];
    let bare = plan_with(&[], &[], &[], &answers).0.expect("a plan");
    assert_eq!(bare.refs, words(&["refs/heads/main", "refs/heads/other"]));
    let named = plan_with(&[], &["origin"], &[], &answers).0.expect("a plan");
    assert_eq!(named.refs, bare.refs, "a remote with no refspec uses the same configuration");
    let mapped = plan_with(&[], &["origin"], &[], &[("config", "remote.origin.push other:refs/heads/mapped")]).0.expect("a plan");
    assert_eq!(mapped.refs, words(&["other"]), "the source side is uploaded, not the destination");
    let explicit = plan_with(&[], &["origin", "main"], &[], &answers).0.expect("a plan");
    assert_eq!(explicit.refs, words(&["main"]), "a refspec typed on the command line wins over the configuration");
}

/// The first positional is the repository whatever `--repo` says, `--repo` and its abbreviations fill in only when none
///  is typed, and a remote only takes the branch Git would send.
#[test]
fn the_repository_is_read_as_git_reads_it() {
    let (positional, _) = plan_with(&[], &["--repo=second", "origin", "main"], &[], &[]);
    let typed: UploadPlan = positional.expect("a plan");
    assert_eq!((typed.remote, typed.refs), (OsString::from("origin"), words(&["main"])));
    for spelling in ["--repo=elsewhere", "--rep=elsewhere"] {
        let (option_only, _) = plan_with(&[], &[spelling, "--force"], &[], &[("symbolic-ref", "refs/heads/work"), ("config", "")]);
        let by_option: UploadPlan = option_only.expect("a plan");
        assert_eq!(by_option.remote, OsString::from("elsewhere"), "{spelling}");
    }
    let (named, _) = plan_with(&[], &["origin"], &[], &[("symbolic-ref", "refs/heads/work"), ("config", "")]);
    let remote_only: UploadPlan = named.expect("a plan");
    assert_eq!((remote_only.remote, remote_only.refs), (OsString::from("origin"), words(&["refs/heads/work"])));
}

/// A path typed as the remote becomes a `file://` URL, which `git-lfs` accepts; a remote name and an URL are untouched.
#[test]
fn a_path_remote_becomes_a_file_url() {
    let directory: PathBuf = PathBuf::from("/srv/work");
    let remote_of = |typed: &str| -> OsString {
        return plan_in(directory.as_path(), &[], &[typed, "main"], &[], &[]).0.expect("a plan").remote;
    };
    assert_eq!(remote_of("/srv/origin.git"), OsString::from("file:///srv/origin.git"));
    assert_eq!(remote_of("../origin.git"), OsString::from("file:///srv/work/../origin.git"));
    assert_eq!(remote_of("origin"), OsString::from("origin"));
    assert_eq!(remote_of("https://example.invalid/r.git"), OsString::from("https://example.invalid/r.git"));
    assert_eq!(remote_of("git@example.invalid:r.git"), OsString::from("git@example.invalid:r.git"));
}

/// A push that also pushes submodules is marked, because their objects are not part of the upload.
#[test]
fn a_push_that_recurses_into_submodules_is_marked() {
    for (value, marked) in [("on-demand", true), ("only", true), ("check", false), ("no", false)] {
        let option: String = format!("--recurse-submodules={value}");
        let (planned, _) = plan_with(&[], &[option.as_str(), "origin", "main"], &[], &[]);
        assert_eq!(planned.expect("a plan").submodules, marked, "{value}");
    }
}

/// Every way of bypassing the upload plans nothing, the environment's pairs and the global option included.
#[test]
fn bypasses_and_dry_runs_plan_no_upload() {
    for region in [vec!["--no-verify", "origin", "main"], vec!["--no-ver", "origin", "main"], vec!["--dry-run", "origin", "main"]] {
        assert_eq!(plan_with(&[], region.as_slice(), &[], &[]).0, None, "{region:?}");
    }
    let off: Vec<&str> = vec!["-c", "hook.cli-git-lfs-pre-push.enabled=false"];
    assert_eq!(plan_with(off.as_slice(), &["origin", "main"], &[], &[]).0, None);
    let environment = |pairs: &[(&str, &str)]| -> Vec<(OsString, OsString)> {
        return pairs
            .iter()
            .map(|(name, value)| {
                return (OsString::from(name), OsString::from(value));
            })
            .collect();
    };
    let skip = environment(&[("GIT_LFS_SKIP_PUSH", "1")]);
    assert_eq!(plan_with(&[], &["origin", "main"], skip.as_slice(), &[]).0, None);
    let pairs = environment(&[
        ("GIT_CONFIG_COUNT", "1"),
        ("GIT_CONFIG_KEY_0", "hook.cli-git-lfs-pre-push.enabled"),
        ("GIT_CONFIG_VALUE_0", "false"),
    ]);
    assert_eq!(plan_with(&[], &["origin", "main"], pairs.as_slice(), &[]).0, None, "the environment's pairs switch it off too");
    assert_eq!(plan_with(&[], &["--no-such-option"], &[], &[]).0, None, "a region Git refuses is left to Git");
}
