# Handover: forbidden-strings engine migration orchestration

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`.

Updated:
 2026-07-17,
 post-compact session resumed;
 subagent freeze lifted by the user.

## URGENT context for the post-compact session

- Subagent freeze LIFTED:
  the user compacted and said continue;
  the port-rerun agent dispatch is authorized.
- #376 state RESOLVED into:
  `e8763d56c` IS pushed (a later auto-push carried it;
  the gate blocks at commit time,
   not push time).
  Its `Closes #376` fired;
  the orchestrator REOPENED #376 with a comment listing the open items
  (518 reshape,
   three-casing expansion,
   skip-list gate fix).
  The user had AUTHORIZED the agent's raw-git commit bypass
  (not real secrets,
   preserve trace,
   return ASAP).
- Skip-list gate fix DONE (`37286df18` plus a trust refresh;
   pushed):
  both ported file paths added to `SCANNER_SELF_MATCH_PATHS`
  (canonical source `package/git-policy/forbidden-strings/src/scan-candidates.ts`;
  the CLI copy is file-enforcer-generated).
  CORRECTION (caught by the port-rerun agent):
  the first throwaway-worktree verification was weaker than recorded;
  the append file has no self-matching line,
  so its gate pass was vacuous
  (the canary only proved scanning was live,
   not that skipping worked),
  and the dist rebuild was never the operative step:
  the gate executes policy code from the frozen trusted snapshot under
  `~/.local/state/cli-git/trust/v1/records/...`,
  which predated the fix.
  `git cli-git trust --yes` in the main worktree activated the fix;
  afterwards `git cli-git check` on the modified
  `builtin-rules.ported.txt` went from flagging line 111 (rule 193)
  to clean while a canary still flagged,
  a non-vacuous verification.
  Durable write-up:
  `doc/troubleshooting/cli-git-trusted-snapshot-stale-policy-code.md`.
- NEW wrapper feature `--no-worktree-copy` (`99806c0c6`,
   pushed;
  user authorized mid-task):
  wrapper-only flag skipping the ignored-state worktree copy
  (the copy feature landed concurrently as the user's `7b63241b3`)
  for one invocation;
  parameterized the escape-hatch stripper for reuse;
  unit tests cover flag-position strip and value-position preservation;
  README + SPEC updated.
  Operational findings for opted-out throwaway worktrees:
  the commit gate there needs the scanner binary copied to
  `package/cli/forbidden-strings/target/release/`,
  a `node_modules` symlink to the main worktree,
  and `git cli-git trust --yes` (trust records are per-worktree).
- #376 port results:
  261 regex rules ported,
   260 compile;
  182 semantic changes:
  172 inline `(?i)` strips,
  16 quantifier bounds,
  1 reshape (rule 172).
  Review doc:
   `doc/planning/forbidden-strings-rule-port-review.md`.
- DECIDED by user 2026-07-16 (corrected phrasing,
   supersedes an earlier
  blanket-strip reading):
  inline `(?i)` spans are NOT simply stripped.
  Keyword literals under `(?i)` scope expand to a three-casing
  non-capturing alternation:
   lowercase,
   Capitalized,
   UPPERCASE
  (e.g. `(?:adobe|Adobe|ADOBE)`),
  because people write those three shapes and never mixed-case `AdOBe_`.
  Character classes under `(?i)` scope widen to both cases
  (`[a-z]` to `[a-zA-Z]`).
  Multi-run tokens capitalize per alphabetic run
  (`api_key` yields `api_key|Api_Key|API_KEY`).
  Applies to the 172 affected rules;
  implement inside `dialectport.rs`,
   rerun,
   update the review doc
  (these rules reclassify from semantically-changed to
  approximately-preserving).
- STANDING PREFERENCE (user,
   2026-07-16):
  lossiness in the over-matching direction is wanted,
   not tolerated;
  a false positive on a base64 blob that happens to embed a secret shape
  is acceptable ("I'd deserve it").
  This ratifies aggressive context simplification generally and weighs
  toward strip-after-@ for the mongodb rule (pi verdict pending).
  The planning doc's earlier "case-insensitivity is a non-issue" claim was
  WRONG:
   it measured only trailing `/i` flags and missed pervasive inline
  `(?i)` groups;
   the plan doc needs this correction.
  Recommendation:
   ratify the strip (issue instruction said follow
  `normalize.rs` precedent,
   which strips case flags;
   user hinted the answer
  is in what is already written),
   gate a possible class-expansion
  follow-up on #387's differential results.
- DECIDED (pi advisor B > D > A > C;
   aligns with user lossiness
  preference):
   rule 518 becomes the credential-bearing core only,
  `\bmongodb(?:\+srv)?://[!-9;-~]{3,50}:[!-?A-~]{3,88}@`.
  Rationale to quote in the review doc:
  preserve the original credential payload but omit non-secret URI suffix
  validation,
   avoiding determinization blow-up and covering templated and
  partially constructed connection strings
  (delimiter-excluding classes make the phases deterministic;
  interpolated hosts like `${MONGO_HOST}` now covered).
  Fallback if noisy:
   append one plausible host-introducer byte
  `(?:[A-Za-z0-9]|\[|%)`.
  Full pi answer preserved durably at
  `doc/planning/forbidden-strings-rule-518-pi-advice.md`.
- Port rerun DONE,
   #376 CLOSED (legitimately;
   all criteria met):
  opus agent delivered `366b3cafa`
  (new `caseexpand.rs` three-casing module wired into `dialectport.rs`,
  rule 518 credential-core special case)
  and `3333f3e63` (review doc restructured;
  172 case rules reclassified approximately-preserving;
  518 rationale quoted;
   verification 261/261).
  Orchestrator landed the regenerated data file as `57cd6b3fd`
  (169 builtin rules changed bytes:
   168 case rules three-cased,
  4 case rules byte-identical for lack of letters,
   plus 518;
  append file unchanged)
  with `Closes #376`,
   through the gate,
   NO bypass.
  The agent correctly refused to bypass and correctly stopped at the wall;
  the wall was the orchestrator's stale-trust gap,
   not agent error.
- #380 DONE and CLOSED (sonnet;
   spot-checked):
  `e24e0ac72` (pre-existing bench-crate implicit_return debt,
  isolated and fixed separately),
  `d949cb8c4` (bench with per-line oracle agreement check),
  `90683cabc` (README numbers).
  Measured (Ryzen 7 8700F,
   single-threaded,
   1.16M lines,
   261 rules):
  per-line `matches()` loop 8.25M lines/s,
  concat hook 5.86M lines/s,
  `line_matches` 7.59M lines/s;
  so `line_matches` is 0.92x the per-line loop and 1.29x the concat hook.
  Reading for #381:
   the seedless and line-start groups still resolve
  per-line,
   which is exactly the headroom #381 would target;
  the absolute scanner workload (a few staged files per commit) makes
  either path sub-millisecond,
   so #381 remains a judgment call for the
  user at its queue position.
  Note for #384:
   the settled decision to use the batch API stands;
  the delta versus a hand-rolled per-line loop is 8% on a corpus far
  larger than any commit delta.
- #383 DONE and CLOSED (opus;
   spot-checked):
  `57a811aeb` (engine dependency),
  `c35ce7c19` (the `src/rule/frx/` module:
   `compile_from_text`,
  `load_precompiled`,
   redacted `LoadError`;
   four test files;
  208 tests pass).
  #217 correctly left OPEN with a comment:
  the leaking sites live in the still-active old load path,
  so #217 closes with #384 when the binary stops reaching them.
  Findings recorded by the agent:
  the scanner crate's `lint:clippy` was already red before its work
  (117 pre-existing implicit_return findings in old-pipeline files;
  #385 owns the gate during teardown),
  and `builtin_ported_all_compile` takes ~21s
  (continuously verifies the ported baseline compiles;
  candidate for `#[ignore]` if suite runtime matters).
- #384 DONE and CLOSED,
   #217 CLOSED with it (opus;
   spot-checked):
  `df1e9e006` (build-time precompiled baseline:
  `build.rs` shares the stage-one parser by `#[path]` include,
  serializes via `to_bytes` into `OUT_DIR`,
  `lib.rs` embeds via `include_bytes!`,
  runtime always `load_precompiled`,
   never recompiles the baseline)
  and `0a15c5c5a` (line-based scan path on `line_matches`,
  columnless `PATH:LINE rule=N` on stderr,
  `catch_unwind` fail-closed kept,
   226/226 tests).
  Rule-id scheme changed:
   runtime rules `0..user_len`,
  builtin offset above;
   gate runs builtin alone so offset 0.
  Reachability check for #217 verified
  (12 disclosure sites only in the now-dead resharp chain).
  Agent stopped once mid-build awaiting its child;
  one SendMessage nudge resumed it cleanly.
- LIVE-GATE FINDING (orchestrator canary,
   post-#384):
  the gate's finding path is format-broken until #388 lands:
  a real finding now surfaces as `plugin-threw`
  "Malformed forbidden-strings scanner output" instead of a redacted
  finding.
  FAIL-CLOSED holds (finding-bearing commits still blocked),
  so no security hole,
   but #388 became urgent and was dispatched
  immediately.
- #388 DONE and CLOSED (opus;
   orchestrator canary re-verified):
  `b9b43ee60` (parser:
   `ScannerHit` drops columns,
  `lastIndexOf`-based split so colon-bearing paths stay safe,
  malformed lines fail closed redacted),
  `215957fba` (file-enforcer regen),
  `68cf30c98` (empty marker commit with `Closes #388` after
  trust-refresh and live verification;
  `--no-enforce-only` used correctly,
   the sanctioned empty-commit case).
  Agent followed the trusted-snapshot activation steps and verified
  non-vacuously;
  orchestrator canary confirms
  `Forbidden string matched at line 1 (rule 58).`
  Real-binary integration test committed
  (spawns the release scanner,
   runtime-built token,
   no leak).
  NOTE for #389's local-appendix port:
  rule indexes are environment-dependent
  (canary rule=58 under repo env versus rule=20 embedded-baseline-only),
  which means the gate scan DOES load a local rules file first and
  offsets the builtin baseline;
  the local file exists and is live in the gate path.
- #385 DONE and CLOSED (opus;
   spot-checked:
   0.2.0 in manifest,
  binary self-reports 0.2.0):
  `a85bf020f` (27-file teardown,
   deps dropped,
   unreferenced-ness
  verified per deletion,
   `catch_unwind` and release hardening kept),
  `063f61fd7` (clippy gate widened to `--all-targets` and green;
  33 surviving implicit_return fixes),
  `aea61186a` (README + CLI-help rewrite for the new dialect,
  0.2.0 bump unpublished,
   troubleshooting pointer notes).
  Its flag about the stale `algebra_tests.rs` entry in git-policy's
  skip list is parked on #390 as a comment
  (with the trusted-snapshot activation chain reminder).
- SEQUENCING CHANGE:
   #386 and #387 run SEQUENTIALLY,
   not parallel:
  both touch the scanner package (fuzz sidecar and `fuzz_api.rs` versus
  `PERF.md` and report),
   and #387's perf re-measure needs a quiet
  machine that a concurrent fuzz smoke pass would poison.
- #386 DONE and CLOSED (opus;
   spot-checked):
  `1c5f3f51d` (fuzzing-gated API surface),
  `e60a8cbfe` (three retargeted targets,
   two-form generator replacing
  the 1636-line resharp generator,
   3374 dead seeds pruned,
  40 fresh seeds,
   dictionary rewritten,
   `.gitattributes` binary marks),
  `aa4d3ac27` (docs plus lockfile,
   `Closes #386`).
  Smoke passes:
   24585/9790/5767 runs in 31s each,
   zero artifacts.
  PROCESS LESSON (now baked into prompts):
  a subagent's backgrounded processes are KILLED when it stops,
  so background-build-and-wait loops both stall the agent and lose the
  build;
   instruct agents to run long commands synchronously.
  This agent stalled twice on Monitor waits before a firm
  "synchronous only" steer finished it.
- #387 DONE and CLOSED (opus,
   44 tool calls;
   the cutover gate PASSED):
  `3200c64f0` (differential report),
   `ca420697e` (`PERF.md`).
  Zero lost findings on all corpora;
  gains all attributed (518 mongodb 4+2,
   172 curl 2);
  rich rule-exercising corpus byte-identical after normalization;
  numbering calibrated (old 1-based source line,
   new 0-based compiled
  index) and cross-checked;
  perf within every budget (cold 13.7ms,
   `--all` 131.9ms,
   0.78x old).
- DISCOVERED post-#387 (orchestrator measurements):
  forbidden-strings 0.2.0 ALREADY LIVE on `crates.io` AND as GitHub
  release `forbidden-strings-v0.2.0`
  (the `aea61186a` version bump auto-triggered the publish lanes;
  benign,
   #387 validated exactly that binary,
   but it voided the
  "unpublished until cutover" intent).
  Separately,
   CI workflow `forbidden-strings.yml` has been RED since
  2026-07-12:
   it still copies `forbidden-strings.local.example.txt`,
  deleted by the user's de-root commit `58995afff`
  (file-enforcer now composes appendixes into
  `.cache/forbidden-strings.rules.txt`,
   `FORBIDDEN_STRINGS_RULES` env,
  `builtinRules` policy option).
  Zero green runs in the last 40;
   predates the engine swap.
- MAINTAINER LEG OF #389 DONE by orchestrator (user-authorized):
  verified the gitignored local appendix loads clean under the 0.2.0
  strict loader (exit 0;
   no port needed,
   dialect-compatible),
  then refreshed the stale `FORBIDDEN_STRINGS_LIST` repo secret from it
  via a straight pipe (contents never displayed;
  `gh secret list` stamps 2026-07-17T11:40Z).
- #389 DONE and CLOSED (opus;
   the CUTOVER IS COMPLETE):
  `570c267aa` (ported files promoted to live names;
  append diff was exactly two no-op `/m` flags),
  `2d3d1d8b0` (skip-list pruned to three entries,
   activation chain run,
  non-vacuous gate canary),
  `3ce0cffd9` (CI materialize rewritten per de-root design),
  `a74cb9df7` (runbook,
   `Closes #389`).
  Verification PRs:
   #394 no-op GREEN,
   #395 API-committed red case RED
  with fully redacted output;
   both closed unmerged,
   branches deleted.
  0.2.0 publish criterion was already satisfied;
   nothing re-published.
- OPEN DECISIONS FOR THE USER after #389 (recorded 2026-07-17):
  1. Push-to-main full-tree CI scan is legitimately RED with 7 redacted
     findings:
      the 6 accepted over-matches documented by #387
     (two planning docs,
      one troubleshooting doc,
      one package README)
     plus ONE secret-ruleset match at
     `package-deprecated/audit/oph-common-look-and-feel/src/index.html`
     line 2657 (local rule compiled index 4;
     fires only with the sensitive ruleset;
     content deliberately not read by agents;
     maintainer must inspect).
  2. Same mechanism blocks committing any edit to
     `doc/planning/forbidden-strings-rule-port-review.md`
     (its rule 518 example strings self-match);
     the #389 agent could not land its pointer note there.
     Remediation options presented to the user,
     ranking B > A > D > C:
     B reshape doc examples to not match,
     A extend a skip/exclusion surface,
     D scanner allowlist feature (plan rejected),
     C accept red.
  3. #381 seedless-routing:
      CLOSED wontfix 2026-07-17 by user directive
     ("Close #381 based on #380"),
      with the #380-numbers rationale in the close comment.
- POST-CUTOVER FOLLOW-UPS (2026-07-17,
   after the migration chain closed):
  - #391 got a brainstorm comment (user asked for ideas):
    migration evidence (no escalations fired across the dispatches),
    the recurring background-wait stall as a stronger persistence
    candidate than escalation,
    options ranked orchestration-doc > wontfix > `AGENTS.md`.
    Still undecided by user.
  - The index.html finding (open decision 1) was diagnosed:
    it is a three-byte bare-literal local appendix rule (term redacted),
    coincidentally embedded in a ~49KB base64 asset on line 2657
    between word bytes.
    Not a real leak;
    a base64 substring collision.
    This finding is exactly what the word-boundary directive below fixes:
    once live,
     the three-byte literal gates to whole-token matches and
    stops firing inside the base64 blob.
  - USER DIRECTIVE 2026-07-17:
    "Bare literals under eight bytes should always require word
    boundaries around them."
    IMPLEMENTED (`296c5169c`,
     pushed,
     code-complete):
    `literal_pattern` in `src/rule/frx/escape.rs` wraps a sub-8-byte bare
    literal with `\b` at each end whose adjacent byte is an ASCII word
    byte (per-end conditional,
     so CJK and punctuation ends stay plain,
    since a `\b` beside a non-word byte asserts an ASCII edge CJK text
    never provides).
    Applied in `format.rs` at the literal call site;
    `escape_literal` kept a pure exact-match transformer (fuzz target
    intact).
    Tests updated (four cases encoded old substring semantics);
    example terms neutralized to `ABC`/`Ab Cd` so the test source does
    not self-match the deny-list.
    ACTIVATION DEFERRED:
     the local release binary and CI both still run
    0.2.0 (old escaping);
    making this live in CI needs a release
    (CI downloads the version-matched release binary),
    bundled with the rule-identity decision below and the block-form
    format (#396) so one release covers all pending scanner changes.
  - ADOPTED 2026-07-20,
     tail-format sectioned rule files (#396):
    after a full choosing-technology vet
    (`doc/audit/tech-forbidden-strings-rule-file-format-vet-2026-07-20.md`,
    finalists block form,
     tail-format,
     NestedText,
     TOML;
    maintainer chose parser auditability as the governing axis),
    the maintainer adopted the tail-format sectioned file:
    `==> name <==` headers (strict kebab name grammar) delimit rule
    sections whose bodies the always-verbose engine reads raw;
    every rule,
     bare literal included,
     is its own named section
    (the `.literals` list form was rejected by maintainer correction);
    near-header lines fail closed;
    format autodetection keeps legacy files working through the
    transition.
    Decision record `doc/decision/forbidden-strings-rule-file-format.md`;
    normative spec `doc/planning/forbidden-strings-tail-rule-format.md`;
    the block-form draft is superseded but keeps the verified engine
    facts.
    Section names give the rule-identity decision its carrier
    (baseline sections get betterleaks ids at conversion).
    CRITICAL sequencing unchanged:
     binary and release first,
     gate/CI
    move second,
     data-file conversion third.
    IMPLEMENTATION LANDED (`36e299efc`,
     pushed,
     #396 auto-closed):
    `sections.rs` parser + 4 redacted line-numbered LoadError variants +
    autodetection in `format.rs` (legacy byte-identical) + build.rs
    embed wiring;
     112/112 tests,
     all lints green,
     boundary-verified
    with the release binary (tail findings fire,
     near-header fails
    closed redacted,
     legacy unchanged).
    NEAR-HEADER HARDENING LANDED (`d15e60397`,
     done in-session,
     no
    subagent,
     per maintainer instruction):
     code now matches spec ruling
    `65dd44404`;
     any line whose trimmed form starts with `==>` that is
    not exactly a strict header is a redacted line-numbered
    `NearHeader` error (out-of-alphabet middles,
     indented headers,
    missing-space arrows alike;
     `#`-comment arrow mentions unaffected;
    `[=]` in a bare literal stays literal,
     new test).
     The grammar was
    also simplified on maintainer prompt (a `?`-chained `strict_name`
    helper,
     extracted `close_section`,
     functional detection and
    duplicate-name tracking).
     116/116 tests,
     all three lints green,
    boundary re-verified:
     `==> /etc/passwd <==` in a rules file fails
    closed printing only its line number,
     and a good tail file still
    yields findings.
  - RULE-IDENTITY DECIDED and NAME MIGRATION UNDERWAY (2026-07-20,
    maintainer:
     "migrating the rules so every rule can have a sanitized
    name that forbidden-string rule match outputs can refer to.
     But for
    *.local.*
     rules:
     Hide even the fact they're about China
    specifically."
     plus "Re-write the porter or recover it from git and
    still auto generate the rule and make the generate rules task refer
    to it.").
     Landed this session,
     all pushed:
    `d95a3a72a` porter chain (TS porter emits `==> id <==` headers,
     two
    named demo sections,
     `.cache` stage-one intermediate;
     dialectport +
    caseexpand recovered from `5a5b9e3b7^` as the standing stage two;
    generate:rules runs both;
     MONGODB_CORE concat!-split against rule
    518 self-match);
    `20db3b030` gate parser relays alphabet-validated rule tokens
    (names verbatim,
     numeric ids kept,
     `rule=0` no longer rejected;
    stale walker-exclusion test realigned with the #389 pruned skip
    list,
     red since `2d3d1d8b0`);
    `2e89b498f` docs/templates (identity decision recorded;
     seed and
    cache templates describe tail + opaque naming;
     scan:all repaired);
    `c8de3b8f1` scanner name support (compile_rules with name table,
    BUILTIN_NAMES build-time sidecar,
     cross-set collision fail-closed,
    named findings `rule=<name>` with numeric fallback;
     fuzz scan-format
    contract widened;
     dialectport split into src/dialectport/ modules
    for max-lines);
     122/122 tests,
     clippy -D warnings green,
     fuzzing
    check green,
     rust-linter green on scanner and bench;
    `79abdfe59` baseline converted to tail sections via the chain,
     all
    259 rule bodies verified byte-identical (compare script kept the
    #387 differential validity).
  - NAME MIGRATION COMPLETE (2026-07-20,
     same session):
    `2ff0ee2c5` bumped 0.3.0 (three locks regenerated;
     the fuzz lock
    also shed a stale orphaned subtree,
     zmij and serde leftovers);
    cargo-publish run 29781894639 succeeded,
     release
    `forbidden-strings-v0.3.0` live with 7 attested assets and the
    crates.io publish;
    git-policy dists rebuilt and `git cli-git trust --yes` refreshed
    BEFORE any appendix converted (parser accepts both token forms,
     so
    the refresh was format-independent);
    gate canary relayed "Forbidden string matched at line 1 (rule
    age-secret-key)" through `git cli-git check`,
     non-vacuous;
    `6382098ce` converted `forbidden-strings.append.txt` (sections
    ncd-reserved-shortcodes,
     ncd-reserved-shortcode-families);
    the gitignored local appendix converted to 36 opaque `local-NNN`
    sections (numbering append-only;
     names deliberately reveal nothing
    about a rule's subject,
     per maintainer instruction);
    `.cache/forbidden-strings.rules.txt` regenerated (38 sections);
    `FORBIDDEN_STRINGS_LIST` refreshed from the converted local file
    via straight pipe (gh secret list stamps 2026-07-20T22:10:50Z);
    composed-ruleset scan verified all three sources render names
    (`rule=local-007`,
     `rule=ncd-reserved-shortcodes`,
    `rule=age-secret-key`),
     and the word-boundary gate is live
    (`xxCCPxx` glued does not fire;
     the standalone token fires as
    `rule=local-005` and the gate relays it).
    CI run 29783022880 on `6382098ce` downloaded and
    attestation-verified 0.3.0 and is red with EXACTLY the six known
    accepted over-matches,
     now named (`rule=curl-auth-user` twice,
    `rule=mongodb-connection-string` four times,
     in the two planning
    docs,
     one troubleshooting doc,
     one package README);
     the
    `index.html` base64 collision finding is GONE (boundary fix
    confirmed in CI).
  - DOC EXAMPLES RESHAPED (maintainer directive "Reshape the doc
    examples",
     closing #389's open decision 1 via its ranked option B):
    `b3808ad16` drops the two mongodb examples to a two-byte password
    below the rule's three-byte minimum (annotated so the illustration
    stays honest),
     hoists the podman example's uid:gid into a variable,
    and quotes the android-exempt-unused task path whose `-unused:run`
    tail matched the curl credential rule.
    At that cutover,
     the rule had no leading token boundary and any
    `...-u<word>:<word>` doc line could match it.
    The later decision in
    `doc/decision/forbidden-strings-curl-auth-user-boundary.md`
    partially supersedes that behavior:
     line start or horizontal whitespace must now precede the option.
    The commit also went through the gate cleanly on
    `doc/planning/forbidden-strings-rule-port-review.md`,
     ending that
    file's commit block.
     Side correction:
     the MONGODB_CORE concat!
    split was reverted;
     probing showed the pattern text cannot
    self-match (the rule wants `://` right after the scheme,
     the text
    interposes `(?:\+srv)?`).
    CI run 29784124951 on `b3808ad16` concluded SUCCESS:
     the only
    green push-event run among the last 40 (workflow red since
    2026-07-12),
     and the local full-tree scan exits 0.
    Unratified proposal from an earlier session:
     AGENTS.md rule `RFY`
    ("'X must change' ratifies the problem,
     not your design;
     present
    options per OPT and get the concrete design ratified before
    implementing or dispatching") awaits maintainer wording approval.
  - OPEN DECISION,
     rule-identity UX (user asked "how does one know which
    rule is the trigger?"):
    the finding `PATH:LINE rule=N` is opaque and the index DRIFTS
    (same builtin rule is `rule=20` alone,
     `rule=58` with the ~38 local
    rules loaded,
     because local rules take `0..k` and builtin is offset
    above).
    KEY CONSTRAINT found:
     the binary embeds only the compiled baseline
    automaton and the port stripped all betterleaks descriptions,
    so a baseline match cannot be mapped to a name/text at runtime;
    `explain`/names/hashes for the baseline all need a build-time
    identity sidecar from `build.rs` (which already parses the source).
    Local rules are resolvable at runtime (read at scan time).
    Orchestrator recommendation put to user:
    namespace findings `rule=builtin:N`/`rule=local:N` (kills the drift,
    one more #388-style parser update),
    embed baseline NAMES at build time (betterleaks porter has them),
    keep `local:N` a bare per-file index (zero disclosure),
    `explain` thin over both.
    User floated a rule HASH;
    assessed:
     stable across baseline versions but a membership oracle for
    low-entropy sensitive local rules (needs keyed/HMAC),
    and names dominate it for the public baseline.
    Forks awaiting user:
    namespaced ids yes/no;
    baseline names vs hashes vs full source;
    local index vs keyed hash.
    FULL design analysis preserved durably at
    `doc/planning/forbidden-strings-rule-identity-ux.md`
    (problem,
     redaction scope,
     the runtime-identity constraint,
    the hash membership-oracle assessment,
     four options,
     recommendation);
    read it to resume this decision,
     this handover only summarizes it.
- #390 DONE and CLOSED (sonnet hygiene pass):
  #158,
   #240,
   and #226 closed as mooted by the engine swap
  (issue comments only,
   no commits:
  resharp,
   the `regex` crate,
   and `aho-corasick` are gone from the scanner
  as of 0.2.0);
  #224 commented,
   not closed:
  items 14,
   16,
   and 17 are now fixed structurally by the strict loader
  (unsupported-flag hard error,
  empty-matchable-pattern rejection,
  BOM stripping),
  items 15 and 18 remain open.
  `64e4a7c06` reshapes the 1.0 checklist's dependency-exposure item
  (resharp exposure becomes own-engine maturity) and annotates the em-dash
  planning doc with the new dialect's implications
  (bounded complements no longer need the literal-space workaround,
  but unbounded `.*` must become bounded repetition).
  `5a5b9e3b7` removes `dialectport.rs` and `caseexpand.rs` from the bench
  sidecar and updates the runbook's pending-#390 phrasing to past tense,
  verified via `cargo build --release`,
  `cargo check`,
  `cargo clippy --release -D warnings`,
  and the max-lines/require-rustdoc linter,
  all green through `mise run`.
  The migration's issue chain #375-#390 is now COMPLETE.
- RESOLVED plan open question:
  startup compilation is NOT viable
  (worst rule 123s pre-strip;
   49 rules over 1s even after fixes);
  the loader (#383) MUST embed a precompiled serialized `RegexSet`
  (`to_bytes`/`from_bytes`);
  update the planning doc's open-questions section.
- The working tree is clean (agent removed its `probe.rs` debris before
  committing).

Fresh milestones (details in State of play):

- #392 done in two commits (`25fd8b9c6` sweep plus gate widening,
  `260823a4a` unwrap-to-expect);
  the engine's all-targets clippy gate is GREEN again.
- #382 done:
  `forbidden-regex` 0.1.0 is LIVE on `crates.io`
  (manual bootstrap publish from the authorized session),
  the maintainer configured trusted publishing,
  and commit `4b502af20` adds the fr- workflow lane.
- Remaining in flight:
  #376 only.
  Grace timer FIRED;
  orchestrator stop procedure was aborted because the user directly told
  the agent to clean up and commit ASAP;
  it reports back imminently with a design decision to resolve,
  which the user believes is answerable from what is already written
  (resolve by citation,
   do not reopen).
  Engine crate and workflow are free;
  bench sidecar still occupied by #376.
Assume nothing else survived compaction;
 this file plus the planning doc are the resume points.

## Canonical references

- Decisions and rationale:
  `doc/planning/forbidden-strings-engine-migration.md`
  (all grilled decisions,
   adopted defaults,
   measured facts,
   issue map).
- Port review (lands with #376):
  `doc/planning/forbidden-strings-rule-port-review.md`.
- Issues #375 through #390 are the migration;
  #391 and #392 are session spin-offs.

## User-issued orchestration policies (verbatim intent)

- Subagents get tiny,
   well-defined tasks;
  the main session stays orchestrator and never implements large slices itself.
- Model class per task difficulty:
  sonnet for mechanical sweeps,
  opus for bounded judgment work,
  fable for genuinely hard design.
- Escalation policy:
  if a class's output is not up to par,
  bump the class for future similar tasks.
  Persistence of this policy is tracked in #391 (needs-triage);
  do not add it to `AGENTS.md` (user vetoed,
   too specific).
- The user is authenticated on `crates.io` and authorized publishing
  `forbidden-regex` from this machine;
  the orchestrator (not a subagent) runs the actual `cargo publish`.
  Trusted publishing config comes after the first version exists.
- Handover duty:
  update this file at every milestone (agent completion,
   publish,
   escalation).
- pi advisor policy (user,
   2026-07-16):
  call `pi --model openai-codex/gpt-5.6-sol --print --no-tools --no-skills
  --no-themes --thinking xhigh` freely as an explicit-context advisor tool
  (background,
   several minutes,
   one-flush output);
  the native advisor tool is disabled as too slow.
- Bench-porter clarification (answers a user question):
  the bench crate's port is deliberately lossy
  (regex lines only,
   compile-filter drops failures,
   context stripped),
  so #376 rightly built a faithful porter;
  but wholesale context-stripping remains a live v2-port option
  if pi and the #387 differential favor it.

## Subagent supervision (no timeouts exist)

- The Agent tool has NO timeout parameter (verified against the tool schema
  2026-07-16);
  the #376 and #377 agents were launched unbounded.
  The user is aware and chose not to kill or relaunch them.
- Available levers:
  completion notifications fire automatically when an agent stops;
  `TaskStop` terminates a running agent manually;
  `TaskList`/`TaskOutput` inspect status.
- Prompt-embedded stop conditions are the only per-task budget mechanism.
  Prompts so far include scoped ones
  (stop-and-report on max-lines,
   report-not-fix on new lint families)
  but no overall effort budget.
- Policy for every future launch:
  include a soft budget line in the prompt,
  for example "if this exceeds roughly five hundred tool calls or you hit a
  wall,
   stop and report state instead of pushing on".
  The user set the calibration:
  forty is extremely low;
  five hundred is about right.
- If an agent looks stalled (no completion notification for an unreasonable
  span),
   inspect via `TaskList` and stop it with `TaskStop` rather than
  launching a duplicate into the same crate.

## Hard sequencing constraints

- Never two agents concurrently in the same crate
  (auto-commit on main plus auto-push makes same-crate churn collide).
- Agents commit with explicit scoped pathspecs
  (repo git guards reject bulk staging)
  and put `Closes #N` in the final commit body once acceptance criteria verified;
  auto-push closes the issue.
- On push race:
  `git pull --rebase`,
   push again.

## State of play

Done:

- #375 engine clippy sweep plus metadata.
  Commit `4106b66a0`,
   pushed.
  All 367 lib errors fixed mechanically;
  221 tests pass;
  `cargo package` verified.
  Scope note:
  roughly 80 more implicit_return hits in test modules
  (lib-only mise clippy task misses them);
  now covered by #392.
- #377 batch API contract.
  Commit `b69611f78`,
   pushed,
   spot-checked.
  API:
   `RegexSet::line_matches(buf, starts) -> Vec<(usize, usize)>`
  in `src/regex/batch.rs`;
  naive per-line `matches()` loop;
  output ordering line-ascending,
   `matches()` order within a line;
  preconditions documented not validated;
  5 new tests (226 total).
  Deviation note:
  an AWS-shaped test literal tripped the forbidden-strings commit gate;
  resolved with the byte-escape convention integration tests already use
  (`\x50` final byte);
  no git-policy files touched.
- #378 single-sweep fast path.
  Commit `4575442f3`,
   pushed.
  `line_matches` now calls `sweep_candidates(buf, starts)` once on the
  caller's buffer (zero copy);
  private `resolve_matches(line, has_seed)` gates only the seeded group;
  line-start and seedless groups untouched (deferred to #381);
  ordering contract preserved;
  new test proves a seed spanning a line boundary does not match;
  227 tests pass,
   all gates clean.
- #379 differential fuzz target.
  Commit `72e830f3d`,
   pushed.
  Target `fuzz_line_matches` plus `RulesetAndBuffer` generator
  (mixed `\n`/`\r\n`,
   forced empty lines,
   optional unterminated final line,
  `starts` built in lockstep);
  independent naive recomputation as the oracle;
  33k bounded runs clean,
   no artifacts;
  no engine-crate files touched.

Running (completion notifications arrive automatically):

- #376 rule-file port,
   opus.
   RETURNED;
   see URGENT section above.
  Bench sidecar plus data files plus repo-root append file plus review doc.
  Settled semantics in the issue:
  512 quantifier caps,
  `/m` drops,
  rule 172 curl-anchor drop,
  line-number alignment preserved,
  strict compile,
   zero drops,
  gitignored local files untouchable.
  CURRENT STATUS:
  scope drift observed by the user (extra probe bin beside the port bin),
  grace timer fired,
  user directly told the agent to clean up and commit ASAP;
  its report arrives imminently and carries a design decision to resolve.
  User guidance:
   the decision is likely answerable from what is already
  written (planning doc,
   engine README,
   port review conventions);
  resolve by citation,
   do not reopen settled decisions.
  Lesson recorded on #391:
  soft budgets and precise prompts are the only bounding mechanism.

Also done (session spin-offs):

- #392 in two commits:
  `25fd8b9c6` (60 test-module implicit_return fixes plus clippy gate widened
  to `--all-targets`;
  agent correctly stopped on the foreign `disallowed_methods` family)
  and `260823a4a` (60 test `Result::unwrap()` to `.expect()` conversions;
  gate green).
  Triage decision came from root `clippy.toml` reason text plus
  sibling-crate precedent (expect,
   never suppress).
  Scope note posted to #385:
  widen the scanner crate's lib-only clippy task during the teardown.
- #382:
  0.1.0 live on `crates.io`,
   trusted publishing configured by the maintainer,
  workflow lane in `4b502af20`.

Filed,
 not started:

- #391 escalation-policy persistence decision (needs-triage,
   human).

## Planned launch order

1. On #377 completion:
   launch #378 (fast path,
    opus,
    engine crate)
   and #379 (differential fuzz target,
    sonnet,
    fuzz sidecar) in parallel.
2. On #378 completion:
   launch #392 (sonnet,
    engine crate;
   its widened gate then also covers test code added by #377/#378).
3. After #378 and #392:
   orchestrator publishes `forbidden-regex` 0.1.0 and extends
   `cargo-publish.yml` (#382).
   Publish is authorized;
   run it from this session.
4. On #376 plus #377 completion:
   launch #383 (scanner rule-compiler module,
    opus).
   Then #384 (scan path),
    then #385 (teardown),
    sequential,
    same crate.
5. After #385:
   #386 (fuzz retarget) and #387 (differential validation plus perf)
   and #388 (git-policy parser,
    after #384 in fact) as crates free up.
6. #389 cutover reclassified ready-for-agent (user,
    2026-07-17):
   full AFK plan in the issue comment
   (API-side commit for the red-case PR,
   0.2.0 via the existing cargo-publish lane,
   runbook still authored under `doc/runbook/`).
   RESOLVED by user 2026-07-17:
   the current gh secret is STALE;
   the gitignored local rules file is authoritative,
   so the cutover overwrites the secret from the ported local file
   without preserving the old value.
   AUTHORIZATION (user,
    2026-07-17,
    supersedes the earlier no-read rule):
   the agent may freely read and manipulate the gitignored local
   ruleset files;
   the constraint is human exposure,
    not agent context.
   Contents must still never reach any human other than the maintainer:
   no tracked files,
    no commit messages,
    no issue text,
    no pushed logs,
   no CI output (scanner redaction invariant unchanged).
7. #390 hygiene last.
8. #380 (bench numbers) after #378;
   #381 (seedless routing) is bench-gated by #380 and may close wontfix.

## Model-class ledger (for the escalation policy)

- sonnet:
  #375 up to par (clean sweep,
   good scope note;
   35 tool calls).
  #379 up to par (fuzz target,
   clean scope discipline;
   34 tool calls).
  #380 up to par (104 tool calls;
   oracle check before timing,
  correctly isolated and separately committed pre-existing clippy debt,
  honored the dialectport/caseexpand exclusion).
- opus:
  #377 up to par (contract exactly as specced,
   spot-checked;
   53 tool calls).
  #378 up to par (correctness argument articulated,
   boundary test added;
  30 tool calls).
  #376 initial port:
   up to par on output,
   scope drift on process
  (extra probe bin;
   user-managed return).
  #376 rerun:
   up to par and then some
  (80 tool calls;
   caught the orchestrator's vacuous verification,
  diagnosed the skip-list gap correctly to within one layer,
  honored the no-bypass and no-git-policy constraints,
  stopped cleanly at the wall with an actionable handoff).
  #383 up to par (77 tool calls;
   leak-safe error type by construction,
  tracing-subscriber redaction test beyond the letter of the spec,
  correct #217 partial-coverage judgment,
   disciplined scope on the
  pre-existing clippy debt).
  #384 up to par (129 calls;
   one mid-build stall,
   nudged once).
  #385 up to par (103 calls;
   per-file unreferenced-ness verification).
  #386 up to par on output,
   stalled twice on background-wait
  (the process lesson above;
   work itself excellent).
  #387 exemplary (44 calls;
   calibrated numbering before trusting it,
  pin proven positively,
   zero unexplained deltas).
  #389 up to par and then some (92 calls;
  isolated the `index.html` finding to the secret ruleset without
  reading content,
   refused unilateral skip-list expansion,
  flagged the review-doc friction instead of working around it).
- Standing conclusion:
   no escalations needed anywhere in the
  migration;
   sonnet for mechanical,
   opus for bounded judgment held.
- No escalations recorded yet.

## Sequencing refinement discovered en route

#380 (bench coverage) is blocked in practice by #376,
 not only by #378:
the bench sidecar crate is where #376's port bin lives,
so #380 launches only after #376 completes.

## Verification duties that stay with the orchestrator

- Spot-check each agent's landed commit against the issue's acceptance criteria;
  reopen the auto-closed issue if deficient and relaunch one class up.
- The differential validation (#387) and the cutover (#389) are the
  fail-open safety nets;
  do not let schedule pressure skip them.

## The gate was running a 0.1.9 binary against a 0.3.0 parser

Found 2026-08-14 while merging main into `translation-repair-rebased`. Every
 push from that worktree was refused with `plugin-threw` and
 `Malformed forbidden-strings scanner output`, the same symptom this document
 records against `#388`, on a tree where `#388` had long since closed.

### `#388` was correct; the deployment was not

```text
built binary   forbidden-strings 0.1.9   2026-07-24
crate source   version = "0.3.0"         2026-08-05
```

Two minor versions and three weeks apart. The formats differ exactly where the
 parser broke:

```text
0.1.9   candidate-0:2:5..24 rule=415
0.3.0   candidate-0:2 rule=aws-access-token
```

So `#388`'s parser, which drops columns and reads a named rule, matches the
 SOURCE and always did. The real-binary integration test it shipped was red for
 the same reason: it spawns `target/release/forbidden-strings`, and that path
 held the pre-migration build.

`mise run //package/cli/forbidden-strings:build` resolves it. After the rebuild
 the strict parser accepts real output untouched, the integration test passes,
 and a full `--all --builtin-rules` sweep of the tree reports nothing.

### The wrong fix, recorded because it was tempting

The first response was to teach the parser to accept both shapes. That is worse
 than the bug. A stale binary scans with a stale rule BASELINE, so a tolerant
 parser makes the gate report success while missing whatever the current rules
 catch, and it does so silently. Fail-closed is correct here; the tolerance was
 reverted.

What survives is the diagnostic. All five throw sites now name the expected
 shape, identify a column span or a numeric rule id as the pre-0.3.0 format, say
 the executed binary predates the crate source, and give the rebuild command.

### Why nothing caught it

The integration test exists precisely to catch this drift, and it did: it was
 red. Nobody read it. That is the same shape as every other defect found in this
 session's sweep, a signal produced and not consumed, and it is worth noting that
 the signal here was a FAILING TEST rather than a log line.

Two smaller consequences worth keeping. The gate's numeric `rule=` ids in old
 reports (`rule=28`, `rule=152`, `rule=183`, `rule=415`) are 0.1.9 indices and do
 not correspond to anything in the current named-rule file, so any earlier
 triage that resolved them by line number in `.cache/forbidden-strings.rules.txt`
 was reading an unrelated table. And the appendix self-match that
 `doc/troubleshooting/forbidden-strings-appendix-self-match.md` documents no
 longer trips under 0.3.0.
