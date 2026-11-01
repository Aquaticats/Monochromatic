# cli-git CAC migration assessment handover

## Status

Assessment started on 2026-08-14.
The user narrowed this session to CAC only.
No other external CLI parser technology may be discovered,
audited,
scored,
or ranked.
No recommendation or adoption decision exists yet.
No product code,
dependency,
configuration,
or decision-record change is authorized by this assessment.

Keep this handover current when the evidence phase,
requirements,
hypotheses,
findings,
rejected paths,
or next action changes.
Detailed evidence belongs in
`doc/audit/tech-cli-git-parser-migration-to-cac-vet-2026-08-14-a48b54e2.md`.
The unqualified same-day report is the superseded broader-scope record and must not receive new evidence.

## User request

Assess the practicality of migrating `package/git-policy/cli/` to the open-source npm CLI framework CAC.
Write this handover immediately and refresh it during the assessment.

The practical question is not only whether CAC can parse one command.
The assessment must determine whether it can replace the consumed parser surfaces without weakening Git argv fidelity,
management-command diagnostics,
policy safety,
package distribution,
or platform behavior.

## Authority boundary

This is an evaluation request.
Documentation and reports may change in the main worktree.
Product code,
dependencies,
configuration,
build artifacts,
and lockfiles must not change.
Any migration probe must use a disposable fixture or throwaway worktree.
A later explicit adoption request is required before implementation or a decision record.

## Initial repository checkpoint

- Baseline branch:
  `main`.
- Baseline commit:
  `54ad78083e8baf95c62d3b0682843967722c563d`.
- Target package:
  `package/git-policy/cli/`.
- `package/git-policy/cli/package.json` currently declares neither CAC nor Optique.
- Its runtime dependencies are `nano-spawn`,
  `rolldown`,
  `type-fest`,
  `typescript`,
  `valibot`,
  `yuku-ast`,
  and `yuku-parser`.
- `package/git-policy/cli/src/management-parser.ts`
  (moved on 2026-10-13 with the TypeScript wrapper: now `package/git-policy/cli/src/native/management_arguments.rs`)
  describes a repository-owned management-command grammar
  that replaced
  the `@optique/core` facade.
- `package/git-policy/cli/src/parser/argv.ts`
  (moved on 2026-10-13 with the TypeScript wrapper: now `package/git-policy/cli/src/native/command_options.rs`)
  describes a repository-owned Git argv region parser that replaced
  `@optique/core` option parsing.
- Several source comments still use Optique terminology.
  These comments are historical signals,
  not evidence that Optique remains installed.
- The existing platform decision is `doc/decision/cli-git-policies-platform.md`.
- The existing implementation-state handover is `doc/handover/cli-git-policies-platform.md`.
- Unrelated modified files existed before this assessment:
  `.serena/project.yml`,
  `package/music-player/desktop-app/src/ui_binding_tests.rs`
  (deleted 2026-10-11, no same-name replacement; the UI tests are the `ui_*_tests.rs` files in that directory),
  and `package/music-player/desktop-app/ui/app.slint`.
  Do not stage,
  restore,
  stash,
  or otherwise disturb them.

## Governing evaluation method

The assessment follows `.agents/skills/choosing-technology/SKILL.md` at commit
`a05818ad70a40e5769a36de669697ba109891b31`,
SHA-256
`393eb68c5b2b2f7b16c8f7f90c100fb8be43eefa4501511360cd0572e4ae8087`.

CAC is an inspectable open-source local technology.
The applicable overlays are:

- incumbent replacement,
  where the incumbent is the current repository-owned parser set;
- high-trust execution,
  because cli-git shadows `git`,
  enforces safety policies,
  and runs in hooks and automation;
- multi-platform behavior,
  subject to the package's actual supported platform contract;
- human auditability,
  because parser behavior controls whether commands are blocked,
  transformed,
  or forwarded to real Git.

The repository-owned parser set remains only as CAC's replacement-parity baseline.
No other external CLI technology is in scope.
CAC cannot receive a practicality conclusion from README or registry metadata alone.

## Current parser map

The target does not have one replaceable parser boundary.
The measured source map at baseline is:

- `package/git-policy/cli/src/parse-global-options.ts`
  (moved on 2026-10-13 with the TypeScript wrapper: now `package/git-policy/cli/src/native/global_arguments.rs`)
  locates the real Git subcommand,
  applies ordered `-C` semantics,
  and preserves global help and version short-circuits.
- `package/git-policy/cli/src/management-parser.ts`
  (moved on 2026-10-13 with the TypeScript wrapper: now `package/git-policy/cli/src/native/management_arguments.rs`)
  implements the closed `git cli-git` grammar.
  It is 265 physical lines,
  including declarations and TSDoc.
- `package/git-policy/cli/src/parser/argv.ts`
  (moved on 2026-10-13 with the TypeScript wrapper: now `package/git-policy/cli/src/native/command_options.rs`)
  is the shared subset parser used by management and Git-region parsers.
  It is 415 physical lines and has eighteen direct unit cases.
- `package/git-policy/cli/src/parser/` contains twenty production TypeScript files and 3,953 physical lines.
  Most of this surface is Git-specific classification rather than generic command routing.
- `parseArgv` or `tryParseArgv` is consumed by management plus the `add`,
  `clean`,
  `reset`,
  `stash`,
  `commit`,
  `push`,
  and `status` regions.
- Branch creation uses a separate linear scan over `branch`,
  `checkout`,
  and `switch` because it must model creation modes,
  Git's accepted option abbreviations,
  option arity,
  and implicit remote-branch guessing.
- Commit parsing separately normalizes attached short-option values,
  identifies pathspecs,
  and extracts transaction paths.
- Clean parsing separately resolves left-to-right positive and negated mode toggles.
- Add parsing separately distinguishes broad staging tokens from option values and pathspecs.

A technology result therefore needs an integration shape:

- keep the owned parsers;
- use CAC only for the closed management namespace;
- use CAC for management plus shared Git-region tokenization while retaining Git-specific scanners;
- or replace every parser path with CAC plus adapters.

The last shape cannot be credited
with deleting the complete parser directory unless a prototype demonstrates the Git-specific
facts without recreating those scans around CAC.

## Frozen hard constraints

These constraints come from current source,
`package/git-policy/cli/SPEC.md`,
`doc/decision/cli-git-policies-platform.md`,
and maintained CI or performance contracts.
A candidate or integration shape that fails one is ineligible.

- Preserve exact wrapper argv until an explicit cli-git transform or escape removal owns a change.
- Preserve Git global-option layout,
  chained `-C`,
  and real Git help or version short-circuits before management dispatch.
- Preserve the complete management grammar in `package/git-policy/cli/SPEC.md`.
- Preserve successful namespace and trust help on stdout with exit `0`,
  before real-Git resolution,
  transaction recovery,
  config discovery,
  candidate building,
  or trust-registry access.
- Preserve usage failures on stderr with exit `2` and no repository config load.
- Preserve pathspec bytes after `--` without wrapper reinterpretation.
- Preserve repeated `--policy` values,
  stable first-occurrence deduplication,
  exact scope validation,
  and unknown-option rejection for management commands.
- Preserve Git-region subset behavior where undeclared options remain forwardable,
  declared values may begin with `-`,
  options may follow positionals,
  joined values are classified correctly,
  and a missing declared value fails closed.
- Preserve Git-supported long-option abbreviations and short-option clusters wherever current guards depend on them.
- Preserve fail-closed policy facts for ambiguous or malformed guarded invocations.
- Keep the package-root import side-effect free and the packed package to one self-contained MJS artifact.
- Support the package's Node range,
  `^22.18.0 || >=24.11.0`.
- Keep Linux,
  macOS,
  and Windows compatibility exercised by the cli-git host-evidence matrix.
- Remain within every maintained lifecycle budget.
  The current `wide-commit` wrapper-added ceiling is 925 milliseconds for 256 changed paths in the bounded benchmark.
- Pass the complete relevant upstream suite and cli-git consumer-boundary suite from a disposable,
  secret-free environment.
- Use inspectable source with compatible license,
  source-to-package provenance,
  and no unaudited native,
  Wasm,
  downloaded,
  or generated runtime boundary.

## Frozen soft criteria

The user supplied no priority ordering,
so every applicable criterion has weight 1 before candidate-specific evidence is rated:

- net removal of repository-owned parser and adapter code;
- human auditability of the resulting parse and forwarding boundary;
- runtime and bundle overhead within the hard budget;
- direct and transitive runtime dependency surface;
- TypeScript inference and declaration compatibility;
- help and diagnostic control without process-global interception;
- upstream maintenance and release health;
- migration and regression-test burden;
- future management-command extensibility;
- fit with the existing pure-parser plus process-adapter seam.

A structurally avoided risk receives a strong evidenced rating.
No soft score can offset a hard-gate failure.

## Questions the evidence must answer

- Which cli-git parsing surfaces are true framework candidates,
  and which are Git-specific scanners that a general CLI framework should not own?
- Can CAC preserve tokens after `--`,
  repeated values,
  option values beginning with `-`,
  Git global options,
  long-option abbreviations,
  clustered short options,
  wrapper-only escapes,
  and unknown or future Git syntax where cli-git must fail conservatively or forward unchanged?
- Can CAC express the `git cli-git` management grammar and its exact stdout,
  stderr,
  help,
  and exit-code contracts without relying on process-global side effects?
- Does CAC permit pure parsing with injectable argv and output,
  or would integration require intercepting process mutation or termination?
- What runtime dependency,
  transitive dependency,
  source-size,
  maintenance,
  license,
  release,
  and package-artifact changes would migration introduce?
- Does CAC's TypeScript surface preserve cli-git's static command
  and option guarantees under this repository's compiler and
  declaration settings?
- Does a migration delete more repository-owned parser complexity than the adapter and compatibility code it adds?
- Which current tests establish replacement parity,
  and which consumer-boundary fixtures are missing?

## Assessment sequence

1. Inventory every parser entry point,
   caller,
   behavior branch,
   test,
   and public diagnostic contract in `package/git-policy/cli/`.
2. Freeze hard constraints and equal-weight soft criteria from measured repository evidence.
3. Freeze and run the CAC-only discovery query schedule.
4. Screen CAC through license,
   provenance,
   security,
   platform,
   source,
   maintenance,
   and auditability gates.
   Use the incumbent only for replacement parity.
5. Clone serious external candidates under private `~/temp/agent/` paths and inspect the consumed source paths.
6. Validate finalists equally,
   then exercise relevant cli-git boundaries in secret-free disposable fixtures.
7. Score validated finalists,
   run sensitivity checks,
   and publish pros,
   cons,
   a complete ranking,
   and either a recommendation or an explicit no-recommendation result.
8. Synchronize this handover after each material evidence phase and complete the CAC-only vet report before concluding.

## Evidence log

### 2026-08-14 initial checkpoint

The first repository read changes the framing from "replace Optique with CAC" to "compare CAC
with the current owned parser
set."
The package manifest shows Optique has already been removed.
The key practicality threshold is therefore net simplification with strict behavioral parity,
not removal of an installed framework.
This is a starting hypothesis only.

### 2026-08-14 incumbent audit checkpoint

Repository history shows that cli-git removed Optique on 2026-07-15:

- `1e53f52e02989b152576c0683f6f228b24c40140` added the owned Git argv region parser;
- `4879a44e6a2460ab3c2531744e24a1f2aef27aeb` moved commit-region parsing to it;
- `be3c522375d9c2652f44921c7396233c93c2397d` moved management parsing and removed the dependency.

The change was not only dependency cleanup.
The commit history records a thousand-path commit taking 4.24 seconds through Optique,
with 2.56 seconds in discarded option-mismatch suggestion work,
then 0.89 seconds through the owned parser.
Treat these values as historical motivation,
not a current CAC comparison.
The maintained benchmark now measures thirty recorded runs after six warm-ups in a bounded two-CPU,
2 GiB container.
Its 2026-07-16 wide-commit evidence had a 456.3-millisecond maximum wrapper-added result for 256 changed paths;
the current contract allows 925 milliseconds.

The source audit also shows why a framework-wide migration may be a category mismatch.
The closed management grammar is conventional CLI parsing.
The forwarded Git surfaces are partial semantic inspections of an external grammar:
they intentionally accept unknown future options,
model only guard-relevant arity,
and leave raw argv for real Git.
CAC must be evaluated separately for these two roles.

Current verification layers include direct argv parser cases,
commit parser cases,
policy and transform tests that consume parser facts,
ninety-six wrapper tests in `src/bin.unit.test.ts`,
trust management subprocess tests,
packed JSONL boundary fixtures,
and a packed lifecycle benchmark.
Migration completeness requires mapping every affected parser branch
to these tests rather than treating management examples alone
as parity.

### 2026-08-14 CAC-only scope checkpoint

The user excluded every other external CLI parser technology from this session.
The active compatibility fingerprint is
`a48b54e274b6d6bda3057027fbcc2ced424ea8c0a00b03a2ce868425a534f986`.
The original unqualified vet report is superseded because its fingerprint included broad alternative discovery.
CAC integration shapes may still be compared with each other,
and current owned behavior remains the required parity baseline.

### 2026-08-14 CAC discovery checkpoint

CAC-only discovery is complete and frozen.
The pinned candidate is `cac@7.0.0`,
release tag `v7.0.0`,
commit `77f602fcb2d1e75d24f5ecd94d5bf667acaa857a`.
The source clone is `~/temp/agent/cac-2026-08-14`.

Screening passed license,
Node-version,
inspectability,
and initial package-provenance gates:

- source and tarball carry the MIT license;
- the package requires Node `>=20.19.0`,
  which covers cli-git's supported range;
- the published package has no declared runtime dependency or lifecycle script;
- its build inlines MIT-licensed `mri@1.2.0`,
  so MRI remains part of the audit surface even though npm reports zero dependencies;
- npm SLSA provenance binds the measured tarball digest to the signed release tag and source commit;
- exact GitHub Advisory Database queries found no advisory affecting npm `cac` or `mri`.

The published tarball contains five files and 41,198 unpacked bytes.
Its runtime is one 19,503-byte ESM file plus a 4,871-byte declaration file.
The CAC source has 960 physical TypeScript lines across six files;
MRI adds 119 source lines.
CAC's test source has fifteen syntactic registrations that produce seventeen executed cases,
no fuzz or mutation harness,
and current CI covers Ubuntu and Windows on Node 22,
24,
and 25.
It does not run macOS,
so macOS remains a consumer-validation requirement.

Source inspection identified capability risks to probe rather than assume:

- MRI coerces option values that look numeric,
  and current `cacjs/cac#165` reports leading-character loss;
- a dash-led token after a value option is treated as another option rather than that option's value;
- the lone `-` spelling needs direct verification because MRI's scanner does not visibly classify it as positional;
- `allowUnknownOptions` suppresses validation but does not preserve an unknown-token fact list;
- help writes through `console.info` rather than an injected sink;
- parsed options and action callbacks use `any` in the shipped declarations;
- `parse` mutates a stateful `CAC` instance and dispatches command events even with `run: false`.

These findings did not alone settle practicality.
The subsequent bounded probes now separate rejected broad migration from the surviving management-only shape.

### 2026-08-14 runtime and parity checkpoint

The published `cac@7.0.0` artifact ran in a network-disabled,
read-only Node 24.18.0 container with 2 GiB memory and 2 CPUs.
Positive controls proved ordinary text options,
post-`--` tokens,
options after positionals,
one-level command routing,
and usage validation work.

CAC failed hard requirements for replacing `src/parser/argv.ts` or the complete parser set:

- numeric-looking option values lose exact spelling;
- `type: [String]` runs after loss and returns an array;
- dash-led declared values are treated as new options;
- lone `-` disappears;
- kebab-case booleans can consume the following token;
- unknown option tokens are normalized rather than retained as a token sequence.

Do not pursue CAC as the Git-region parser or as a complete parser replacement.
Those integration shapes have exited the audit on hard-gate failure.

Management-only use reaches Linux/Node 26.7.0 parity and Node 24.18.0 built-consumer validation,
but ultimately fails the complete package runtime gate.
A direct 41-case mapping matched the incumbent in 37 cases.
The mismatches were policy IDs `001`,
`+2`,
`-x`,
and `--all` when consumed as a policy value.
A second prototype retained an owned policy scanner,
replaced exact values with safe placeholders before CAC,
and restored them after parsing.
That shape matched all 44 expanded catalog cases with empty stderr.

The result is the central tradeoff:
CAC can own the closed command router only if cli-git keeps custom lexical parsing for exact policy values,
keeps all Git-region parsers,
keeps hand-authored exact help,
and adds runtime validation around CAC's `any` result types.
The untyped disposable adapter plus scanner occupies physical lines 9 through 94 of its harness.
Current replaceable management specs and parsing occupy `management-parser.ts` lines 129 through 265,
but production TSDoc,
logging,
validation,
and lint requirements make this an upper-bound comparison rather than net line-count proof.

### 2026-08-14 source reproduction checkpoint

The pinned upstream source passed its complete Linux/Node 24 package path in a bounded offline container:
build,
typecheck,
lint,
and all seventeen Vitest cases.
The test phase reported seventeen passes and no failures.
The rebuilt 19,503-byte ESM and 4,871-byte declaration files matched the published npm artifact byte-for-byte.
Their SHA-256 hashes are
`01af40eab1e1de3d543e740fa73c0095ce188c752300dd25d90ef0cd32a5d7c9` and
`25265ad103164bfc85707531963d66c59b84a230e3551cf5bc336166a74ae93c`.

The dependency fetch used the frozen lockfile with all lifecycle scripts disabled.
The 455,847,337-byte tree remained below its declared 1.5 GiB ceiling.
The numeric-preservation patch then failed its four exact-value assertions before the change and passed after it.
The durable diagnosis,
patch,
workaround catalog,
and upstream comment draft are in
`doc/troubleshooting/cac-option-value-coercion.md`.

### 2026-08-14 production integration checkpoint

A typed disposable package integration matches the incumbent on 52 management cases under Linux/Node 26.7.0.
The expanded set includes exact numeric and dash-led policies,
repeated booleans,
lone dashes,
placeholder collisions,
unknown joined options,
and exact post-`--` tokens.
The first typed adapter refused repeated `--yes` and `--all` because CAC returns repeated booleans as arrays;
explicit runtime validation fixed that mismatch.

The production shape is not a simplification.
Its sibling CAC adapter occupies 570 physical lines and 291 measured noncomment code lines.
Together with the delegated parser,
it totals 720 physical and 360 measured code lines versus the incumbent's 265 physical and 159 code lines.
The measured increases are 455 physical and 201 code lines.
The custom exact-value scanner remains,
and now also handles collision-free placeholders,
lone-dash restoration,
repeated boolean arrays,
and CAC's `any`-typed option object.

Type checking,
build,
packaging,
the built trust consumer,
side-effect-free package import,
authored help,
and invalid-usage exit `2` all passed.
The final MJS grew from 1,029,609 to 1,037,963 bytes,
a measured 8,354-byte or 0.8114 percent increase.
The packed manifest declares exact dependency `cac: 7.0.0` and retains one public MJS application artifact.

Candidate and unchanged-baseline oxlint both report zero warnings and the same 96 pre-existing test-import errors.
The candidate file itself has no remaining finding.
The full unit run passed the cli-git entry-point and other reported groups,
then one Git fixture collided on `.git/config` locking;
the exact failing file passed on isolated retry.

Upstream release-commit Action run `22491031174` passed CAC's Ubuntu and Windows tests on Node 22,
24,
and 25 plus Deno.
The overall workflow was red only because its lint job failed.
The expired lint log returns HTTP 410,
so its exact diagnostic is unavailable.
Combined cli-git adapter behavior remains directly tested only on Linux.

The candidate passed all maintained lifecycle budgets.
`wide-commit` measured 300.6160 ms median,
311.0310 ms p95,
and 316.5200 ms maximum against the 925 ms ceiling across thirty measured runs.
No-config startup measured 87.5854 ms median and 94.3790 ms maximum against 275 ms.
The Linux/Node 24.18.0 absolute latency gate passes.
Two runs per shape found a 292.2756 to 292.6579 ms baseline `wide-commit` median band and a
299.5193 to 300.6160 ms candidate band.
The measured candidate delta is 6.8614 to 8.3404 ms,
still far below the 925 ms contract.
Other scenario bands overlap or move in opposite directions,
so the audit does not infer one universal startup cost.

### 2026-08-14 terminal runtime checkpoint

The exact Node 22.18.0 lower-bound probe ended the surviving management-only shape.
Official image digest
`sha256:0d130e2ee18e88e1561375276daced6bff032539200173f2daf48c2e33f38ff5`
rejected both unchanged and CAC-integrated built artifacts on retained `await using` syntax.
A source-level candidate probe separately failed because shared logger code calls absent `Error.isError`.
Node 24 release notes identify both language and API surfaces as Node 24 additions.

This defect predates CAC,
but the frozen hard gate applies to the resulting package range
`^22.18.0 || >=24.11.0`.
The management-only shape therefore is not a validated finalist.
The durable diagnosis and remediation paths are in
`doc/troubleshooting/cli-git-node-22-runtime-contract.md`.

No CAC shape is scored:
Git-region and complete replacement fail argv fidelity,
while management-only fails the final package runtime gate.
The recommendation is to retain the repository-owned parsers.
Even after a future Node-floor correction,
CAC keeps the custom scanner and adds code,
so it has no practical simplification benefit.

## Open risks

- A future Node-floor correction would remove the package runtime blocker,
  but it would not remove CAC's exact-value scanner or the measured code increase.
- Existing historical comments can obscure which parser behavior is current.
  Source and tests,
  not comments alone,
  are authoritative.
- The target package is a shadowing `git` executable.
  A parser that exits,
  writes help,
  or normalizes argv unexpectedly can alter safety and automation behavior even when ordinary examples pass.

## Completion state

Final audit and troubleshooting Markdown validation passed.
The audit lock is released.
Both disposable repository worktrees were removed after preserving the production integration patch and hashed evidence
under `~/temp/agent/cac-artifact-2026-08-14/`.
No audit work remains.
