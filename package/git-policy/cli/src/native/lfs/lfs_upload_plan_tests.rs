//! Controls for what the Git LFS upload sends,
//! including every way the plan must fail closed.

use super::*;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// An argument list from words.
fn words(items: &[&str]) -> Vec<OsString> {
    return items.iter().map(OsString::from).collect();
}

/// Configuration entries from `(key, value)` pairs.
fn config(pairs: &[(&str, &str)]) -> Settings {
    return pairs
        .iter()
        .map(|(key, value)| {
            return ((*key).to_owned(), (*value).to_owned());
        })
        .collect();
}

/// What a stubbed Git answers:
/// a query's key (its first word,
/// or `for-each-ref:<prefix>`) and the answer,
/// and the keys
///  whose queries fail.
struct Stub<'a> {
    /// Answers by key.
    answers: &'a [(&'a str, &'a str)],
    /// Keys that fail.
    failing: &'a [&'a str],
}

/// What:
///  The key a stubbed question is answered by:
/// the last argument for the three questions the plan asks about
///  names,
/// and the first word for the rest.
/// Why:
///   One stub answers both faces of the plan,
/// and the key has to say which name a question was about without
///  spelling out a whole command line per case.
fn stub_key(arguments: &[&str]) -> String {
    return match arguments[0] {
        "for-each-ref" | "ls-remote" | "config" => format!("{}:{}", arguments[0], arguments[arguments.len() - 1]),
        _ => arguments[0].to_owned(),
    };
}

/// Plan a push in `directory` with a stubbed Git;
/// the plan or the failure,
/// and how many queries were made.
fn plan_in(
    directory: &Path,
    global_prefix: &[&str],
    region: &[&str],
    environment: &[(OsString, OsString)],
    settings: &Settings,
    stub: &Stub<'_>,
) -> (Result<Option<UploadPlan>, PlanError>, usize) {
    let mut asked: usize = 0;
    let mut query = |arguments: &[&str]| -> Result<String, String> {
        asked += 1;
        let key: String = stub_key(arguments);
        if stub.failing.contains(&key.as_str()) {
            return Err(format!("{key} failed"));
        }
        return Ok(stub
            .answers
            .iter()
            .find(|(name, _)| {
                return *name == key;
            })
            .map_or(String::new(), |(_, answer)| {
                return (*answer).to_owned();
            }));
    };
    let planned = plan_upload(words(global_prefix).as_slice(), words(region).as_slice(), environment, directory, settings.as_slice(), &mut query);
    return (planned, asked);
}

/// The same,
/// in a directory that does not matter to the case,
/// expecting a plan.
fn plan_of(region: &[&str], settings: &[(&str, &str)], answers: &[(&str, &str)]) -> UploadPlan {
    let (planned, _) = plan_in(Path::new("/work"), &[], region, &[], &config(settings), &Stub { answers, failing: &[] });
    return planned.expect("the plan is worked out").expect("a plan");
}

/// The same,
/// expecting no upload.
fn no_plan(region: &[&str], settings: &[(&str, &str)], answers: &[(&str, &str)]) {
    let (planned, _) = plan_in(Path::new("/work"), &[], region, &[], &config(settings), &Stub { answers, failing: &[] });
    assert_eq!(planned.expect("the plan is worked out"), None, "{region:?}");
}

/// The first run's arguments for a destination that is not a configured remote:
/// `push --all`,
/// the destination,
/// then the refs.
#[test]
fn an_unconfigured_destination_uses_all_with_the_refs_so_no_tracking_ref_hides_an_object() {
    let plan: UploadPlan = plan_of(&["origin", "main", "topic:refs/heads/other"], &[], &[]);
    assert_eq!(plan.refs, words(&["main", "topic"]));
    let mut expected: Vec<OsString> = words(&["push", "--all"]);
    expected.push(plan.remotes[0].clone());
    expected.extend(plan.refs.iter().cloned());
    assert_eq!(plan.arguments(plan.remotes[0].as_os_str()), expected);
    let (planned, asked) = plan_in(Path::new("/work"), &[], &["origin", "main"], &[], &config(&[]), &Stub { answers: &[], failing: &[] });
    assert_eq!((planned.expect("planned").is_some(), asked), (true, 0), "named refs need no Git start");
}

/// A configured remote that is fetched from and pushed to the same place uses git-lfs's bounded form:
/// no `--all`,
/// so the
///  objects that the remote's tracking refs reach are skipped and a push that adds none sends no batch request.
#[test]
fn a_configured_remote_with_one_url_uses_the_bounded_form() {
    let configured: &[(&str, &str)] = &[("remote.origin.url", "https://example.invalid/r.git")];
    let plan: UploadPlan = plan_of(&["origin", "main", "topic:refs/heads/other"], configured, &[]);
    assert!(!plan.full_scan);
    assert_eq!(plan.arguments(plan.remotes[0].as_os_str()), words(&["push", "origin", "main", "topic"]));
    let bare: UploadPlan = plan_of(&[], configured, &[("symbolic-ref", "refs/heads/work")]);
    assert_eq!(bare.arguments(bare.remotes[0].as_os_str()), words(&["push", "origin", "refs/heads/work"]));
    let ignored: UploadPlan = plan_of(&["origin", "main"], &[configured[0], ("remote.origin.fetch", "+refs/heads/*:refs/remotes/origin/*")], &[]);
    assert!(!ignored.full_scan, "settings the plan does not read change nothing");
}

/// Every destination whose tracking refs may not describe it keeps the full scan:
/// a push URL (alone or several),
/// a
///  `pushInsteadOf` rewrite,
/// a URL or path that is not a configured remote,
/// a mirror,
/// and a remote nobody configured.
#[test]
fn a_destination_that_tracking_refs_may_not_describe_keeps_the_full_scan() {
    let url: (&str, &str) = ("remote.origin.url", "https://example.invalid/r.git");
    let full = |region: &[&str], settings: &[(&str, &str)]| -> Vec<OsString> {
        let plan: UploadPlan = plan_of(region, settings, &[]);
        assert!(plan.full_scan, "{region:?} {settings:?}");
        return plan.arguments(plan.remotes[0].as_os_str());
    };
    assert_eq!(full(&["origin", "main"], &[url, ("remote.origin.pushurl", "https://other.invalid/r.git")])[..2], words(&["push", "--all"])[..]);
    assert_eq!(full(&["origin", "main"], &[url, ("remote.origin.pushurl", "https://example.invalid/r.git")])[..2], words(&["push", "--all"])[..]);
    let two: UploadPlan = plan_of(&["origin", "main"], &[url, ("remote.origin.pushurl", "https://a.invalid/r.git"), ("remote.origin.pushurl", "https://b.invalid/r.git")], &[]);
    assert!(two.full_scan);
    assert_eq!(two.arguments(two.remotes[1].as_os_str()), words(&["push", "--all", "https://b.invalid/r.git", "main"]));
    assert_eq!(full(&["origin", "main"], &[url, ("url.https://other.invalid/.pushinsteadof", "https://example.invalid/")])[..2], words(&["push", "--all"])[..]);
    assert_eq!(full(&["https://example.invalid/r.git", "main"], &[url])[..3], words(&["push", "--all", "https://example.invalid/r.git"])[..]);
    assert_eq!(full(&["origin", "main"], &[("remote.origin.pushurl", "https://other.invalid/r.git")])[..2], words(&["push", "--all"])[..]);
    assert_eq!(full(&["elsewhere", "main"], &[url])[..3], words(&["push", "--all", "elsewhere"])[..]);
    let mirrored: UploadPlan = plan_of(
        &["--mirror", "origin"],
        &[url],
        &[("symbolic-ref", "refs/heads/work"), ("config:remote.origin.mirror", "true")],
    );
    assert!(mirrored.full_scan);
    assert_eq!(mirrored.arguments(mirrored.remotes[0].as_os_str()), words(&["push", "--all", "origin"]));
    let configured_mirror: UploadPlan = plan_of(
        &["origin"],
        &[url, ("remote.origin.mirror", "true")],
        &[("symbolic-ref", "refs/heads/work"), ("config:remote.origin.mirror", "true")],
    );
    assert!(configured_mirror.full_scan);
    assert_eq!(configured_mirror.arguments(configured_mirror.remotes[0].as_os_str()), words(&["push", "--all", "origin"]));
}

/// A forced refspec uploads its source,
/// a deletion uploads nothing,
/// `tag <name>` is the tag's ref.
#[test]
fn forced_deleting_and_tag_refspecs_read_as_git_reads_them() {
    assert_eq!(plan_of(&["origin", "+HEAD:refs/heads/x"], &[], &[]).refs, words(&["HEAD"]));
    no_plan(&["origin", ":gone"], &[], &[("symbolic-ref", "refs/heads/work")]);
    no_plan(&["--delete", "origin", "gone"], &[], &[]);
    assert_eq!(plan_of(&["origin", "tag", "v1"], &[], &[]).refs, words(&["refs/tags/v1"]));
}

/// Git splits a refspec at its last colon,
/// so a source that holds a colon survives.
#[test]
fn a_source_with_a_colon_is_split_at_the_last_colon() {
    let plan: UploadPlan = plan_of(&["origin", ":/unique message:refs/heads/new"], &[], &[]);
    assert_eq!(plan.refs, words(&[":/unique message"]));
    assert_eq!(plan_of(&["origin", "HEAD^{/a:b}:refs/heads/x"], &[], &[]).refs, words(&["HEAD^{/a:b}"]));
}

/// The refspec `:` (and `+:`,
/// and the configured `push = :`) means every matching branch,
/// not a deletion.
#[test]
fn the_matching_refspec_uploads_the_branches_and_is_not_a_deletion() {
    let heads: &[(&str, &str)] = &[("for-each-ref:refs/heads", "refs/heads/main\nrefs/heads/topic")];
    for spec in [":", "+:"] {
        assert_eq!(plan_of(&["origin", spec], &[], heads).refs, words(&["refs/heads/main", "refs/heads/topic"]), "{spec}");
    }
    let configured = plan_of(&["origin"], &[("remote.origin.push", ":")], heads);
    assert_eq!(configured.refs, words(&["refs/heads/main", "refs/heads/topic"]));
}

/// A pattern is matched as a refspec matches it:
/// the star crosses `/`,
/// and the text around it must match.
#[test]
fn a_star_crosses_slashes_as_a_refspec_does() {
    let heads: &[(&str, &str)] = &[(
        "for-each-ref:refs/heads",
        "refs/heads/main\nrefs/heads/feature/nested\nrefs/heads/feature/deep/er\nrefs/heads/other",
    )];
    let all: UploadPlan = plan_of(&["origin", "refs/heads/*:refs/heads/*"], &[], heads);
    assert_eq!(all.refs.len(), 4);
    let nested: UploadPlan = plan_of(&["origin", "refs/heads/feature/*:refs/heads/f/*"], &[], &[("for-each-ref:refs/heads/feature", "refs/heads/feature/nested\nrefs/heads/feature/deep/er")]);
    assert_eq!(nested.refs, words(&["refs/heads/feature/nested", "refs/heads/feature/deep/er"]));
    let suffixed: UploadPlan = plan_of(&["origin", "refs/heads/*-ok:refs/heads/*-ok"], &[], &[("for-each-ref:refs/heads", "refs/heads/a-ok\nrefs/heads/b-no\nrefs/heads/c/d-ok")]);
    assert_eq!(suffixed.refs, words(&["refs/heads/a-ok", "refs/heads/c/d-ok"]));
}

/// `--all` and `--tags` upload the refs they stand for,
/// never everything;
/// `--mirror` uploads everything.
#[test]
fn branch_and_tag_options_expand_and_mirror_uploads_everything() {
    let heads: &[(&str, &str)] = &[("for-each-ref:refs/heads", "refs/heads/main\nrefs/heads/other"), ("for-each-ref:refs/tags", "refs/tags/v1")];
    assert_eq!(plan_of(&["--all", "origin"], &[], heads).refs, words(&["refs/heads/main", "refs/heads/other"]));
    let tags: UploadPlan = plan_of(&["--tags", "origin"], &[], heads);
    assert_eq!((tags.refs, tags.everything), (words(&["refs/tags/v1"]), false));
    no_plan(&["--tags", "origin"], &[], &[]);
    let mirror: UploadPlan = plan_of(&["--mirror", "origin"], &[], &[]);
    assert!(mirror.everything);
    assert_eq!(mirror.arguments(mirror.remotes[0].as_os_str()), words(&["push", "--all", "origin"]));
}

/// A bare push takes the remote and branch from Git,
/// and a configured mirror,
/// `remote.<name>.push` or
///  `push.default=matching` decides what it sends.
#[test]
fn a_bare_push_resolves_its_remote_and_refs_the_way_git_does() {
    let branch: &[(&str, &str)] = &[("symbolic-ref", "refs/heads/work"), ("for-each-ref:refs/heads", "refs/heads/work\nrefs/heads/side")];
    let remote_of = |pairs: &[(&str, &str)]| -> OsString {
        return plan_of(&[], pairs, branch).remotes[0].clone();
    };
    let layered: &[(&str, &str)] = &[
        ("branch.work.remote", "upstream"),
        ("remote.pushdefault", "shared"),
        ("branch.work.pushremote", "mine"),
    ];
    assert_eq!(remote_of(layered), OsString::from("mine"));
    assert_eq!(remote_of(&layered[..2]), OsString::from("shared"));
    assert_eq!(remote_of(&layered[..1]), OsString::from("upstream"));
    assert_eq!(remote_of(&[("branch.other.remote", "elsewhere")]), OsString::from("origin"));
    assert_eq!(plan_of(&[], &[], branch).refs, words(&["refs/heads/work"]));
    assert_eq!(plan_of(&[], &[], &[("symbolic-ref", "refs/tags/work")]).refs, words(&["HEAD"]), "a tag is not a branch");
    assert_eq!(plan_of(&[], &[], &[]).refs, words(&["HEAD"]), "a detached HEAD");
    let matching = plan_of(&[], &[("push.default", "matching")], branch);
    assert_eq!(matching.refs, words(&["refs/heads/work", "refs/heads/side"]));
    let configured = plan_of(&["origin"], &[("remote.origin.push", "refs/heads/*:refs/heads/*"), ("remote.other.push", "x:y")], branch);
    assert_eq!(configured.refs, words(&["refs/heads/work", "refs/heads/side"]));
    let mapped = plan_of(&["origin"], &[("remote.origin.push", "other:refs/heads/mapped")], branch);
    assert_eq!(mapped.refs, words(&["other"]), "the source side is uploaded");
    let mirror_answers: &[(&str, &str)] = &[
        ("symbolic-ref", "refs/heads/work"),
        ("for-each-ref:refs/heads", "refs/heads/work\nrefs/heads/side"),
        ("config:remote.origin.mirror", "true"),
    ];
    let mirrored = plan_of(&["origin"], &[("remote.origin.mirror", "2")], mirror_answers);
    assert!(mirrored.everything, "Git reads mirror=2 as true");
    let ordinary = plan_of(&["origin"], &[("remote.origin.mirror", "false")], &[("symbolic-ref", "refs/heads/work"), ("config:remote.origin.mirror", "false")]);
    assert!(!ordinary.everything);
    let explicit = plan_of(&["origin", "main"], &[("remote.origin.push", "other:y"), ("remote.origin.mirror", "true")], branch);
    assert_eq!(explicit.refs, words(&["main"]), "a refspec typed on the command line wins over the configuration");
}

/// What:
///  A mirror setting is read through Git,
///  never from a list of spellings kept in this repository,
/// and a value Git
///  cannot read stops the push rather than being planned around.
/// Why:
///   Git's boolean grammar makes `2`,
/// `-1` and `0x10` true and the empty value false;
/// a mirror read as false
///  publishes the other branches' pointers without their objects.
#[test]
fn a_mirror_is_read_through_git_and_never_from_a_list_of_spellings() {
    let mut asked: Vec<String> = Vec::new();
    let mut answering = |arguments: &[&str]| -> Result<String, String> {
        asked.push(arguments.join(" "));
        return Ok(match arguments[0] {
            "symbolic-ref" => String::from("refs/heads/work"),
            "config" => String::from("true"),
            "for-each-ref" => String::from("refs/heads/work"),
            _ => String::new(),
        });
    };
    let settings: Settings = config(&[("remote.origin.mirror", "0x10")]);
    let planned = plan_upload(&[], &[], &[], Path::new("/work"), settings.as_slice(), &mut answering);
    let plan: UploadPlan = planned.expect("the plan is worked out").expect("a plan");
    assert!(plan.everything, "Git reads 0x10 as true");
    assert_eq!(plan.arguments(plan.remotes[0].as_os_str()), words(&["push", "--all", "origin"]));
    assert!(asked.contains(&String::from("config --type=bool --get remote.origin.mirror")), "{asked:?}");
    let mut refusing = |arguments: &[&str]| -> Result<String, String> {
        return match arguments[0] {
            "symbolic-ref" => Ok(String::from("refs/heads/work")),
            _ => Err(String::from("bad boolean config value 'maybe' for 'remote.origin.mirror'")),
        };
    };
    let unreadable: Settings = config(&[("remote.origin.mirror", "maybe")]);
    let refused = plan_upload(&[], &[], &[], Path::new("/work"), unreadable.as_slice(), &mut refusing);
    let error: PlanError = refused.expect_err("Git cannot read the value the push would read");
    assert!(error.0.contains("remote.origin.mirror"), "{error:?}");
    let settings_without_mirror: Settings = config(&[("remote.origin.url", "https://example.invalid/r.git"), ("push.default", "current")]);
    let answers: &[(&str, &str)] = &[("symbolic-ref", "refs/heads/work"), ("for-each-ref:refs/remotes/origin", "")];
    let (ordinary, ordinary_asked) = plan_in(Path::new("/work"), &[], &["origin"], &[], &settings_without_mirror, &Stub { answers, failing: &[] });
    assert!(!ordinary.expect("planned").expect("a plan").everything);
    assert_eq!(ordinary_asked, 2, "a repository without a mirror setting asks Git nothing about one");
}

/// Every query that fails,
/// and a push whose refs cannot be listed,
/// is an error and never "nothing to upload".
#[test]
fn a_failed_query_fails_closed() {
    let cases: [(&[&str], &str); 5] = [
        (&["--all", "origin"], "for-each-ref:refs/heads"),
        (&["--tags", "origin"], "for-each-ref:refs/tags"),
        (&["origin", "refs/heads/*:refs/heads/*"], "for-each-ref:refs/heads"),
        (&["origin", ":"], "for-each-ref:refs/heads"),
        (&["origin"], "symbolic-ref"),
    ];
    for (region, failing) in cases {
        let (planned, _) = plan_in(Path::new("/work"), &[], region, &[], &config(&[]), &Stub { answers: &[], failing: &[failing] });
        let error: PlanError = planned.expect_err(&format!("{region:?} must not plan on a failed query"));
        assert!(error.0.contains(failing.split(':').next().expect("a name")), "{error:?}");
    }
    let (config_failure, _) = plan_in(Path::new("/work"), &[], &["origin"], &[], &config(&[]), &Stub { answers: &[], failing: &[] });
    assert!(config_failure.is_ok());
    let mut refusing = |_arguments: &[&str]| -> Result<String, String> { return Err(String::from("no start")); };
    assert!(read_settings(&mut refusing).is_err());
}

/// A remote with several URLs is uploaded to once per URL,
/// because Git pushes to every one of them;
/// a remote with
///  one URL,
/// and one with one push URL,
/// keep the remote's name.
#[test]
fn a_remote_with_several_urls_is_uploaded_to_each() {
    let url: (&str, &str) = ("remote.origin.url", "https://a.invalid/r.git");
    let second: (&str, &str) = ("remote.origin.url", "https://b.invalid/r.git");
    let two: UploadPlan = plan_of(&["origin", "main"], &[url, second], &[]);
    assert_eq!(two.remotes, words(&["https://a.invalid/r.git", "https://b.invalid/r.git"]));
    assert!(two.full_scan, "Git writes to both, so neither store may stand for the other");
    assert_eq!(two.arguments(two.remotes[1].as_os_str()), words(&["push", "--all", "https://b.invalid/r.git", "main"]));
    assert_eq!(plan_of(&["origin", "main"], &[url], &[]).remotes, words(&["origin"]));
    let one_push_url: UploadPlan = plan_of(&["origin", "main"], &[url, second, ("remote.origin.pushurl", "https://c.invalid/r.git")], &[]);
    assert_eq!(one_push_url.remotes, words(&["origin"]), "a push URL is the only place Git writes to");
}

/// What:
///  Whether the destination confirms the tracking refs decides between the bounded form and `--all`,
/// and every
///  answer that is not the same branch at the same commit sends everything.
/// Why:
///   `refs/remotes/<remote>/*` describe the server the last fetch saw,
/// which need not be the one this push goes to;
///  the destination's own answer is the only evidence the plan takes.
#[test]
fn the_destination_itself_decides_whether_the_tracking_refs_are_evidence() {
    let url: (&str, &str) = ("remote.origin.url", "https://example.invalid/r.git");
    let tracked: &str = "1111111111111111111111111111111111111111\t\trefs/remotes/origin/main\n3333333333333333333333333333333333333333\trefs/remotes/origin/main\trefs/remotes/origin/HEAD\n";
    let planned = |answers: &[(&str, &str)], failing: &[&str]| -> (Option<UploadPlan>, usize) {
        let (result, asked) = plan_in(Path::new("/work"), &[], &["origin", "main"], &[], &config(&[url]), &Stub { answers, failing });
        return (result.expect("the plan is worked out"), asked);
    };
    let same: &[(&str, &str)] = &[("for-each-ref:refs/remotes/origin", tracked), ("ls-remote:origin", "1111111111111111111111111111111111111111\trefs/heads/main\n")];
    let (confirmed, asked) = planned(same, &[]);
    let plan: UploadPlan = confirmed.expect("a plan");
    assert!(!plan.full_scan, "the destination reports the branch this clone's tracking ref holds");
    assert_eq!(asked, 2, "one local listing and one round trip to the destination");
    assert_eq!(plan.arguments(plan.remotes[0].as_os_str()), words(&["push", "origin", "main"]));
    let sends_everything = |answers: &[(&str, &str)], failing: &[&str]| -> String {
        let (other, _asked) = planned(answers, failing);
        let full: UploadPlan = other.expect("a plan");
        assert!(full.full_scan, "{answers:?} {failing:?}");
        return full.arguments(full.remotes[0].as_os_str())[1].to_string_lossy().into_owned();
    };
    let tracked_listing: (&str, &str) = ("for-each-ref:refs/remotes/origin", tracked);
    assert_eq!(sends_everything(&[tracked_listing, ("ls-remote:origin", "2222222222222222222222222222222222222222\trefs/heads/main\n")], &[]), "--all", "the destination moved the branch on");
    assert_eq!(sends_everything(&[tracked_listing, ("ls-remote:origin", "1111111111111111111111111111111111111111\trefs/heads/other\n")], &[]), "--all", "the destination no longer has the branch");
    assert_eq!(sends_everything(&[tracked_listing, ("ls-remote:origin", "not a listing\n")], &[]), "--all", "an answer that is not a listing");
    assert_eq!(sends_everything(&[tracked_listing], &["ls-remote:origin"]), "--all", "a destination that cannot be asked");
    assert_eq!(sends_everything(&[], &["for-each-ref:refs/remotes/origin"]), "--all", "tracking refs that cannot be listed");
    let (none, none_asked) = planned(&[("for-each-ref:refs/remotes/origin", "")], &[]);
    let nothing_to_skip: UploadPlan = none.expect("a plan");
    assert!(!nothing_to_skip.full_scan, "a clone with no tracking refs has nothing to skip");
    assert_eq!(none_asked, 1, "and nothing to ask the destination about");
    let (symbolic, symbolic_asked) = planned(&[("for-each-ref:refs/remotes/origin", "3333333333333333333333333333333333333333\trefs/remotes/origin/main\trefs/remotes/origin/HEAD\n")], &[]);
    assert!(!symbolic.expect("a plan").full_scan, "a symbolic ref holds no belief about the destination");
    assert_eq!(symbolic_asked, 1);
}

/// A remote with several push URLs is uploaded to once per URL;
/// one push URL or none keeps the remote's name.
#[test]
fn a_remote_with_several_push_urls_is_uploaded_to_each() {
    let two: UploadPlan = plan_of(&["origin", "main"], &[("remote.origin.pushurl", "https://a.invalid/r.git"), ("remote.origin.pushurl", "https://b.invalid/r.git")], &[]);
    assert_eq!(two.remotes, words(&["https://a.invalid/r.git", "https://b.invalid/r.git"]));
    let one: UploadPlan = plan_of(&["origin", "main"], &[("remote.origin.pushurl", "https://a.invalid/r.git")], &[]);
    assert_eq!(one.remotes, words(&["origin"]));
    assert_eq!(plan_of(&["origin", "main"], &[], &[]).remotes, words(&["origin"]));
}

/// A destination typed as a path becomes a percent-encoded `file://` URL whether or not it starts with a dot;
/// a configured
///  remote with the same name as a directory,
/// a URL and an scp-style address stay as typed.
#[test]
fn a_path_destination_becomes_an_encoded_file_url() {
    let root = fixture("lfs-plan-paths");
    let (backup, hashed): (PathBuf, PathBuf) = (root.join("backup.git"), root.join("hash#dir.git"));
    std::fs::create_dir(&backup).expect("backup directory");
    std::fs::create_dir(&hashed).expect("hash directory");
    let remote_of = |typed: &str, settings: &[(&str, &str)]| -> OsString {
        let (planned, _) = plan_in(root.as_path(), &[], &[typed, "main"], &[], &config(settings), &Stub { answers: &[], failing: &[] });
        return planned.expect("planned").expect("a plan").remotes[0].clone();
    };
    let prefix: String = format!("file://{}", root.display());
    assert_eq!(remote_of("backup.git", &[]), OsString::from(format!("{prefix}/backup.git")));
    assert_eq!(remote_of("./backup.git", &[]), OsString::from(format!("{prefix}/backup.git")));
    assert_eq!(remote_of("hash#dir.git", &[]), OsString::from(format!("{prefix}/hash%23dir.git")));
    assert_eq!(remote_of("backup.git", &[("remote.backup.git.url", "x")]), OsString::from("backup.git"));
    assert_eq!(remote_of("origin", &[]), OsString::from("origin"), "no such directory: a remote name");
    assert_eq!(remote_of("https://example.invalid/r.git", &[]), OsString::from("https://example.invalid/r.git"));
    assert_eq!(remote_of("git@example.invalid:r.git", &[]), OsString::from("git@example.invalid:r.git"));
    remove(root.as_path());
}

/// The first positional is the repository whatever `--repo` says,
/// and `--repo` and its abbreviations fill in only when
///  none is typed.
#[test]
fn the_repository_is_read_as_git_reads_it() {
    let positional: UploadPlan = plan_of(&["--repo=second", "origin", "main"], &[], &[]);
    assert_eq!((positional.remotes, positional.refs), (words(&["origin"]), words(&["main"])));
    for spelling in ["--repo=elsewhere", "--rep=elsewhere"] {
        let option_only: UploadPlan = plan_of(&[spelling, "--force"], &[], &[("symbolic-ref", "refs/heads/work")]);
        assert_eq!(option_only.remotes, words(&["elsewhere"]), "{spelling}");
    }
}

/// A push that recurses into submodules is marked;
/// every way of bypassing the upload plans nothing.
#[test]
fn submodule_pushes_are_marked_and_bypasses_plan_nothing() {
    for (value, marked) in [("on-demand", true), ("only", true), ("check", false), ("no", false)] {
        let option: String = format!("--recurse-submodules={value}");
        assert_eq!(plan_of(&[option.as_str(), "origin", "main"], &[], &[]).submodules, marked, "{value}");
    }
    for region in [vec!["--no-verify", "origin", "main"], vec!["--no-ver", "origin", "main"], vec!["--dry-run", "origin", "main"]] {
        no_plan(region.as_slice(), &[], &[]);
    }
    let environment = |pairs: &[(&str, &str)]| -> Vec<(OsString, OsString)> {
        return pairs
            .iter()
            .map(|(name, value)| {
                return (OsString::from(name), OsString::from(value));
            })
            .collect();
    };
    let stub: Stub<'_> = Stub { answers: &[], failing: &[] };
    let off: &[&str] = &["-c", "hook.cli-git-lfs-pre-push.enabled=false"];
    assert_eq!(plan_in(Path::new("/work"), off, &["origin", "main"], &[], &config(&[]), &stub).0, Ok(None));
    let skip = environment(&[("GIT_LFS_SKIP_PUSH", "1")]);
    assert_eq!(plan_in(Path::new("/work"), &[], &["origin", "main"], skip.as_slice(), &config(&[]), &stub).0, Ok(None));
    let pairs = environment(&[("GIT_CONFIG_COUNT", "1"), ("GIT_CONFIG_KEY_0", "hook.cli-git-lfs-pre-push.enabled"), ("GIT_CONFIG_VALUE_0", "false")]);
    assert_eq!(plan_in(Path::new("/work"), &[], &["origin", "main"], pairs.as_slice(), &config(&[]), &stub).0, Ok(None));
    assert_eq!(plan_in(Path::new("/work"), &[], &["--no-such-option"], &[], &config(&[]), &stub).0, Ok(None), "Git refuses it");
}

/// The snapshot's keys are filtered to the ones the plan reads,
/// with the same rule the query's pattern states.
#[test]
fn only_the_keys_the_plan_reads_are_taken_from_a_snapshot() {
    for key in [
        "branch.work.pushremote",
        "branch.a.b.remote",
        "remote.pushdefault",
        "remote.origin.push",
        "remote.origin.pushurl",
        "remote.origin.mirror",
        "remote.o.r.url",
        "push.default",
        "url.https://example.invalid/.pushinsteadof",
    ] {
        assert!(crate::lfs_upload_settings::wanted_key(key), "{key}");
    }
    for key in ["url.https://example.invalid/.insteadof", "remote.origin.fetch", "branch.work.merge", "push.followtags", "core.bare", "remote.origin.tagopt"] {
        assert!(!crate::lfs_upload_settings::wanted_key(key), "{key}");
    }
}

/// What:
///  A stubbed Git whose questions are awaited:
/// the same answers and the same failures the synchronous stub gives.
/// Why:
///   The two faces of the plan must ask the same questions and read the same answers,
/// so one stub answers
///  both.
struct StubAwaiting<'a> {
    /// Answers by key.
    answers: &'a [(&'a str, &'a str)],
    /// Keys that fail.
    failing: &'a [&'a str],
    /// How many questions were asked.
    asked: usize,
}

impl StubAwaiting<'_> {
    /// The answer to one question,
    /// counted.
    fn answer(&mut self, arguments: &[&str]) -> Result<String, String> {
        self.asked += 1;
        let key: String = stub_key(arguments);
        if self.failing.contains(&key.as_str()) {
            return Err(format!("{key} failed"));
        }
        return Ok(self
            .answers
            .iter()
            .find(|(name, _)| {
                return *name == key;
            })
            .map_or(String::new(), |(_, answer)| {
                return (*answer).to_owned();
            }));
    }
}

impl GitQueryAwaiting for StubAwaiting<'_> {
    fn ask<'a>(&'a mut self, arguments: &'a [&'a str]) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send + 'a>> {
        return Box::pin(async move { return self.answer(arguments); });
    }
}

/// Plan a push with the awaited face of the plan and the same stub.
fn plan_in_awaiting(
    directory: &Path,
    global_prefix: &[&str],
    region: &[&str],
    environment: &[(OsString, OsString)],
    settings: &Settings,
    stub: &Stub<'_>,
) -> (Result<Option<UploadPlan>, PlanError>, usize) {
    let mut awaiting: StubAwaiting<'_> = StubAwaiting { answers: stub.answers, failing: stub.failing, asked: 0 };
    let (planned, _ticks): (Result<Option<UploadPlan>, PlanError>, u32) = crate::ticker_beside_a_wait::with_a_ticker_beside(plan_upload_awaiting(
        words(global_prefix).as_slice(),
        words(region).as_slice(),
        environment,
        directory,
        settings.as_slice(),
        &mut awaiting,
    ));
    return (planned, awaiting.asked);
}

/// What:
///  The awaited face of the plan asks the same questions and gives the same plan as the synchronous face,
/// for every shape of a push and every way a question can fail.
/// Why:
///   The landing round's push plans its upload as a task,
/// and a plan that differed would send other objects or
///  refuse a push the synchronous face allows.
#[test]
fn the_awaited_face_of_the_plan_asks_and_answers_as_the_synchronous_face_does() {
    let answers: &[(&str, &str)] = &[
        ("for-each-ref:refs/heads", "main\nother\n"),
        ("for-each-ref:refs/tags", "v1\n"),
        ("symbolic-ref", "refs/heads/main"),
    ];
    let settings: Settings = config(&[("remote.origin.push", "refs/heads/*:refs/heads/*")]);
    let cases: &[(&[&str], &[&str])] = &[
        (&[], &[]),
        (&["origin"], &[]),
        (&["--all"], &[]),
        (&["--tags"], &[]),
        (&["--mirror"], &[]),
        (&["origin", "refs/heads/main"], &[]),
        (&["origin", "main"], &[]),
        (&["origin", "refs/heads/*"], &[]),
        (&["--dry-run"], &[]),
        (&["--no-verify"], &[]),
        (&["origin", ":refs/heads/gone"], &[]),
        (&["--recurse-submodules=only"], &[]),
    ];
    for (region, failing) in cases {
        for environment in [vec![], vec![(OsString::from("GIT_LFS_SKIP_PUSH"), OsString::from("1"))]] {
            let stub: Stub<'_> = Stub { answers, failing };
            let synchronous = plan_in(Path::new("/work"), &[], region, environment.as_slice(), &settings, &stub);
            let awaited = plan_in_awaiting(Path::new("/work"), &[], region, environment.as_slice(), &settings, &stub);
            assert_eq!(format!("{awaited:?}"), format!("{synchronous:?}"), "{region:?}");
        }
    }
    // One question that fails, so the failure text is compared too.
    let stub: Stub<'_> = Stub { answers, failing: &["for-each-ref:refs/heads"] };
    let synchronous = plan_in(Path::new("/work"), &[], &["--all"], &[], &settings, &stub);
    let awaited = plan_in_awaiting(Path::new("/work"), &[], &["--all"], &[], &settings, &stub);
    assert_eq!(format!("{awaited:?}"), format!("{synchronous:?}"));
    // A mirror setting is asked of Git on both faces, and the two plans agree about it.
    let mirror_settings: Settings = config(&[("remote.origin.mirror", "2")]);
    let mirror_answers: &[(&str, &str)] = &[("symbolic-ref", "refs/heads/main"), ("config:remote.origin.mirror", "true")];
    let mirror_stub: Stub<'_> = Stub { answers: mirror_answers, failing: &[] };
    let mirror_synchronous = plan_in(Path::new("/work"), &[], &["origin"], &[], &mirror_settings, &mirror_stub);
    let mirror_awaited = plan_in_awaiting(Path::new("/work"), &[], &["origin"], &[], &mirror_settings, &mirror_stub);
    assert_eq!(format!("{mirror_awaited:?}"), format!("{mirror_synchronous:?}"));
    assert!(mirror_synchronous.0.expect("planned").expect("a plan").everything);
}
