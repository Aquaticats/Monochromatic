# Mise 2026.10.9 cannot discover an excluded research task from the repository root

## Symptom

Native Depot run `b7zbgvws7h`,
at harness `aec580b2ddbc62126eb9264d8d8cff36cd8f1c06`,
successfully admitted the frozen source and restored tools.
Its first `depot tests run` command then failed before Nextest started:

```text
mise ERROR no task //.depot/research/test-shards:execute-selected found
exit status 1; additionally, no JUnit XML report files matched
```

The missing report follows the missing command.
It does not establish a JUnit parser failure,
test failure or failed runner allocation.
The original 3,421-byte ZIP is retained as artifact `01a14e8d-78a4-7613-b667-b60092563940`,
SHA256 `0cd4e97353d57452b41a3ee11f1ee0eadaa35051afe5da1b9fd22e2b5eea575e`.

## Root cause

The repository's `mise.toml:26` explicitly restricts discovered configuration roots:

```toml
# mise.toml
config_roots = [
  "package/*/*",
]
```

The research configuration lives under `.depot/research/test-shards`.
It is outside that discovery set.
In Mise `v2026.10.9`,
commit `ac91d06fa6100935b7a07b08bebf03dd6dd9a0dd`,
`src/config/mod.rs:5466` chooses explicit roots instead of walking the tree:

```rust
//! jdx/mise,
//! src/config/mod.rs:5466
if let Some(patterns) = config_roots
    && !patterns.is_empty()
{
    return expand_config_roots(root, patterns, ctx);
}
```

The outer research Mise task resolves from its own configuration,
then changes its execution directory to the repository root.
The child Depot process inherited that directory.
Its `--command` started another Mise process there,
where the research task was undiscovered.
The fully qualified task name does not itself load an excluded configuration.

## Verification

The failing catalog is the actual first seed invocation in `b7zbgvws7h`.
Depot selected all 34 candidates,
but Mise never executed the selected-test task.
No application test or report upload receives passing credit from that invocation.

The working catalog is run `fm9d4wkbj9`,
harness `e1b2f76f62fbb7fea99eef4fca88443d54308455`.
All three 34-case seed executions passed.
Separate physical jobs then executed 17 cases each,
and their downstream consumer verified the exact disjoint 34-case union.
The CLI reported successful JUnit ingestion for each execution.
All owned tmpfs mounts were unmounted.
Historical timing coverage remained zero in both shard commands;
that separate lookup boundary is not repaired by this directory change.

Local task discovery and JavaScript syntax checks also passed from the owning configuration directory.
Those checks alone had not exercised the child command's inherited directory.

## Verified workaround

Keep the task name unchanged and launch the Depot child from the directory owning its Mise configuration:

```javascript
// .depot/research/test-shards/mise.toml, run-cohort task on the research branch
const result = spawnSync('depot', args, {
  cwd: base,
  env: { ...process.env, PROBE_REPORT_LABEL: label },
  encoding: 'utf8',
});
```

Here `base` is the absolute `.depot/research/test-shards` directory.
The selected task still uses its explicit Mise `dir` to execute from the repository root.
Candidate and report paths are absolute,
so changing the Depot child's directory does not change those inputs.
The tradeoff is that this research harness depends on explicit directory ownership;
its configuration remains deliberately outside normal package discovery.

## What does not work

Checking the selected task only from its owning directory missed the actual child-process boundary.
Leaving the child at the repository root reproduces the missing-task diagnostic.
Adding the entire research directory to production task discovery is unnecessary for this experiment
and would enlarge the active task graph.

## Upstream filing decision

Nothing to add upstream.
The consumer selected the wrong working directory for the repository's explicit configuration scope.

- Responsibility:
  the research launcher,
  rather than an established Mise defect.
- Fixability:
  the committed one-field launcher change passed actual cloud execution.
- Supported use:
  explicit configuration roots are the deciding source path.
- Contribution policy:
  the inspected `CONTRIBUTING.md` restricts AI replies on other people's issues and discussions.
  No upstream reply or change is proposed.
- Maintainer intent:
  the source deliberately uses explicit roots instead of walking every directory.
- Minimal patch:
  the owned launcher change is verified;
  no upstream prototype is required.

The `.out-of-scope/` inventory has no matching Mise exemption.
No upstream issue,
comment or message was sent.
