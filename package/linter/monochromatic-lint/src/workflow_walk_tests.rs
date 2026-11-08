//! What:
//!  Controls for the structural workflow walk.
//! Why:
//!  The walk decides which `run:` is a step's script,
//!  and a wrong decision either invents a step or hides one from the rules.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('workflow walk', () => { /* steps, defaults, runs-on, refusals */ });
//! ```

/// Import the walk,
///  the walker whose value skipping is checked directly,
///  its records and the deadline that ends a walk which stops advancing.
use super::{Walker, walk};
use crate::test_deadline::finished_within_deadline;
use crate::workflow_lines::{Content, parse_entry, read_lines};
use crate::workflow_model::{RunBody, WalkError, Workflow};

/// Walk a fixture under the deadline,
///  so a walk that stops advancing fails the test instead of hanging it.
fn walk_bounded(text: &str) -> Result<Workflow, WalkError> {
    let owned: String = String::from(text);
    return finished_within_deadline(move || return walk(owned.as_str()));
}

/// Walk a fixture that must be readable.
fn read(text: &str) -> Workflow {
    return walk_bounded(text).expect("readable workflow");
}

/// Walk a fixture that must be refused.
fn refused(text: &str) -> WalkError {
    return walk_bounded(text).expect_err("refused workflow");
}

/// The script text of a step as one string,
///  for comparison.
fn script(text: &str, workflow: &Workflow, index: usize) -> String {
    match workflow.steps[index].body.as_ref().expect("body") {
        RunBody::Plain { start, end } => return String::from(&text[*start..*end]),
        RunBody::Block { lines } => {
            let mut joined: String = String::new();
            for line in lines {
                joined.push_str(&text[line.payload..line.end]);
            }
            return joined;
        }
    }
}

/// A workflow with plain and block steps in two jobs.
#[test]
fn steps_belong_to_their_jobs() {
    let text: &str = "name: ci\non: push\njobs:\n  a:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: echo one\n      - name: block\n        run: |\n          echo two\n          echo three\n        shell: bash\n  b:\n    steps:\n      - run: echo four\n";
    let workflow: Workflow = read(text);
    assert_eq!(workflow.jobs.len(), 2);
    assert_eq!(workflow.steps.len(), 4);
    assert_eq!(workflow.steps[0].body, None);
    assert_eq!(script(text, &workflow, 1), "echo one");
    assert_eq!(script(text, &workflow, 2), "echo two\necho three\n");
    assert_eq!(
        workflow.steps[2].shell.as_ref().expect("shell").value,
        "bash"
    );
    assert_eq!(script(text, &workflow, 3), "echo four");
    assert_eq!(workflow.steps[0].job, Some(0));
    assert_eq!(workflow.steps[1].job, Some(0));
    assert_eq!(workflow.steps[2].job, Some(0));
    assert_eq!(workflow.steps[3].job, Some(1));
}

/// A `run` key that is not a step's key is never a script.
#[test]
fn run_keys_elsewhere_are_not_steps() {
    let text: &str = "defaults:\n  run:\n    shell: bash\njobs:\n  a:\n    defaults:\n      run:\n        shell: pwsh\n    env:\n      run: not a script\n    steps:\n      - uses: x\n        with:\n          run: also not a script\n          script: |\n            - run: nor this\n            run: nor this\n      - run: real\n";
    let workflow: Workflow = read(text);
    assert_eq!(workflow.steps.len(), 2);
    assert_eq!(workflow.steps[0].body, None);
    assert_eq!(script(text, &workflow, 1), "real");
    assert_eq!(
        workflow.shell.as_ref().expect("workflow shell").value,
        "bash"
    );
    assert_eq!(
        workflow.jobs[0].shell.as_ref().expect("job shell").value,
        "pwsh"
    );
}

/// `runs-on` is kept lowercase in inline and sequence form.
#[test]
fn runs_on_is_captured() {
    let inline: Workflow =
        read("jobs:\n  a:\n    runs-on: Windows-2022\n    steps:\n      - run: x\n");
    assert!(inline.jobs[0].runs_on.contains("windows"));
    let listed: Workflow = read(
        "jobs:\n  a:\n    runs-on:\n      - self-hosted\n      - Windows\n    steps:\n      - run: x\n  b:\n    runs-on: ubuntu-latest\n",
    );
    assert!(listed.jobs[0].runs_on.contains("windows"));
    assert!(!listed.jobs[1].runs_on.contains("windows"));
    let matrix: Workflow =
        read("jobs:\n  a:\n    runs-on: ${{ matrix.runner }}\n    steps:\n      - run: x\n");
    assert_eq!(matrix.jobs[0].runs_on, "${{ matrix.runner }}");
}

/// A composite action's steps have no job.
#[test]
fn composite_actions() {
    let text: &str = "name: setup\nruns:\n  using: composite\n  steps:\n    - run: echo a\n      shell: bash\n    - uses: x\n";
    let workflow: Workflow = read(text);
    assert_eq!(workflow.steps.len(), 2);
    assert_eq!(workflow.steps[0].job, None);
    assert_eq!(
        workflow.steps[0].shell.as_ref().expect("shell").value,
        "bash"
    );
    assert_eq!(script(text, &workflow, 0), "echo a");
}

/// A sequence may sit at the same indentation as its key.
#[test]
fn flush_sequences() {
    let text: &str = "jobs:\n  a:\n    steps:\n    - run: one\n    - run: |\n        two\n    other: 1\n  b:\n    steps:\n    - run: three\n";
    let workflow: Workflow = read(text);
    assert_eq!(workflow.steps.len(), 3);
    assert_eq!(script(text, &workflow, 0), "one");
    assert_eq!(script(text, &workflow, 1), "two\n");
    assert_eq!(workflow.steps[2].job, Some(1));
}

/// A dash alone starts a step whose keys follow on later lines.
#[test]
fn dash_alone_starts_a_step() {
    let text: &str =
        "jobs:\n  a:\n    steps:\n      -\n        run: one\n      - # comment\n        run: two\n";
    let workflow: Workflow = read(text);
    assert_eq!(workflow.steps.len(), 2);
    assert_eq!(script(text, &workflow, 1), "two");
}

/// Comments,
///  blank lines,
///  a leading document marker,
///  multi-line flow values and quoted values do not disturb the walk.
#[test]
fn noise_is_skipped() {
    let text: &str = "---\n# top\n\non:\n  push:\n    branches: [\n      main,\n      \"feature: x\",\n    ]\n  schedule:\n    - cron: '0 0 * * *'\njobs:\n  a:\n    if: >\n      run: not a step\n      still not\n    name: \"multi\n      line: x\"\n    steps:\n      # between\n      - run: one # trailing\n\n      - run: two\n";
    let workflow: Workflow = read(text);
    assert_eq!(workflow.steps.len(), 2);
    assert_eq!(script(text, &workflow, 0), "one");
    assert_eq!(script(text, &workflow, 1), "two");
}

/// A top-level quoted key such as `"on"` is fine;
///  inside `jobs` it is refused.
#[test]
fn odd_keys_are_refused_inside_structure() {
    assert!(walk_bounded("\"on\": push\njobs:\n  a:\n    steps:\n      - run: x\n").is_ok());
    for text in [
        "jobs:\n  \"a\":\n    steps:\n      - run: x\n",
        "jobs:\n  a:\n    steps:\n      - \"run\": x\n",
        "jobs:\n  a:\n    steps:\n      - <<: *base\n",
        "jobs:\n  a:\n    steps:\n      - &s run: x\n",
        "runs:\n  steps:\n    - *alias : x\n",
    ] {
        let error: WalkError = refused(text);
        assert!(
            error.message.contains("plain key"),
            "{text:?}: {}",
            error.message
        );
    }
}

/// A flow mapping in a build matrix,
///  and a quoted key in an environment or input mapping,
///  hold no step and are ignored.
#[test]
fn odd_keys_outside_steps_are_ignored() {
    let text: &str = "jobs:\n  a:\n    strategy:\n      matrix:\n        include:\n        - { os: linux, runner: ubuntu-24.04 }\n        # note\n        - { os: windows, runner: windows-2025 }\n    env:\n      \"A B\": x\n    steps:\n      - run: one\n        env:\n          'Q': y\n          <<: *shared\n";
    let workflow: Workflow = read(text);
    assert_eq!(workflow.steps.len(), 1);
    assert_eq!(script(text, &workflow, 0), "one");
}

/// Structure written inline,
///  and steps that are not block mappings,
///  are refused.
#[test]
fn inline_structure_and_scalar_steps_are_refused() {
    for text in [
        "jobs: {}\n",
        "jobs:\n  a: *job\n",
        "jobs:\n  a:\n    steps: [{run: x}]\n",
        "jobs:\n  a:\n    steps: *s\n",
        "jobs:\n  a:\n    defaults: {run: {shell: bash}}\n",
        "runs: {using: composite}\n",
        "jobs:\n  a:\n    steps:\n      - run\n",
        "jobs:\n  a:\n    steps:\n      - &anchor\n        run: x\n",
        "jobs:\n  a:\n    steps:\n      - {run: x}\n",
        "jobs:\n  a:\n    steps:\n      - [x]\n",
    ] {
        assert!(walk_bounded(text).is_err(), "{text:?}");
    }
}

/// Run and shell values the readers refuse stop the walk,
///  with the offset of the value.
#[test]
fn value_refusals_surface() {
    let text: &str = "jobs:\n  a:\n    steps:\n      - run: \"quoted\"\n";
    assert_eq!(
        refused(text).offset,
        text.find("\"quoted\"").expect("quote")
    );
    assert!(
        walk_bounded("jobs:\n  a:\n    steps:\n      - run: x\n        shell: |\n          bash\n")
            .is_err()
    );
    assert!(walk_bounded("defaults:\n  run:\n    shell: [bash]\n").is_err());
}

/// Document structure the walk does not model is refused.
#[test]
fn documents_and_tabs() {
    assert!(refused("a: 1\n---\nb: 2\n").message.contains("second"));
    assert!(refused("a: 1\n...\n").message.contains("end marker"));
    assert!(refused("jobs:\n\ta: 1\n").message.contains("tab"));
    assert!(walk_bounded("%YAML 1.2\n---\njobs:\n  a:\n    steps:\n      - run: x\n").is_ok());
}

/// Mapping and sequence keys after a step do not leak into it.
#[test]
fn later_keys_do_not_leak_into_the_last_step() {
    let text: &str = "jobs:\n  a:\n    steps:\n      - run: one\n    env:\n      shell: not a step key\n      run: not a step\n";
    let workflow: Workflow = read(text);
    assert_eq!(workflow.steps.len(), 1);
    assert_eq!(workflow.steps[0].shell, None);
    assert_eq!(script(text, &workflow, 0), "one");
}

/// Where the walker resumes after the value of the key on the first line,
///  as a line index.
fn resumes_at(text: &str) -> usize {
    let walker: Walker<'_> = Walker {
        lines: read_lines(text).expect("lines"),
        frames: Vec::new(),
        found: Workflow::default(),
        runs_on: None,
        content_seen: false,
    };
    let entry = parse_entry(&walker.lines[0]);
    let Content::Key { column, value, .. } = entry.content else {
        panic!("fixture has no key");
    };
    return walker.skip_value(0, column, value);
}

/// A key with no value leaves its lines to the walk,
///  and a one-line flow collection ends on its own line.
#[test]
fn an_empty_value_and_a_one_line_flow_collection_end_on_their_own_line() {
    assert_eq!(resumes_at("k:\n  a\n  b\nnext: 1\n"), 1);
    assert_eq!(resumes_at("k: { a: 1 }\nnext: 1\n"), 1);
    assert_eq!(resumes_at("k: [a, b]\nnext: 1\n"), 1);
    assert_eq!(resumes_at("k: {}\nnext: 1\n"), 1);
    assert_eq!(resumes_at("k: [a, b] # ]\nnext: 1\n"), 1);
}

/// A flow collection runs to the line where its brackets balance,
///  across nested brackets of both kinds,
///  or to the end of the file when they never do.
#[test]
fn a_flow_collection_runs_to_the_line_where_its_brackets_balance() {
    assert_eq!(
        resumes_at("k: {\n  a: [1,\n    2],\n  b: 3\n}\nnext: 1\n"),
        5
    );
    assert_eq!(resumes_at("k: [ { a: 1 },\n  { b: 2 } ]\nnext: 1\n"), 2);
    assert_eq!(resumes_at("k: [a,\n  b\n"), 2);
    assert_eq!(resumes_at("k: [a,\n  b\nnext: 1\n"), 3);
}

/// A scalar continues over deeper lines and over blank and comment lines,
///  and stops at the first other line that is no deeper than its key.
#[test]
fn a_scalar_continues_over_deeper_lines_and_skippable_lines() {
    assert_eq!(
        resumes_at("k: plain # c\n  more\n\n# note\n  more2\nnext: 1\n"),
        5
    );
    assert_eq!(resumes_at("k: |\n  text\n  more\nnext: 1\n"), 3);
    assert_eq!(resumes_at("k: \"quoted\n  continues\"\nnext: 1\n"), 2);
    assert_eq!(resumes_at("k: x\nnext: 1\n"), 1);
    assert_eq!(resumes_at("k: x\n  more\n"), 2);
}

/// A scalar value at a deeper column stops at its own sibling.
#[test]
fn a_nested_scalar_stops_at_its_own_sibling() {
    let text: &str = "a:\n  k: x\n    deeper\n  # top\n    deeper still\n  sibling: 1\n";
    let walker: Walker<'_> = Walker {
        lines: read_lines(text).expect("lines"),
        frames: Vec::new(),
        found: Workflow::default(),
        runs_on: None,
        content_seen: false,
    };
    let entry = parse_entry(&walker.lines[1]);
    let Content::Key { column, value, .. } = entry.content else {
        panic!("fixture has no key");
    };
    assert_eq!(walker.skip_value(1, column, value), 5);
}

/// A quoted key next to a plain sibling inside structure is still refused,
///  because only the keys above its column are its path.
#[test]
fn an_odd_key_is_judged_by_the_keys_above_its_column() {
    for text in [
        "jobs:\n  a:\n    runs-on: x\n    \"b\": y\n    steps:\n      - run: x\n",
        "jobs:\n  a:\n    steps:\n      - run: x\n        shell: bash\n        'q': y\n",
    ] {
        let error: WalkError = refused(text);
        assert!(
            error.message.contains("plain key"),
            "{text:?}: {}",
            error.message
        );
    }
}

/// A step is anchored at its dash.
#[test]
fn a_step_is_anchored_at_its_dash() {
    let text: &str = "jobs:\n  a:\n    steps:\n      - run: x\n      - run: y\n";
    let workflow: Workflow = read(text);
    let dashes: Vec<usize> = text
        .match_indices("- run")
        .map(|(offset, _)| return offset)
        .collect::<Vec<usize>>();
    let anchors: Vec<usize> = workflow
        .steps
        .iter()
        .map(|step| return step.anchor)
        .collect::<Vec<usize>>();
    assert_eq!(anchors, dashes);
}

/// A structural key written with a value is refused at the value's own byte.
#[test]
fn a_refused_structural_value_is_reported_at_its_own_byte() {
    let text: &str = "name: ci\njobs:\n  a: *job\n";
    let error: WalkError = refused(text);
    assert_eq!(error.offset, text.find("*job").expect("the value"));
}
