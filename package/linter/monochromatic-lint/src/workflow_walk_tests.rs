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

/// Import the walk and its records.
use super::walk;
use crate::workflow_model::{RunBody, WalkError, Workflow};

/// Walk a fixture that must be readable.
fn read(text: &str) -> Workflow {
    return walk(text).expect("readable workflow");
}

/// Walk a fixture that must be refused.
fn refused(text: &str) -> WalkError {
    return walk(text).expect_err("refused workflow");
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
    assert!(walk("\"on\": push\njobs:\n  a:\n    steps:\n      - run: x\n").is_ok());
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
        assert!(walk(text).is_err(), "{text:?}");
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
        walk("jobs:\n  a:\n    steps:\n      - run: x\n        shell: |\n          bash\n")
            .is_err()
    );
    assert!(walk("defaults:\n  run:\n    shell: [bash]\n").is_err());
}

/// Document structure the walk does not model is refused.
#[test]
fn documents_and_tabs() {
    assert!(refused("a: 1\n---\nb: 2\n").message.contains("second"));
    assert!(refused("a: 1\n...\n").message.contains("end marker"));
    assert!(refused("jobs:\n\ta: 1\n").message.contains("tab"));
    assert!(walk("%YAML 1.2\n---\njobs:\n  a:\n    steps:\n      - run: x\n").is_ok());
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
