# Mise 2026.10.0 parses a GitHub expression inside an inline JavaScript task as a task template

## Symptom

A research task generating GitHub workflow JSON fails before its JavaScript starts:

```text
mise ERROR failed to validate task prepare-job-policy-history
mise ERROR invalid task script template for task prepare-job-policy-history
mise ERROR Found `)` but expected =.
```

The triggering JavaScript string contains `${{ toJSON(needs) }}`.
JavaScript quotes do not protect that string from Mise's earlier template pass.
The observed local tool is Mise `2026.10.0`.

## Root cause

The exact release source is commit `bc11f90c74eba23bf0d7350efb540e62fb7d9ffd`.
`crates/mise-util/src/tera.rs:52` detects marker text without parsing JavaScript string boundaries:

```text
# jdx/mise, crates/mise-util/src/tera.rs:52
pub fn contains_template_syntax(input: &str) -> bool {
    input.contains("{{") || input.contains("{%") || input.contains("{#")
}
```

`src/task/task_executor.rs:2052` validates each marked script during task preflight:

```text
# jdx/mise, src/task/task_executor.rs:2052
task.validate_template_syntax_for_preflight(&script)
    .wrap_err_with(|| {
        format!("invalid task script template for task {}", task.name)
    })?;
```

`src/task/mod.rs:2285` delegates to `TaskScriptParser`.
`src/task/task_script_parser.rs:665` then invokes the template engine:

```text
# jdx/mise, src/task/task_script_parser.rs:665
let (mut tera, _, _, _) = self.setup_tera_for_spec_parsing(task);
tera.validate_template_syntax("__mise_validate", input)
    .map_err(Self::task_script_tera_error)
```

The JavaScript helper has not received control when that syntax diagnostic is emitted.

## Verification

The failing catalog is the actual first `mise run prepare-job-policy-history` invocation.
It fails during validation,
before the task's file-writing code.
Its literal GitHub expression is the marked input.

The working catalog constructs the opening braces at JavaScript runtime.
The actual corrected Mise task generated the committed workflow field
`${{ toJSON(needs) }}`,
and both the GitHub and native consumers received valid dependency JSON.
GitHub `39122132487` completed successfully;
native `kjf78stmkb` completed its consumer successfully.
Native's separate aggregate continuation difference is documented in
[`depot-job-level-continue-on-error.md`](depot-job-level-continue-on-error.md).

## Verified workaround

For this generator,
construct only the destination expression's opening delimiter at runtime:

```javascript
// Private research controller, prepare-job-policy-history task
const needsExpression = '$' + '{' + '{ toJSON(needs) }}';
```

The task source no longer contains adjacent opening braces for that expression,
while the generated JSON retains the exact GitHub expression.
The tradeoff is less direct generator source;
the generated workflow remains ordinary GitHub syntax.
This does not disable Mise templating for other task inputs.

## What does not work

Putting the GitHub expression in a JavaScript single-quoted string still triggers Mise's marker detection.
Syntax-checking the JavaScript alone cannot validate the preceding Mise template pass.
No third-party source,
global setting or unrelated task template was modified.

### Silent runner-label corruption recurrence

The 2026-10-18 guest-file generator used an opening delimiter containing adjacent braces
inside its JavaScript expression helper.
Mise accepted that input but replaced the quoted template content before Node evaluated it.
The generated workflow contained literal `$ + value + ` fields instead of GitHub expressions.
Run `39132577785`,
source `3829831039cc43aa717c6c927f835b867e9313b6`,
remained queued with both job labels reported as `$ + value +`,
empty runner assignments and no execution result.
It is not a measured VM consumer.

The corrected generator constructs both delimiters without adjacent braces in the task source:

```javascript
// Private research controller, repair-mvm-expression-generator task
const expression = value => String.fromCharCode(36, 123, 123)
  + ' ' + value + ' ' + String.fromCharCode(125, 125);
```

The resulting workflow requires a separate destination-field check before dispatch.
JavaScript syntax validation and task discovery alone passed the faulty generator;
neither checks the final runner label,
provider fields or artifact-name expressions.
The new `validate:workflow` owner checks all six destination expressions and the finite matrix settings.
The corrected cloud replay remains distinct from the queued attempt.

## Upstream filing decision

Nothing to add upstream.
This is a consumer generator crossing two documented template grammars.

- Responsibility:
  the generator embedded destination-template syntax directly into an earlier templated input.
- Fixability:
  the owned delimiter construction passed real task and workflow execution.
- Supported use:
  Mise intentionally recognizes template markers in task scripts.
- Contribution policy:
  the inspected `CONTRIBUTING.md` restricts AI replies on other people's issues and discussions.
  No reply or upstream change is proposed.
- Maintainer intent:
  the marker detector and preflight validator explicitly implement this behavior.
- Minimal patch:
  the owned generator change is sufficient;
  no upstream prototype is needed.
