//! What:
//!  The restricted endpoint normalizer against outputs measured from the unchanged incumbent.
//! Why:
//!  `fixtures/lfs-url-parity.json` records what `lfsObjectBase` and `parseLfsConfig` returned
//! under Node for 868 endpoints and 73 `.lfsconfig` texts,
//!  plus the native rejection each
//! out-of-contract input must get.
//!  Self-authored expectations alone would not establish parity.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const c of fixture.bases) expect(native(c.endpoint)).toEqual(c.rejection ?? c.expected);
//! ```

/// Import the normalizer,
///  its rejection type and the config-level reader.
use super::{LfsUrlRejection, lfs_object_base};
use crate::markdown_lfs_config::{parse_lfs_config, read_lfs_object_base};
use crate::test_fs::Fixture;
/// Import the existing typed JSON decoder for the committed measured fixture.
use serde::Deserialize;

/// One measured endpoint:
///  the incumbent's output and the native rejection,
///  if any.
#[derive(Deserialize)]
struct BaseCase {
    /// Corpus label,
    ///  used only in failure messages.
    class: String,
    /// Endpoint as it would be written in `.lfsconfig`.
    endpoint: String,
    /// Incumbent output,
    ///  or absent when the incumbent throws.
    expected: Option<String>,
    /// Fixture name of the first native rejection,
    ///  or absent when the native function must accept.
    rejection: Option<String>,
}

/// One measured `.lfsconfig` text.
#[derive(Deserialize)]
struct ConfigCase {
    /// Corpus label,
    ///  used only in failure messages.
    class: String,
    /// Complete file contents.
    text: String,
    /// Incumbent bases in declaration order,
    ///  or absent when the incumbent throws.
    expected: Option<Vec<String>>,
    /// Fixture name of the first declaration's native rejection,
    ///  or absent when all are accepted.
    rejection: Option<String>,
}

/// Measured catalogs;
///  the JSON also retains incumbent source hashes and the oracle as provenance.
#[derive(Deserialize)]
struct Cases {
    /// Endpoint cases.
    bases: Vec<BaseCase>,
    /// Configuration-text cases.
    configs: Vec<ConfigCase>,
}

/// Decode the embedded fixture,
///  independent of live repository files during a container run.
fn cases() -> Cases {
    let source: &str = include_str!("../fixtures/lfs-url-parity.json");
    return serde_json::from_str::<Cases>(source).expect("measured fixture");
}

/// Every endpoint either equals the incumbent's output or gets exactly the recorded rejection.
#[test]
fn measured_endpoints_match_or_are_rejected_for_the_recorded_reason() {
    let catalog: Cases = cases();
    // Independently counted catalog sizes prove the fixture did not become silently smaller.
    assert_eq!(catalog.bases.len(), 868);
    let mut matched: usize = 0;
    let mut classified: usize = 0;
    let mut both_reject: usize = 0;
    for case in catalog.bases {
        let actual: Result<String, LfsUrlRejection> = lfs_object_base(case.endpoint.as_str());
        match case.rejection {
            None => {
                let expected: String = case.expected.expect("an accepted case records its base");
                assert_eq!(actual, Ok(expected), "{} {:?}", case.class, case.endpoint);
                matched += 1;
            }
            Some(name) => {
                let rejection: LfsUrlRejection =
                    actual.expect_err(format!("{} {:?}", case.class, case.endpoint).as_str());
                assert_eq!(
                    rejection.fixture_name(),
                    name,
                    "{} {:?}",
                    case.class,
                    case.endpoint
                );
                if case.expected.is_some() {
                    classified += 1;
                } else {
                    both_reject += 1;
                }
            }
        }
    }
    // The evaluation's recorded split: equal outputs, native-only rejections, and shared rejections.
    assert_eq!((matched, classified, both_reject), (459, 289, 120));
}

/// Every configuration text yields the incumbent's bases in order,
///  or the first declaration's rejection.
#[test]
fn measured_configuration_texts_match_or_are_rejected_for_the_recorded_reason() {
    let catalog: Cases = cases();
    assert_eq!(catalog.configs.len(), 73);
    let mut matched: usize = 0;
    let mut classified: usize = 0;
    let mut both_reject: usize = 0;
    for case in catalog.configs {
        let actual: Result<Vec<String>, LfsUrlRejection> = parse_lfs_config(case.text.as_str());
        match case.rejection {
            None => {
                let expected: Vec<String> = case.expected.expect("an accepted case records bases");
                assert_eq!(actual, Ok(expected), "{} {:?}", case.class, case.text);
                matched += 1;
            }
            Some(name) => {
                let rejection: LfsUrlRejection =
                    actual.expect_err(format!("{} {:?}", case.class, case.text).as_str());
                assert_eq!(
                    rejection.fixture_name(),
                    name,
                    "{} {:?}",
                    case.class,
                    case.text
                );
                if case.expected.is_some() {
                    classified += 1;
                } else {
                    both_reject += 1;
                }
            }
        }
    }
    assert_eq!((matched, classified, both_reject), (53, 11, 9));
}

/// The incumbent's own unit cases:
///  userinfo,
///  query,
///  fragment and one final slash are removed;
///  a path prefix stays.
#[test]
fn incumbent_unit_cases_hold() {
    assert_eq!(
        lfs_object_base("https://lfs:token@lfs.example/?x=1#y"),
        Ok(String::from("https://lfs.example"))
    );
    assert_eq!(
        lfs_object_base("https://host.example/repo/info/lfs/"),
        Ok(String::from("https://host.example/repo/info/lfs"))
    );
    // Exactly one final slash is removed, the scheme and host are lowercased, and a default port is dropped.
    assert_eq!(
        lfs_object_base("HTTPS://LFS.Example:443/a//"),
        Ok(String::from("https://lfs.example/a/"))
    );
    assert_eq!(
        lfs_object_base("http://lfs.example:080/"),
        Ok(String::from("http://lfs.example"))
    );
    assert_eq!(
        lfs_object_base("http://lfs.example:0443/x"),
        Ok(String::from("http://lfs.example:443/x"))
    );
}

/// Every rejection has a distinct fixture name and an explanation that names a remediation.
#[test]
fn every_rejection_explains_itself_without_the_endpoint() {
    let all: [LfsUrlRejection; 12] = [
        LfsUrlRejection::Scheme,
        LfsUrlRejection::Backslash,
        LfsUrlRejection::Ipv6Literal,
        LfsUrlRejection::EmptyHost,
        LfsUrlRejection::HostCharacter,
        LfsUrlRejection::HostLength,
        LfsUrlRejection::EmptyHostLabel,
        LfsUrlRejection::NumericHost,
        LfsUrlRejection::PortCharacter,
        LfsUrlRejection::PortRange,
        LfsUrlRejection::PathCharacter,
        LfsUrlRejection::DotSegment,
    ];
    let mut names: Vec<&str> = Vec::<&str>::new();
    for rejection in all {
        assert!(!names.contains(&rejection.fixture_name()));
        names.push(rejection.fixture_name());
        assert_eq!(rejection.to_string(), rejection.explanation());
        // Each explanation is two sentences: what was wrong, then what to write.
        assert!(rejection.explanation().contains(". "), "{rejection:?}");
        assert!(rejection.explanation().ends_with('.'), "{rejection:?}");
    }
    assert_eq!(names.len(), 12);
}

/// A repository root's first base is read;
///  a rejected endpoint fails the read without echoing credentials.
#[test]
fn configuration_reads_use_the_normalizer_and_never_echo_credentials() {
    let fixture: Fixture = Fixture::new();
    let path: std::path::PathBuf = fixture.path.join(".lfsconfig");
    std::fs::write(&path, "[lfs]\n\turl = https://lfs:t@lfs.example/\n").expect("configuration");
    assert_eq!(
        read_lfs_object_base(&fixture.path).expect("accepted endpoint"),
        Some(String::from("https://lfs.example"))
    );
    std::fs::write(
        &path,
        "[lfs]\n\turl = https://lfs.example\n[remote \"o\"]\n\tlfsurl = https://user:secret@[::1]/x\n",
    )
    .expect("configuration with a later rejected declaration");
    let error = read_lfs_object_base(&fixture.path).expect_err("later declaration is rejected");
    assert!(error.message.contains(".lfsconfig"), "{}", error.message);
    assert!(error.message.contains("IPv6"), "{}", error.message);
    assert!(!error.message.contains("secret"), "{}", error.message);
    assert!(!error.message.contains("[::1]"), "{}", error.message);
}
