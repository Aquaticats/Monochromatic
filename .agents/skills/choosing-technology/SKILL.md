---
name: choosing-technology
description: >-
  Selects and vets SaaS vendors, libraries, frameworks, build tools, local executables, and replacement dependencies
  through hard gates, source audits, validation, and scored comparison. Use when choosing, recommending, evaluating,
  vetting, replacing, or comparing technologies or vendors for this repo.
---

# Choosing technology and vendors

Use this workflow before recommending:

- a SaaS or managed service;
- a library,
  framework,
  build tool,
  local executable,
  or self-hosted service;
- a proprietary local tool;
- a replacement for an incumbent dependency;
- a plugin,
  hook,
  CI component,
  credential handler,
  native artifact,
  Wasm artifact,
  or prebuilt binary.

Candidates may be named during discovery.
The prohibition is recommending a winner before every applicable gate is complete.

Walk every applicable section.
Mark a non-applicable gate with its reason;
do not silently skip it.
Hard-gate failure removes a candidate.
Soft evidence affects score only after every surviving finalist receives equal-depth validation.

## Required deliverables

A completed selection produces:

- a candidate ledger;
- evidence records for every gate and score;
- a completed vet report when the substantial-evaluation threshold is crossed;
- hard-gate outcomes;
- score calculations and sensitivity results;
- pros and cons for every ranked finalist;
- a fully sorted ranking with the reason for each adjacent ordering;
- a recommendation or an explicit no-recommendation result;
- a decision document only after adoption is separately authorized.

Documentation,
reports,
and Markdown artifacts belong directly in the main worktree.
Product code,
dependencies,
configuration,
builds,
installations,
and experimental mutations remain outside an evaluation request.

## Lifecycle

Use these states in order:

1. **Discovered**:
   a required discovery source produced the candidate.
2. **Screened**:
   metadata evidence checked category fit and every screening hard gate.
3. **Serious alternative**:
   no screening hard gate failed and targeted evidence is warranted.
4. **Hard-gate confirmed**:
   targeted evidence confirmed constraints,
   license,
   provenance,
   security,
   reproducibility,
   inspectability,
   and category fit.
5. **Finalist**:
   every hard-gate-confirmed candidate enters full source and runtime validation.
6. **Validated**:
   the candidate completed every required upstream,
   platform,
   and consumer-boundary check.
7. **Scored**:
   every validated finalist received the same frozen criteria,
   rating evidence,
   and sensitivity procedure.
8. **Recommended**:
   the report is complete and the fully sorted order is stable.
9. **Adopted**:
   a later action request authorizes product changes and a decision record.

Only a hard-gate failure or product-category mismatch exits a candidate before finalist validation.
A preliminary soft score never controls promotion.

## Context forks before discovery

First read repository,
deployment,
and incumbent facts that can be measured.
Do not ask the user for facts available from source,
configuration,
history,
registries,
or primary documentation.

Ask only unresolved preferences that would change:

- the candidate set;
- a hard constraint;
- a score weight that can change ordering;
- adoption authority.

Typical context forks:

- primary delivery versus backup;
- personal versus business-critical workload;
- self-hosted versus managed;
- free-only versus a price ceiling;
- geography,
  compliance,
  residency,
  privacy,
  or retention;
- single-user,
  team,
  or public-facing use;
- runtime,
  browser,
  operating-system,
  architecture,
  and deployment constraints;
- trust boundary:
  credentials,
  agent execution,
  CI execution,
  native code,
  Wasm,
  or sensitive data;
- feature breadth versus human auditability.

Ask no rubber-stamp question.
When settled requirements determine the answer,
record the inference and continue.

### Check in before broad discovery

Each broad vetting process checks in with the user before enumerating all possible candidates,
because the user may delegate the enumeration to a cheaper model of their choosing
(not necessarily Haiku)
or run it themselves.
A vetting process is broad when it names or finds candidates from anywhere other than:

1.  the agent's memory or general knowledge;
2.  the user's picks;
3.  repository precedent (incumbents,
    prior vet reports,
    decision records).

Vetting only candidates from those three sources needs no check-in.

Ask through `AskUserQuestion`,
one question per vetting process,
stating the subject,
the context forks settled so far,
the source classes,
and the draft query schedule.
Offer who runs it:
this session,
a model the user names,
or the user.
Wait for the answer before the first discovery query.
This question decides who does the work,
so it is never a rubber stamp.

When the user delegates,
write the settled context,
hard constraints,
and frozen query schedule into the vet report first,
so the delegate starts from the report alone.

## Classify base components and overlays

Classify every candidate component into exactly one base category.

### Managed service or SaaS

The adopted component is a hosted service or control plane.
An open-source engine or client does not make a hosted control plane open source.

### Inspectable open-source local technology

This includes a library,
framework,
build tool,
local executable,
or self-hosted service whose relevant source is inspectable.

### Proprietary local technology

This includes installed software whose relevant source is unavailable.
It remains eligible only under the open-source exception and cannot pass a high-trust overlay.

### Cross-cutting overlays

Apply every matching overlay:

- incumbent dependency replacement;
- high-trust execution in an agent,
  plugin,
  hook,
  CI runner,
  or credential boundary;
- native,
  Wasm,
  prebuilt binary,
  or generated-code boundary;
- sensitive data,
  privacy,
  compliance,
  residency,
  retention,
  or geography;
- multi-platform or browser-baseline claim.

A hosted service plus a local client has two components and receives two base classifications.
Base and overlay gates accumulate.

## Shared invariants

### Open-source precedence

Open-source candidates precede proprietary candidates unless:

- the user explicitly requests proprietary technology;
- every open-source candidate fails a hard constraint.

A proprietary exception must name the failed open-source constraints.
A proprietary high-trust component fails inspectability and is ineligible.

### Constraint fit before stack fit

A hard performance,
scale,
latency,
compatibility,
platform,
or deployment constraint selects the technology.
Existing stack familiarity is a soft criterion unless the user makes it a hard constraint.

### Existing tools before custom implementation

Survey ready-to-use technologies before proposing a custom implementation.
A custom implementation is eligible only when every existing tool fails a named hard constraint.

### Alternatives

Name at least two concrete alternatives with evidence-backed exit or ranking reasons.
If saturated discovery finds one survivor,
record every exhausted source and continue with that survivor.
If it finds zero,
return the no-survivor terminal result.
Never invent an alternative to satisfy a count.

### Equal-depth finalist validation

Every hard-gate-confirmed survivor becomes a finalist.
Every finalist receives the full applicable source,
maintenance,
upstream,
platform,
and consumer-boundary validation.
Do not validate a likely winner more deeply than another survivor.

## Discovery protocol

Create a candidate ledger before recommending.
Each row is a Markdown subsection,
not a pipe table.
Record candidate,
discovery source,
base category,
overlays,
and screening result.

### Required source classes

Search all applicable classes:

1. Category package registry,
   vendor directory,
   or official ecosystem index.
2. Repository-host topics,
   code,
   releases,
   and organization projects.
3. Broader web searches for peer tools,
   comparison terms,
   and nearest comparable technologies.
4. This repository's incumbent,
   parallel systems,
   prior vet reports,
   decision records,
   and hand-rolled alternatives.

Use category-appropriate registries and source hosts.
Do not treat npm and GitHub as universal sources.

### Freeze the query schedule

Before executing discovery,
record a finite literal query schedule for each source class.
Include:

- exact problem class;
- known synonyms;
- protocol or format;
- incumbent plus `alternative`;
- deployment model;
- ecosystem;
- category-specific registry keywords.

After the initial schedule:

1. Collect every new taxonomy term recorded in the ledger.
2. Append one de-duplicated expansion round.
3. Freeze the schedule.
4. Record later taxonomy terms without recursively appending more queries.

### Record every query

For each query record:

- literal query;
- provider or registry;
- include and exclude filters;
- sort order;
- page or cursor;
- result count;
- newly discovered candidates;
- newly discovered screening survivors.

Do not truncate or negatively filter results without recording what the filter can hide.

### Pagination and saturation

A screening survivor has:

- no known hard-gate failure;
- each screening gate marked pass or pending targeted evidence;
- plausible category fit;
- apparently inspectable source and execution where required.

For paginated sources,
continue until:

- two consecutive complete pages add no screening survivor;
- the provider reports exhaustion.

A provider cap reached before either condition marks the source class blocked,
not saturated.
Use an uncapped API or independent enumeration source.
If no replacement can complete the class,
recommend none.

A source class is saturated only when its frozen query schedule and page rule finish.
Discovery is saturated only when every required source class is saturated.

Record one terminal discovery result:

- saturated with at least two survivors;
- saturated with one survivor;
- saturated with zero survivors;
- blocked by an unenumerable required source.

## Screening hard gates

Apply before targeted audits.
A candidate exits when evidence proves:

- category mismatch;
- hard-constraint failure;
- incompatible license or terms;
- required source is unavailable;
- high-trust code is proprietary or uninspectable;
- native,
  Wasm,
  or prebuilt provenance cannot be mapped to source;
- build provenance is unknown;
- security boundary required by the use cannot be established;
- reproducible validation has no inspectable path;
- a required platform or browser is unsupported.

NDA-only evidence does not satisfy a public audit gate.
A sparse issue tracker is not a hard failure.

## Vet-report lifecycle

### Substantial-evaluation threshold

Create or reopen a vet report when any condition holds:

- an evaluation promotes a serious alternative using external source,
  vendor,
  maintenance,
  clone,
  or execution evidence;
- external discovery and screening saturate with no serious alternative;
- a required external discovery source remains blocked after alternate enumeration paths are exhausted.

A narrow factual lookup or application of a current decision record does not create a redundant report.

### Report path and subject slug

Use:

```text
# .agents/skills/choosing-technology/SKILL.md
doc/audit/<subject>-vet-YYYY-MM-DD.md
```

Build `<subject>` by scanning the Unicode NFC subject:

- preserve ASCII letters and digits in lowercase;
- replace each run of every other code point with one hyphen;
- trim leading and trailing hyphens;
- use `technology` when nothing remains;
- prefix `tech-`;
- cap at 48 ASCII characters and trim a final hyphen.

The fixed prefix and alphabet avoid path separators,
Windows reserved names,
trailing dots,
and trailing spaces.

### Compatibility fingerprint

Build the fingerprint input from:

- schema version 1;
- subject;
- decision scope;
- hard constraints;
- deployment;
- trust boundary;
- incumbent name and version or null;
- base categories;
- overlays.

Before canonicalization:

- normalize every string to Unicode NFC;
- remove outer whitespace from subject and scope only;
- preserve internal whitespace;
- de-duplicate set-valued arrays and sort by raw UTF-16 code-unit order;
- preserve ordered arrays;
- sort deployment keys recursively by raw UTF-16 code-unit order;
- preserve empty strings;
- encode absent optional scalars as JSON null;
- reject duplicate keys,
  lone surrogates,
  non-finite numbers,
  and values outside I-JSON.

Serialize with the
[RFC 8785 JSON Canonicalization Scheme](https://www.rfc-editor.org/rfc/rfc8785).
Hash the exact UTF-8 bytes with SHA-256.
Store the full hash in report metadata.

### Select or create the report

Search reports with the same subject slug.
A report is compatible only when its full fingerprint matches.

When several compatible reports exist:

1. Select greatest `last-updated` date.
2. Break a remaining tie by lexicographically smallest repo-relative path.
3. Record duplicate paths.

For an incompatible same-day report:

1. Start a context qualifier with the first eight fingerprint characters.
2. If occupied by a different fingerprint,
   extend four characters at a time.
3. Continue through all 64 characters.
4. A full-hash path with a different stored fingerprint is corruption and blocks the write.

Mutable status,
owner,
last-updated date,
and skill revision do not participate in collision identity.

### Atomic report writes

Before each report edit:

1. Run `mkdir --parents "${HOME}/temp/agent/technology-vet-locks"`,
   then `chmod 700 "${HOME}/temp/agent" "${HOME}/temp/agent/technology-vet-locks"`.
2. Derive a lock key from the absolute report path.
3. Atomically create the lock with create-new semantics.
4. Record session ID,
   process ID,
   start time,
   report path,
   and pre-edit SHA-256.
5. Rerun report selection after acquiring the lock.
6. Create a new final path with create-new semantics.
7. For an update,
   write a sibling temporary file,
   re-read the report hash,
   abort if it changed,
   then atomically rename the temporary file.
8. Release the lock after durable completion.

Treat a lock as stale only when:

- its process is absent;
- its recorded start is more than 30 minutes old.

If either fact cannot be established,
block and report the owner rather than deleting the lock.

### Update throughout the audit

Create the report as soon as the threshold is crossed.
When threshold crossing follows completed screening,
snapshot context,
queries,
ledger,
and exits immediately.

Update and commit the scoped report after:

- context and rubric freeze;
- discovery;
- screening;
- targeted evidence;
- finalist validation;
- scoring and sensitivity;
- synthesis.

Complete the report before returning a recommendation.

## Evidence records

Every gate,
score,
and recommendation claim receives an evidence record containing:

- candidate and exact version,
  plan,
  release,
  revision,
  artifact checksum,
  or service plan;
- claim and decision relevance;
- base gate or overlay gate;
- hard-gate,
  scored-concern,
  low-signal,
  not-applicable,
  pass,
  fail,
  or excluded status;
- primary URL,
  page or document section,
  and access date;
- independent corroboration and counterevidence when material;
- clone path,
  commit or tag,
  source `path:line` range,
  and adjacent excerpt for code-behavior claims;
- exact command,
  working directory,
  OS,
  architecture,
  container or VM image and digest,
  environment boundaries,
  exit status,
  elapsed time,
  and relevant output excerpt or log path;
- score criterion,
  weight,
  rating,
  confidence,
  score effect,
  and sensitivity range when scored;
- outcome and reason.

Reviews,
aggregators,
and tracker counts can discover or corroborate a claim.
They do not replace primary terms,
status history,
security disclosure,
source,
or reproduced behavior.

## Managed service and SaaS gates

Inspect every domain for every SaaS candidate.
A finding affects score only when the report explains its causal link to the proposed use.
An inspected irrelevant finding receives no score effect.

### Retained historical domains

1. **Layoffs and headcount over 24 months**:
   record dated events,
   affected functions,
   and evidence quality.
2. **Customer reviews**:
   inspect account suspension,
   billing automation,
   support,
   cancellation,
   and appeal patterns.
3. **Outages over 12 months**:
   use the official status history plus independent corroboration;
   separate affected services and regions.
4. **Funding,
   ownership,
   and business model**:
   record bootstrapping,
   VC,
   private equity,
   acquisition,
   divestiture,
   and revenue model.
5. **Signup friction**:
   inspect email-domain,
   KYC,
   payment,
   geography,
   and account-approval restrictions.
6. **Security and abuse history**:
   inspect breaches,
   phishing or abuse reputation,
   disclosure quality,
   and response behavior.

### Direct operational domains

Inspect:

- terms and pricing stability;
- account suspension,
  enforcement,
  appeal,
  and termination;
- security,
  privacy,
  data use,
  and compliance;
- availability,
  SLA,
  and support;
- data export,
  portability,
  retention,
  and deletion;
- lock-in,
  migration,
  and exit cost;
- ownership changes and business continuity;
- API deprecation and compatibility policy;
- geography and access restrictions.

Official terms,
status,
security,
privacy,
pricing,
and export documentation are primary sources.
Review sites,
layoff trackers,
and aggregators are leads or corroboration.

## Open-source technology gates

Clone every serious alternative and finalist under a private `~/temp/agent/` root using the repository
cloning convention.
Record exact clone path and revision.
Do not recommend from registry metadata,
README,
stars,
or issue count alone.

### License and dependency surface

Inspect:

- license text and compatibility;
- package manifest;
- runtime and optional dependencies;
- transitive dependency tree;
- lifecycle scripts;
- native,
  Wasm,
  generated,
  and downloaded artifacts;
- same-author packages that extend the audit surface.

### Production source

Read the source paths that implement:

- promised behavior;
- integration boundary;
- error handling;
- credentials,
  network,
  filesystem,
  parser,
  escaping,
  or serialization boundaries;
- native or Wasm host functions;
- build and release provenance.

Cite paths,
lines,
and excerpts.

### Tests and CI

Inspect:

- unit tests;
- integration tests;
- end-to-end tests;
- property tests;
- fixtures;
- platform and feature matrices;
- coverage configuration and published reports;
- required versus optional CI jobs.

Search for fuzzing and mutation evidence,
including language-equivalent tools and harnesses.
Report absence inline.
When present,
read invariants,
corpora,
generators,
and CI wiring.

### Source quality

Inspect:

- module and file size;
- type strictness;
- lint policy;
- generated-code boundaries;
- native or Wasm safety boundaries;
- syntax-boundary tests;
- unreachable-state handling;
- error propagation.

## Maintenance audit

Issue count alone is not a maintenance signal.

For each open-source candidate:

- inspect issues created or updated in the last 12 months;
- if at most 20 exist,
  inspect all;
- otherwise inspect the 10 most recently updated without cherry-picking;
- count maintainer comments separately from maintainer actions such as labels,
  assignment,
  closure,
  linking,
  or milestone changes;
- inspect the 10 most recently updated pull requests in the same way when available;
- measure maintainer-authored work,
  reviews,
  merge latency,
  and unreviewed external fixes;
- inspect latest publish,
  releases,
  changelog links,
  and whether maintenance merges reach a release;
- inspect stale unanswered bugs,
  compatibility failures,
  security reports,
  and abandonment comments;
- measure maintainer concentration.

Distinguish:

- active releases with weak public issue support;
- responsive maintainers with a triaged backlog;
- low-signal tracker activity in a project evaluated through source,
  release,
  ownership,
  and test evidence;
- abandoned or effectively unmaintained.

Zero tracker activity is low-signal,
not proof of health.

## Replacement parity overlay

When replacing an incumbent:

- inventory every exported and consumed incumbent path;
- identify the exact incumbent defect or constraint failure;
- audit each survivor to the incumbent's depth;
- compare transitive dependencies;
- read candidate source handling the incumbent's failure;
- inspect native or Wasm compiler flags,
  imported host functions,
  source archives,
  checksums,
  and release verification;
- compare maintenance with the same sample method;
- exercise every consumer boundary;
- include keeping the incumbent as a candidate.

A replacement is incomplete when it trades a known flaw for an unaudited flaw.

RCO:
 Removing an incumbent:
 ledger every consumed responsibility with its owner,
 selection status,
 parity test,
 and retired behavior;
 recommend removal only when every entry has a viable owner.

## Human-auditability overlay

Apply to code that runs inside another tool,
handles credentials,
runs in CI,
or crosses another high-trust boundary.

Measure for every finalist:

- non-test source lines and file count;
- runtime dependency count;
- same-author dependency count;
- control-flow shape;
- event bus,
  plugin handshake,
  generated-code,
  and distributed-state boundaries;
- concentration of credentials,
  network,
  filesystem,
  and process execution;
- TUI,
  UI,
  rendering,
  and platform surface.

Name the tradeoff between feature breadth and verifiability.
Do not give unused features positive weight.
A materially leaner candidate can outrank a richer candidate when both pass hard constraints.

## Native, Wasm, and prebuilt overlay

Require:

- source-to-artifact mapping;
- compiler and linker flags;
- imported and exported host functions;
- build scripts;
- source archive or commit;
- checksum or signature;
- release workflow;
- target matrix;
- reproducible verification.

Unknown build provenance is a hard failure.
A prebuilt artifact without verifiable source mapping is ineligible.

## External execution gate

Inspect before executing any third-party command tree.
This includes installs,
lifecycle scripts,
generated commands,
builds,
tests,
fuzzers,
benchmarks,
and downloaded executables.

### Execution manifest

Record:

- candidate,
  pinned revision,
  artifact checksums,
  and clone origin;
- exact top-level command;
- every statically discovered lifecycle,
  generated,
  subprocess,
  plugin,
  native,
  Wasm,
  and shell command it can reach;
- files inspected to establish the tree;
- expected reads,
  writes,
  subprocesses,
  network endpoints,
  and outputs;
- container or VM image and digest;
- memory,
  CPU,
  process,
  file-descriptor,
  disk,
  and command-specific wall-clock ceilings;
- credential,
  environment,
  home-directory,
  network,
  and repository-mount policy;
- success,
  failure,
  cleanup,
  and stop conditions.

Default isolation:

- 2 GiB memory;
- 2 CPUs;
- no ambient credentials;
- no real home-directory mount;
- read-only repository mount;
- private scratch write volume;
- network disabled after any separately inspected dependency-fetch phase.

Record and justify every deviation.
Use the process or container boundary for wall-clock ceilings rather than wrapping routine verification in
an external timeout command.

When execution reveals an undeclared command,
write,
network endpoint,
or native boundary:

1. Stop.
2. Update the manifest.
3. Inspect the new path.
4. Continue only after it passes.

If the full path cannot be inspected or bounded,
fail the candidate.
Isolation limits impact;
it does not turn unknown code into trusted code.

## Finalist validation

Before running a finalist:

1. Inventory every upstream task and CI job.
2. Inspect the commands through the execution gate.
3. Pin revision and environment.

Then:

1. Run the complete default CI-equivalent path in a secret-free,
   disposable,
   resource-bounded environment.
2. Run every non-default suite relevant to claimed behavior or repo risk,
   including test,
   integration,
   fuzz,
   native,
   Wasm,
   feature,
   and platform suites.
3. Exercise relevant platform claims on available Linux x64,
   macOS arm64,
   Windows x64,
   and required browsers.
4. Exercise the real consumer boundary after upstream validation.

For every omitted suite record:

- exact command;
- purpose;
- evidence it cannot affect any claimed or consumed surface.

Diagnose every failure.
Reject unless the failing path is proven outside every claimed and consumed surface.
A relevant suite that cannot be inspected or run fails the candidate.
Duration alone does not justify omission;
use resource isolation.

## Weighted scoring

Score only validated finalists.
Hard gates remain outside arithmetic and cannot be offset by points.

### Freeze decision-level criteria

Before candidate-specific soft evidence is rated:

- derive criteria from requirements,
  base categories,
  overlays,
  and resolved preferences;
- apply each relevant criterion to every finalist;
- give a candidate that structurally avoids a risk a strong evidenced rating rather than removing the criterion;
- remove a criterion irrelevant to the whole decision from every denominator;
- assign weight 1 to every criterion whose priority is unspecified;
- assign explicit preference weights from 1 through 5;
- publish and freeze criteria,
  applicability,
  and weights;
- count overlapping evidence once under a primary criterion.

### Ratings

Rate each finalist from 0 through 4:

- 0:
  serious concern;
- 1:
  weak;
- 2:
  acceptable;
- 3:
  good;
- 4:
  strong.

Every rating records evidence and confidence:

- high;
- medium;
- low.

A low-signal rating records evidence-supported minimum and maximum values.
Use their arithmetic midpoint as the provisional rating,
which may be a half step.
The midpoint is a baseline input,
not a confidence claim.

### Calculation

For each finalist:

```text
# .agents/skills/choosing-technology/SKILL.md
earned = sum(weight * rating)
maximum = sum(weight * 4)
normalized = earned / maximum * 100
```

Rank by the unrounded fraction.
Display raw numerator,
denominator,
and normalized score rounded to one decimal place.
Publish minimum and maximum totals from low-signal ranges.

If no soft criterion applies,
report `score: not applicable` for every finalist.

### Sensitivity

Run every test one input at a time:

- raise each equal-default weight from 1 through 5;
- move every medium-confidence and low-confidence exact rating one step down and up within 0 through 4;
- test both endpoints of every low-signal range.

When any test changes the winner or adjacent order:

1. Gather decisive evidence or ask only the controlling preference.
2. Refreeze the rubric.
3. Rerun the baseline calculation.
4. Rerun the complete weight,
   confidence,
   and range matrix.

Rank by the provisional baseline only when every defined one-at-a-time test preserves order.
Publish aggregate ranges and state that stability does not cover simultaneous multi-input changes.

If an exact tie remains,
or no criteria apply and factual tradeoffs do not order finalists,
ask for the unresolved preference.

If the rerun remains unstable,
evidence cannot resolve it,
or the user declines the preference,
finish the report with conditional rankings and recommend none.

## Recommendation output

The report begins with:

- status and lifecycle phase;
- subject and scope;
- start and last-updated dates;
- governing skill commit;
- governing skill SHA-256;
- compatibility fingerprint;
- active audit owner;
- hard constraints;
- base components and overlays;
- frozen criteria and weights;
- unresolved preferences;
- prior compatible report or reason for a new report.

The body includes:

- context;
- query schedule and discovery ledger;
- evidence records;
- execution manifests;
- hard-gate exits;
- validation results;
- score arithmetic;
- sensitivity;
- pros and cons for every finalist;
- complete ranking;
- reason for each adjacent order;
- confidence and evidence limits;
- recommendation or terminal result.

State the recommendation only after these sections are complete.
Do not bury material risk in trailing caveats.

## Terminal outcomes

### Discovery blocked

Snapshot the query ledger,
name the blocked source and attempted alternatives,
finish the report,
and recommend none.

### No serious alternative

Record every hard-gate exit,
finish the report,
recommend none,
and ask whether the user wants to change a named hard constraint.

### No validated finalist

Record each targeted or validation failure,
finish the report,
and recommend none.
Soft points cannot rescue a hard failure.

### Relevant execution unavailable

Fail that candidate and continue with other survivors.
If none remain,
recommend none.

### Sensitivity unresolved

Record conditional rankings for each outcome-changing input and recommend none.

### Score tie

Ask the controlling preference.
Do not invent a tiebreaker.

### Report conflict

Do not overwrite.
Name report path,
lock owner,
and conflicting evidence.
Resolve ownership before continuing.

### Every candidate fails

State that no current candidate is recommendable.
Do not propose an unverified custom implementation.

## Adoption boundary

Recommendation is not adoption.
An evaluation may write its vet report but must not change product code,
dependencies,
configuration,
or decision records.

After a separate action request adopts a candidate,
write or update `doc/decision/<project>.md` or a package-local decision document with:

- adopted candidate,
  version,
  revision,
  plan,
  or checksum;
- adoption date and authorizing request;
- linked vet report and governing skill revision;
- hard constraints and frozen weights;
- complete ranking and rejected alternatives;
- integration boundary;
- migration;
- exit and rollback;
- revisit triggers.

## Synthetic worked example

All names and evidence in this example are fictional.
They teach workflow only and make no real-world claim.

### Prompt and context

Prompt:
"Choose credential-handling CI automation for Linux and Windows runners.
Managed or self-hosted is acceptable.
Auditability matters more than feature breadth.
"

Measured context:

- credentials reach the automation component;
- Linux x64 and Windows x64 are hard requirements;
- exportable configuration is a hard requirement;
- no residency requirement;
- the user ranks auditability above breadth.

Base components and overlays:

- Managed candidate M:
  SaaS base plus sensitive-data and multi-platform overlays;
- Open candidate O:
  inspectable open-source local base plus high-trust and multi-platform overlays;
- Opaque candidate P:
  proprietary local base plus high-trust overlay.

### Discovery and hard gates

The fictional query ledger completes every required source class and two-page saturation rule.
It finds M,
O,
and P plus two category mismatches that exit during screening.

P fails because proprietary high-trust execution is not inspectable.
M and O pass hard gates.
Both become finalists and receive equal-depth applicable validation.

### Evidence and validation

M:

- all six historical SaaS domains inspected;
- all direct operational domains inspected;
- fictional outage evidence is relevant to CI reliability;
- unrelated fictional funding evidence receives no score effect;
- open local client source is audited and its Linux and Windows integration fixtures pass;
- service export and account-appeal paths pass.

O:

- repository,
  dependencies,
  production source,
  tests,
  CI,
  fuzzing search,
  maintenance,
  and human-auditability surface are audited;
- default CI,
  relevant non-default suites,
  Linux,
  Windows,
  and consumer-boundary fixtures pass;
- no relevant upstream failure remains.

### Frozen score

The user preference produces:

- auditability weight 5;
- reliability weight 4;
- portability weight 3;
- cost weight 2.

Maximum points:

```text
# .agents/skills/choosing-technology/SKILL.md
5 * 4 + 4 * 4 + 3 * 4 + 2 * 4 = 56
```

O ratings:

- auditability 4;
- reliability 3;
- portability 4;
- cost 3.

O score:

```text
# .agents/skills/choosing-technology/SKILL.md
5 * 4 + 4 * 3 + 3 * 4 + 2 * 3 = 50
50 / 56 * 100 = 89.3
```

M ratings:

- auditability 2;
- reliability range 3 through 4 with midpoint 3.5;
- portability 3;
- cost 2.

M provisional score:

```text
# .agents/skills/choosing-technology/SKILL.md
5 * 2 + 4 * 3.5 + 3 * 3 + 2 * 2 = 37
37 / 56 * 100 = 66.1
```

M reliability endpoint 4 produces 39 points,
or 69.6.
The complete one-at-a-time sensitivity matrix does not reverse O over M.

### Ranking and boundary

Ranking:
O > M > P.

- O beats M because both pass hard gates,
  while O's measured auditability advantage exceeds M's reliability endpoint advantage under frozen weights.
- M beats P because M passes every applicable hard gate and P fails inspectability.

Recommendation:
O.

The evaluation completes its fictional vet report.
It does not install O or create a decision record.
A later `adopt O` action would authorize integration and the decision document.

## Completion checklist

Do not recommend until every applicable item passes.

### Context and routing

- Measurable context gathered before questions.
- Outcome-changing preferences resolved.
- Every component has one base category.
- Every matching overlay is active.
- Hard constraints and soft criteria are frozen before candidate ratings.

### Discovery

- Every required source class searched.
- Literal query schedule and one expansion round recorded.
- Pagination completed or source marked blocked.
- Saturation result recorded.
- At least two alternatives named,
  or fewer justified by saturated evidence.

### Hard gates

- Category fit confirmed.
- Open-source precedence applied.
- License or terms pass.
- Provenance and inspectability pass.
- Security boundary passes.
- Required platforms and browsers pass.
- Failed candidates exit with evidence.

### Category and overlay audits

- Every SaaS historical and direct domain inspected.
- Open-source source,
  dependency,
  test,
  CI,
  fuzzing,
  mutation,
  quality,
  and maintenance audits complete.
- Replacement parity complete when applicable.
- Human-auditability measurements complete when applicable.
- Native,
  Wasm,
  and prebuilt provenance complete when applicable.
- Sensitive-data and multi-platform gates complete when applicable.

### Execution and validation

- Execution manifest approved before each command tree.
- Undeclared execution stopped and re-audited.
- Default CI-equivalent validation passed or an out-of-surface failure is proven.
- Every relevant non-default suite ran.
- Every required platform and browser ran.
- Consumer boundary exercised.
- Every omission has exact evidence.

### Scoring and recommendation

- All finalists received equal-depth validation.
- Criteria and weights published.
- Ratings,
  confidence,
  ranges,
  raw arithmetic,
  and one-decimal scores published.
- Complete sensitivity matrix ran.
- Order-changing inputs resolved and the full matrix reran.
- Pros and cons stated for every finalist.
- Fully sorted ranking and adjacent reasons stated.
- Material evidence limits appear inline.

### Artifacts and authority

- Vet report created or reused at threshold crossing.
- Report updated through every phase.
- Governing skill commit and SHA-256 recorded.
- Report fingerprint,
  path,
  lock,
  and ownership are valid.
- Report complete before recommendation.
- No product or decision mutation occurred during evaluation.
- Adoption document waits for separate authorization.

If any required item is unmet,
return the matching terminal outcome instead of recommending.

## Violation signals

Stop and correct course when you are about to:

- recommend before discovery saturation;
- treat candidate naming as recommendation;
- apply every SaaS finding to score without relevance;
- call a hosted control plane open source because its engine is open source;
- use open issue count as maintenance evidence;
- recommend from README or registry metadata without cloning;
- skip a hard-gate survivor because its preliminary soft evidence looks weaker;
- run third-party code before inspecting and manifesting its command tree;
- treat isolation as proof of trust;
- accept unknown build provenance;
- score a hard failure;
- change weights after seeing a preferred winner;
- omit sensitivity reruns after a preference answer;
- force a tiebreaker without a user preference;
- recommend when discovery,
  validation,
  or ranking remains blocked;
- write a decision record before adoption;
- leave a substantial evaluation without a completed vet report.
