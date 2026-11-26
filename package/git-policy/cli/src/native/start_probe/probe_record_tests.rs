//! What:
//!  Controls of the record's text:
//!  `v1` first,
//!  one fixed key per line,
//!  the started record with no verdict,
//!  each arm's line,
//!  and whether the source existed.
//! Why:
//!  `doctor` and the add preflight read this file,
//!  so a key that moved or a value that holds a newline would change a
//!  reader's question.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(text.split('\n')[0]).toBe('v1'); expect(text).toContain('state finished');
//! ```

use super::super::probe_arms::ProbeStep;
use super::super::probe_fingerprint::ProbeFingerprint;
use super::super::probe_report::ProbeReport;
use super::super::probe_report::ProbeVerdict;
use super::super::probe_status_line::StepLine;
use super::{
    ProbeRecord,
    SourcePresence,
    started_text,
};

/// A fingerprint with every attribute present.
fn fingerprint() -> ProbeFingerprint {
    return ProbeFingerprint {
        kernel_release: String::from("6.8.0-147-generic"),
        module_version: Some(String::from("2.4.4-1")),
        module_srcversion: Some(String::from("A3553C308C29FD4B6595A72")),
        module_build_id: Some(String::from("00ff")),
        boot_id: String::from("6f0f1a2b-3c4d"),
        handler: Some(String::from(
            "/usr/libexec/mount-git-worktree-clone-datasets\t1\t2.3\t4.5\t6\t7",
        )),
    };
}

/// The started record holds the fingerprint,
/// says `state started`,
/// and carries no verdict.
#[test]
fn the_started_record_carries_no_verdict() {
    let text: String = started_text(&fingerprint(), "2026-11-20T10:00:00-05:00");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], "v1");
    assert_eq!(lines[1], "kind probe-result");
    assert_eq!(lines[2], "state started");
    assert!(text.contains("boot 6f0f1a2b-3c4d"));
    assert!(text.contains("kernel 6.8.0-147-generic"));
    assert!(!text.contains("verdict"));
}

/// The finished record names the verdict,
/// each arm and each step,
/// and the last step that finished.
#[test]
fn the_finished_record_names_verdict_arms_and_steps() {
    let mut report: ProbeReport = ProbeReport::new();
    for step in ProbeStep::WORKER_ORDER {
        if step == ProbeStep::CheckoutRoot {
            report.record(StepLine::not_exercised(step, "no-checkout-path", None));
        } else {
            report.record(StepLine::passed(step));
        }
    }
    for step in ProbeStep::HOLDER_ORDER {
        report.record(StepLine::passed(step));
    }
    let verdict: ProbeVerdict = ProbeVerdict::Blocked {
        arm: super::super::probe_arms::ProbeArm::Walk,
    };
    let record = ProbeRecord {
        fingerprint: &fingerprint(),
        time: "2026-11-20T10:00:00-05:00",
        source: Some("zpcachyos/nosnap/git-worktree-probe@ready"),
        source_refusal: None,
        report: &report,
        verdict,
        worker_namespace: Some("mnt:[4026531840]"),
        holder_namespace: Some("mnt:[4026532001]"),
        skip_reason: None,
    };
    let text: String = record.text();
    assert!(text.starts_with("v1\nkind probe-result\nstate finished\n"));
    assert!(text.contains("verdict blocked"));
    assert!(text.contains("last-finished teardown"));
    assert!(text.contains("arm walk not-exercised errno none"));
    assert!(text.contains("arm attach passed errno none"));
    assert!(text.contains("step checkout-root not-exercised errno none reason no-checkout-path"));
    assert!(text.contains("worker-namespace mnt:[4026531840]"));
    assert!(text.contains("holder-namespace mnt:[4026532001]"));
    assert!(text.contains("source-existed yes"));
}

/// A source the exclusive create refused with `ENOENT` did not exist,
/// and one it refused with `EBUSY` did.
#[test]
fn the_source_presence_follows_the_exclusive_create() {
    for (errno, presence) in [
        (Some("ENOENT"), SourcePresence::No),
        (Some("EBUSY"), SourcePresence::Yes),
    ] {
        let mut report: ProbeReport = ProbeReport::new();
        report.record(StepLine::failed(ProbeStep::AttachExclusiveCreate, errno));
        let record = ProbeRecord {
            fingerprint: &fingerprint(),
            time: "t",
            source: None,
            source_refusal: None,
            report: &report,
            verdict: ProbeVerdict::Failed {
                step: ProbeStep::AttachExclusiveCreate,
            },
            worker_namespace: None,
            holder_namespace: None,
            skip_reason: None,
        };
        let word: &str = match presence {
            SourcePresence::Yes => "yes",
            SourcePresence::No => "no",
            SourcePresence::Unknown => "unknown",
        };
        assert!(
            record.text().contains(&format!("source-existed {word}")),
            "{}",
            record.text()
        );
    }
}

/// A missing module attribute is written as `absent`,
/// never invented.
#[test]
fn an_absent_attribute_is_written_absent() {
    let mut print: ProbeFingerprint = fingerprint();
    print.module_build_id = None;
    let text: String = started_text(&print, "t");
    assert!(text.contains("build-id absent"));
}
