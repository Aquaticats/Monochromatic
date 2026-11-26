//! What:
//!  The ban on a control taking the runner the whole service process shares:
//!  a control that reviews a commit declares a `Containment` of its own (`static OWN`)
//!  and reaches the review through `review_item_in`.
//! Why:
//!  `crate::standards_review_run::RUNNER` is one runner for the whole process,
//!  a runner admits one job and answers every other caller with `Contained::Busy`,
//!  and `cargo test` runs a crate's tests as threads of one process,
//!  so a module whose controls all review through it fails whichever of them ask while another holds it:
//!  measured on 2026-11-26,
//!  `service::service_review::tests` failed 7 of its 16 controls that way,
//!  a varying set of them from run to run,
//!  and passed all 16 with `--test-threads=1` (issue 6499).
//!  The container gate starts each test in a process of its own,
//!  so the gate cannot show it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(filesThatCall(PRODUCTION_RUNNER_FACES)).toEqual([]);
//! ```

use std::path::Path;

use crate::source_scan_tests::{
    CrateRoot,
    SourceFile,
    classified_sources,
    real_roots,
    source_root,
    write_tree,
};

/// What:
///  `PRODUCTION_RUNNER_CALLS` is each call that runs a job in `crate::standards_review_run::RUNNER`,
///  with the reason that call is the shared runner's.
///  A slice (`&[..]`) of pairs,
///  whose siblings a reader might expect are `[(&str, &str); N]`,
///  an array whose length is written beside it,
///  and `Vec<(&str, &str)>`,
///  which is built while the program runs.
/// Why:
///  A slice keeps its length in the value,
///  so adding a call edits no count
///  (`how-to-write-this-packages-code-and-tests.md`,
///  "Write less code").
/// Gotcha:
///  A needle must not be a prefix of the call a control may use instead:
///  `review_item(` is not inside `review_item_in(`,
///  and `run_contained(` is not inside `run_contained_in(`,
///  because the underscore after the name keeps them apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PRODUCTION_RUNNER_CALLS: ReadonlyArray<readonly [string, string]> = [
///   ['run_contained(', 'the contained runner of the whole process'],
/// ];
/// ```
const PRODUCTION_RUNNER_CALLS: &[(&str, &str)] = &[
    (
        "run_contained(",
        "runs a job in `crate::standards_review_run::RUNNER`",
    ),
    ("review_item(", "runs a work item's review in `RUNNER`"),
    ("review_item_within(", "the same, with the caller's ceiling"),
    (
        "review_item_awaiting(",
        "the same, awaited by a task of the runtime",
    ),
    (
        "review_item_within_awaiting(",
        "the same, awaited, with the caller's ceiling",
    ),
];

/// What:
///  `NAMES_THE_CALLS_WITHOUT_MAKING_THEM` is each file that names one of those calls without making it,
///  with why it does.
///  This check's own file names every one of them in the list above and plants one in a fixture.
/// Why:
///  An entry here that matches nothing is reported,
///  so a file that moves or stops naming the calls shrinks this list instead of excusing nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NAMES_THE_CALLS_WITHOUT_MAKING_THEM = [['source_guard/no_control_takes_the_production_runner_tests.rs', 'this check names the calls and plants one in a fixture']];
/// ```
const NAMES_THE_CALLS_WITHOUT_MAKING_THEM: &[(&str, &str)] = &[(
    "source_guard/no_control_takes_the_production_runner_tests.rs",
    "this check names the calls and plants one in a fixture, and makes none of them",
)];

/// What:
///  `ROUTES_THROUGH_THE_HANDLERS` is each call that asks the service's real handlers for a review,
///  whose job then runs in `crate::standards_review_run::RUNNER`
///  unless the control hands the handlers a runner of its own.
/// Why:
///  `GitHandlers::review` and `GitHandlers::review_awaiting` reach the runner without naming any call above,
///  so this scan could not see that route and two controls of
///  `landing/service_work_tests.rs` reviewed in the shared runner (issue 6500,
///  measured there: two of them in one process answered `1 passed; 1 failed` in each of three runs).
///  Pointing the review's gateway call at a mock is what says a control reviews through these handlers.
/// Gotcha:
///  The check is per file:
///  a file that hands one pair of handlers a runner and points a second pair at a mock passes,
///  so a new control of such a file states its own runner by the review it makes,
///  and a control that reviews through the handlers without a mock is not seen at all.
const ROUTES_THROUGH_THE_HANDLERS: &[(&str, &str)] = &[(
    "with_review_endpoint(",
    "points the real handlers' review at a gateway, so their reviews run in `RUNNER` unless the control hands them one",
)];

/// What:
///  `HANDS_THE_HANDLERS_A_RUNNER` is the call that gives the real handlers a runner of the control's own.
/// Why:
///  A control that takes a route above must name it,
///  and the finding says which call to write.
const HANDS_THE_HANDLERS_A_RUNNER: &str = "with_review_runner(";

/// What:
///  `lines_making_the_call(text, call)` is the number of each line of `text` that makes `call`,
///  counting from one as an editor does.
///  A call split over two lines is found where it starts,
///  and a line whose call sits in a `//` comment is not a call.
/// Why:
///  A finding has to name the line a reader edits,
///  and the exemption list is by file,
///  so a comment that explains the rule must not be a finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function linesMakingTheCall(text: string, call: string): number[] {
///   const found: number[] = [];
///   text.split('\n').forEach((line, index) => {
///     let searched = 0;
///     while (true) {
///       const place = line.indexOf(call, searched);
///       if (place < 0) break;
///       if (!line.slice(0, place).trimStart().startsWith('//')) found.push(index + 1);
///       searched = place + call.length;
///     }
///   });
///   return found;
/// }
/// ```
fn lines_making_the_call(
    text: &str,
    call: &str,
) -> Vec<usize> {
    let mut found: Vec<usize> = Vec::new();
    for (index, line) in text.lines().enumerate() {
        // What: `searched` is the place of the text already looked at in this line, so a line with the call twice
        //  is read twice and not once.
        // Why: a fixture states its layout in one line as often as in three.
        let mut searched: usize = 0;
        while let Some(place) = line[searched..].find(call) {
            let column: usize = searched + place;
            let before: &str = &line[..column];
            let commented: bool = before.trim_start().starts_with("//");
            if !commented {
                // What: `index + 1` converts the count from zero that `enumerate` gives to the count from one that
                //  an editor shows.
                // Why: a finding one line off sends its reader to the wrong line.
                found.push(index + 1);
            }
            searched = column + call.len();
        }
    }
    return found;
}

/// What:
///  `findings_in(root, roots)` is every `file:line` of the test-only sources under `root` that makes one of the
///  calls above,
///  or that asks the real handlers for a review without handing them a runner,
///  or one text saying why the scan cannot answer.
/// Why:
///  Release code may take the production runner:
///  the service's own work path is what the runner is for,
///  and only a control must not,
///  because a run holds many of them at once in one process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingsIn(root: string, roots: CrateRoot[]): string[]  // throws Error
/// ```
fn findings_in(
    root: &Path,
    roots: &[CrateRoot],
) -> Result<Vec<String>, String> {
    let mut found: Vec<String> = Vec::new();
    for file in classified_sources(root, roots)? {
        if !file.test_only {
            continue;
        }
        if NAMES_THE_CALLS_WITHOUT_MAKING_THEM
            .iter()
            .any(|(named, _)| return *named == file.path)
        {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(file.path.as_str()))
            .map_err(|error| return format!("cannot read {}: {error}", file.path))?;
        for (call, why) in PRODUCTION_RUNNER_CALLS {
            for number in lines_making_the_call(text.as_str(), call) {
                found.push(format!(
                    "{}:{number} names `{call}`, which {why}",
                    file.path
                ));
            }
        }
        // What:     `takes_the_route` is whether this file asks the handlers for a review, and
        //           `hands_a_runner` whether it gives them a runner of the control's own.
        // Why:      a file that does both is what the rule asks for, and one that does neither is no site.
        for (route, why) in ROUTES_THROUGH_THE_HANDLERS {
            let takes_the_route: Vec<usize> = lines_making_the_call(text.as_str(), route);
            let hands_a_runner: Vec<usize> =
                lines_making_the_call(text.as_str(), HANDS_THE_HANDLERS_A_RUNNER);
            if takes_the_route.is_empty() || !hands_a_runner.is_empty() {
                continue;
            }
            for number in takes_the_route {
                found.push(format!(
                    "{}:{number} names `{route}`, which {why}; declare `static OWN: \
                     crate::standards_review_run::Containment = \
                     crate::standards_review_run::Containment::new();` and write \
                     `{HANDS_THE_HANDLERS_A_RUNNER}&OWN)` beside it",
                    file.path
                ));
            }
        }
    }
    return Ok(found);
}

/// What:
///  `findings()` is the findings in this crate's own sources.
/// Why:
///  The control below judges the tree the build compiled,
///  never a fixture.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findings(): string[] { return findingsIn(sourceRoot(), realRoots()); }
/// ```
fn findings() -> Vec<String> {
    return findings_in(&source_root(), &real_roots()).expect("the crate's sources");
}

/// What:
///  `sources()` is the classified listing of this crate's own sources.
/// Why:
///  The exemption control and this one read the same answer to "which files exist and which are test-only".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sources(): SourceFile[] { return classified_sources(sourceRoot(), realRoots()); }
/// ```
fn sources() -> Vec<SourceFile> {
    return classified_sources(&source_root(), &real_roots()).expect("the crate's sources");
}

/// No control runs a job in the runner the whole process shares.
/// A control that reviews a commit declares a `Containment` of its own and calls `review_item_in`,
/// and one that reviews through the real handlers gives them that runner with `with_review_runner`.
#[test]
fn no_control_takes_the_runner_the_whole_process_shares() {
    let found: Vec<String> = findings();
    assert!(
        found.is_empty(),
        "the runner is one job at a time for the whole process, and a crate's tests run as threads of it, so a \
         control that takes it fails while another control's job holds it: declare `static OWN: \
         crate::standards_review_run::Containment = crate::standards_review_run::Containment::new();` and call \
         `review_item_in(&OWN, ..)`, or give the real handlers that runner with `with_review_runner(&OWN)` (issues \
         6499 and 6500):\n{}",
        found.join("\n")
    );
}

/// Every file the scan exempts is still a test-only source of this crate,
/// so the list shrinks when a file moves rather than excusing nothing.
#[test]
fn every_exempt_file_is_a_test_only_source_of_this_crate() {
    let sources: Vec<SourceFile> = sources();
    let stale: Vec<String> = NAMES_THE_CALLS_WITHOUT_MAKING_THEM
        .iter()
        .filter(|(named, _)| {
            return !sources
                .iter()
                .any(|file| return file.test_only && file.path == *named);
        })
        .map(|(named, why)| return format!("{named}: {why}"))
        .collect();
    assert!(
        stale.is_empty(),
        "these entries name no test-only source of this crate, so remove them:\n{}",
        stale.join("\n")
    );
}

/// The scan fires on the shared runner's calls and on the handlers' review route,
/// and spares what a control owns,
/// so a check that stopped reading would fail here rather than passing every tree.
#[test]
fn the_scan_finds_a_planted_call_and_spares_the_one_a_control_owns() {
    let root: std::path::PathBuf = crate::test_support::fixture("source-guard-production-runner");
    write_tree(
        root.as_path(),
        &[
            (
                "lib.rs",
                "#[cfg(test)]\nmod handler_tests;\n#[cfg(test)]\nmod handler_with_a_runner_tests;\n\
                 #[cfg(test)]\nmod helper_tests;\n",
            ),
            (
                "handler_tests.rs",
                "/// A control that reviews through the real handlers, which take the shared runner.\n\
                 fn a_control() {\n\
                 static OWN: Containment = Containment::new();\n\
                 let handlers = GitHandlers::new(context).with_review_endpoint(\"http://127.0.0.1:9\");\n\
                 assert_eq!(handlers.review(&item), Verdict::Done);\n\
                 }\n",
            ),
            (
                "handler_with_a_runner_tests.rs",
                "/// A control that hands the real handlers a runner of its own.\n\
                 fn a_control() {\n\
                 static OWN: Containment = Containment::new();\n\
                 let handlers = GitHandlers::new(context).with_review_endpoint(\"http://127.0.0.1:9\")\n\
                 .with_review_runner(&OWN);\n\
                 assert_eq!(handlers.review(&item), Verdict::Done);\n\
                 }\n",
            ),
            (
                "helper_tests.rs",
                "/// A comment that names review_item( is not a call.\n\
                 fn a_control() {\n\
                 static OWN: Containment = Containment::new();\n\
                 assert_eq!(run_contained_in(&OWN, || return 7_u8, roomy), Contained::Finished(7));\n\
                 assert_eq!(review_item_in(&OWN, &context, &item, options, ceiling), Verdict::Done);\n\
                 assert_eq!(review_item(&context, &item, options), Verdict::Done);\n\
                 }\n",
            ),
        ],
    );
    let found: Vec<String> = findings_in(
        root.as_path(),
        &[crate::source_scan_tests::release_root("lib.rs")],
    )
    .expect("the fixture sources");
    assert_eq!(
        found,
        vec![
            String::from(
                "handler_tests.rs:4 names `with_review_endpoint(`, which points the real handlers' review at a \
                 gateway, so their reviews run in `RUNNER` unless the control hands them one; declare `static OWN: \
                 crate::standards_review_run::Containment = crate::standards_review_run::Containment::new();` and \
                 write `with_review_runner(&OWN)` beside it"
            ),
            String::from(
                "helper_tests.rs:6 names `review_item(`, which runs a work item's review in `RUNNER`"
            ),
        ],
        "the planted call or the planted route was not found, or a control's own runner was"
    );
    crate::test_support::remove(root.as_path());
}
