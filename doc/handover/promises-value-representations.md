# Promise lesson value representations

## Current request and constraints

The user identified missed opportunities to show `JSON.stringify` representations of objects and Promises.
They then requested `util.inspect` where JSON is insufficient,
and added `showify` to the workspace catalog.
`pnpm-workspace.yaml:131` currently declares `showify: '>=0.2.8'`.

The pre-change lesson is preserved as `doc/planning/promises-teaching-before-value-views.local.html`,
SHA-256 `319ffcb20481d468beb6353d6bbe94b682e55cb682c3542dffefc983f6e82eb7`.
The current lesson contains the value views and has completed integrated verification.
The previous completed repairs are recorded in [the correction handover](promises-review-corrections.md).

Preserve:

- The learner profile and Promise-focused destination.
- Independent sending,
  no click races,
  and truthful fixture policies.
- Native code editing,
  opaque iframe isolation,
  and self-contained offline HTML.
- Complete print/export counterparts and achromatic long-form reading surfaces.
- The settled learner-observation veto.
- The withdrawal of mandatory visible-window presentation;
  headless verification is valid.

No final teaching-skill confirmation or implementation is authorized.

## Work in progress

Task 29 completed the representation audit and bounded consumer probe of the user-supplied formatter.
Task 30 implemented contextual views.
Task 31 completed integration verification and recorded the resulting teaching-discovery evidence.

Research workspace:
`/var/home/user/temp/agent/promises-value-inspection`.
Lesson authoring workspace:
`/var/home/user/temp/agent/promises-revision`.
The root devDependencies now consume `showify: 'catalog:'`,
resolved to 0.2.8.
`proc_d778` installed it with lifecycle scripts disabled;
the generated lockfile adds only Showify.
Pnpm also sorted existing devDependency keys without changing their versions.
The dependency change was committed separately.
The formatter is now bundled into the lesson and learning exports.
The first settlement-UX prototypes omitted the resolver-bundle and inner-Promise snapshot cards.
The integrated A-neutral lesson retains the current bundle and inner Promise in visible cards,
uses `window.lessonValues.captureValue` for the same audited JSON/showify representations,
and groups additional current/earlier captures in disclosures that expand for print.
The generated lesson,
actual PDF,
Firefox ESR,
and downloaded artifacts were verified.
[The settlement-UX handover](promises-settlement-ux.md) records that redesign.

## Existing omissions

`promises-revision/labs.mjs`,
`installMonitor`,
currently:

- Replaces a Promise with the prose `Promise object; no public state field`.
- Replaces an Error with its name and message.
- Formats arrays itself to preserve empty slots.
- Uses JSON for other values without naming that representation.

The data example constructs a reply object but reports only its text list,
count,
and comparison.
The Promise collection does not show its input Promise array or returned collection Promise.
The adoption example does not show either actual Promise object.
The resolver bundle,
error metadata,
and cancellation objects offer additional useful inspection points.

## Measured native output

`mise run probe` in the research workspace passed as `proc_4981`.
`probe-values.mjs` writes `native-inspection-probe.json` using Node `v26.8.1`.
It explicitly sets `Error.stackTraceLimit = 0` for its authored examples,
so error output focuses on name/message and fixture metadata rather than host stack paths.

Observed examples:

- Pending,
  fulfilled,
  rejected,
  and adopting Promise objects all stringify as `{}`.
- Native `util.inspect` distinguishes pending,
  fulfilled contents,
  and rejection.
- A resolver bundle stringifies as `{"promise":{}}`;
  inspect also shows its resolving functions.
- Sparse reply slots become JSON `null` entries;
  inspect identifies empty items.
- A callback array stringifies as `[null]`;
  inspect identifies the stored function.
- Top-level `undefined` produces no JSON text,
  not the JSON string `"undefined"`.
- JSON of the fixture Error includes added enumerable metadata but omits its ordinary message.
- Native inspection exposes controller/signal state;
  explicit public-property snapshots also expose `aborted` and `reason`.
- Circular objects and BigInt throw during JSON serialization but have inspector representations.

Node source is pinned at tag `v26.8.1`,
commit `7be6d3af31a65adea57c94c41e50c2b071ed0b3a`.
The treeless,
no-checkout clone is `promises-value-inspection/node-v26.8.1`.
Extracted source files are `native-inspect-source.js`,
`native-util-source.cc`,
and `native-util-doc.md`.
`src/node_util.cc:102` defines `GetPromiseDetails`;
it uses native Promise state/result access.
This is not an ordinary browser-page JavaScript property.

## User-supplied showify

Read-only clone:
`promises-value-inspection/showify`.
Commit and release tag `0.2.8` both identify `9a455119e0735ec2fc2a2de8ea14bb925fd52c38`.
The complete `src/index.ts`,
package manifest,
Promise tests,
CI workflow,
MPL-2.0 license,
and commercial-license explanation have been read.

Relevant source:

- `src/index.ts:1268` emits `<state unknown>` for a Promise.
- `test/promise.spec.ts` expects that spelling and tests added properties and a custom tag.
- Options include depth,
  indentation,
  length bounds,
  getter control,
  and custom-hook controls.
- Default custom inspection hooks can execute supplied functions;
  the teaching viewer should configure those deliberately rather than claiming arbitrary inspection is passive.
- The source has no runtime imports or network/filesystem entry points.
- The upstream `prepare` script copies Git hooks.
  Do not execute it for this task.

The bounded consumer probe now passes for the published artifact.
`proc_506a` verified Chromium 153.0.8010.12 and Firefox 155.0,
with no CSP violations or page errors.
The lesson's Firefox ESR 140 integration remains a later verification step.
The complete findings are in [the inspection boundary note](../troubleshooting/showify-promise-inspection.md).
The cached Playwright container matches installed `@playwright/test` 1.63.0.
Use 2 GiB memory,
2 CPUs,
no network or ambient credentials,
read-only dependency/source mounts,
and a private output directory.

### Consumer-probe execution manifest

`build:showify-probe` uses the repository's existing Rolldown build API to parse the pinned TypeScript source
and produce one IIFE with its MPL notice retained.
It executes no upstream npm,
build,
test,
prepare,
or hook script.
The source is one dependency-free production file and has been read completely.

`probe:showify-browser` runs the authored `probe-showify-browser.mjs` in cached image
`4b8805002ee369c81b7826b941afc52c0c9678a0d0e427a0fefd1684b78b5f94`.
That image contains Node `v24.20.0` and the Playwright browsers;
`playwright.Dockerfile` pins the base image and the installed test package is `1.63.0`.
The script imports that existing Playwright package and launches Chromium and Firefox sequentially.
It loads only the generated formatter and bounded authored values.
No upstream lifecycle commands or benchmark suites are invoked.

Bounds:
2 GiB RAM,
2 CPUs,
256 process/thread entries,
read-only root filesystem,
512 MiB `/tmp`,
256 MiB shared memory,
no network,
and a 60-second controller watchdog.
Only the script/bundle and dependency directory are mounted read-only;
the private results directory is the sole persistent write mount.
No real home,
repository source tree,
or ambient credentials are mounted.
SELinux label enforcement is disabled for those isolated mounts without altering their host labels.

The initial source bundle failed in `proc_8dc2` before executing the formatter:
Rolldown 1.2.7 reported `[PARSE_ERROR] Identifier show has already been declared`
for the source's ambient function declaration and exported constant at lines 271 and 280.
The upstream publishing script erases these TypeScript declarations first.
The next probe uses the normal published JavaScript artifact rather than patching the third-party clone.
`fetch:showify` retrieves version 0.2.8 as data and verifies its registry SHA-512 integrity before extraction.

The probe checks Promise states as unknown,
functions,
sparse arrays,
errors,
circles,
BigInt,
JSON contrasts,
ordinary getters and configured hooks,
special-tag getter reads,
CSP violations,
and whether formatting changes unhandled-rejection behavior.
Successful completion writes `results/showify-browser-probe.json` and closes both browsers.

### Consumer-probe results

The published archive is 117,323 bytes,
unpacked size 247,842 bytes,
with registry `gitHead` matching the inspected release commit.
`index.js` SHA-256 is `a2e36462757588bfbdc6aae926dbc967bc91718fa2d8dbadb688c95faac2b745`.
The generated probe IIFE measured 22,084 bytes.

`proc_6e5e` passed Chromium but Firefox failed before executing the library with
`Could not find profile folder.`
Providing a private writable HOME/cache under the container's bounded `/tmp`
resolved that environment failure without making the root filesystem writable.
`proc_506a` passed both browser engines in 3 seconds.

## Other factual lookups

No alternative package has been selected or installed.
These were capability checks,
not completed technology recommendations:

- `util-inspect-isomorphic`,
  commit `5bf9910a371fc7c61045cacd00fc1e973cc40175`,
  emits `<uninspectable>` for Promise internals.
- `node-inspect-extracted`,
  commit `ebee2b2c2c19d1e8ba70a518e7dc5f918f4e8427`,
  has `src/util.js:52` return the pending marker for every Promise.
  Do not use that output as a truthful lifecycle display.

## Audited teaching locations

- Data:
  the reply object,
  its selected string property,
  and the growing array.
- Callback delivery:
  the supplied function value,
  the initiating call's return,
  and the actual reply object.
- Callback collection:
  partial arrays with empty slots,
  then the completed collection.
- Constructor:
  the returned Promise versus its observer's received value.
- Resolver bundle and manual settlement:
  Promise plus resolving functions,
  and observed outcomes.
- Promise collection:
  the array of Promise inputs,
  the collection Promise,
  and the resulting values.
- Adoption:
  distinct inner/outer Promise objects and the separately observed adopted result.
- Await:
  the input Promise,
  async invocation's returned Promise,
  and resumed value.
- Supplied API and rejection:
  request options,
  reply data,
  actual Error,
  and explicitly selected error fields.
- Cancellation and ownership:
  controller/signal objects,
  public aborted/reason fields,
  and controller collections.

Keep the final reference chat's functional code independent of the viewer.
The existing narrative monitor need not become a formatting dump;
add explicitly labelled inspection snapshots at the relevant boundaries.
For opaque previews,
formatted strings can cross the existing source/run-validated ownership boundary;
parent-page cards avoid hiding the new content inside a fixed-height iframe.
Standalone exercise exports must render those same snapshots locally.

## Implemented value views

New authoring modules:

- `value-capture.mjs`:
  actual JSON and Showify calls,
  labelled failures and bounds,
  payload validation.
- `value-render.mjs` and `value-view.css`:
  plain-text paired representations in document flow.
- `value-view.mjs`:
  per-run recorders;
  opaque children post only serialized text snapshots.
- `value-host.mjs`:
  source/run/sequence checks,
  stale-run clearing,
  and source download.
- `bundle-values.mjs`:
  catalog package bundling and exact corresponding-source checks.
- `value-assets.mjs`:
  MPL notice,
  source links,
  and embedded preferred TypeScript source.
- `value-runtime-patches.mjs` and `value-workshop-patches.mjs`:
  targeted instrumentation,
  not replacement semantics.
- `native-inspection-note.mjs`:
  clearly labelled,
  genuinely generated Node inspector comparison.

`showify-0.2.8.source.txt` preserves the preferred source without modifying the upstream checkout.
Its SHA-256 is `148de3a0c105b8c241a5b6a04b91a8275b9ba12437b1af6d3d026d9633465e51`.

The data chapter introduces JSON,
the supplied `inspectValue({ label, value })` helper,
Showify's role,
and the `typeof` label before relying on them.
Snapshots now accompany objects,
selected properties,
callback values/lists,
partial collections,
returned Promises,
resolver bundles,
observed outcomes,
adoption,
async completion,
errors,
signals,
and ownership collections.
The shop's snapshots stay inside its optional source disclosure,
after the meaningful activity.

Embedded previews send snapshots to validated parent-owned panels,
so value representations do not become another clipped iframe pocket.
Standalone learning exports render locally and retain the formatter notice/license/source link.
The separate reference chat's functional code and export remain independent of the viewer.
All formatted characters are inserted through textContent.

`proc_3dd6` passed the first contextual integration check in 21 seconds.
`proc_f213` rebuilt,
passed the expanded value-view test including the shop callback list,
and passed the retained shop tests in 26 seconds.
Further static print explanations,
label wrapping,
and the explicit display-limit legend were added afterward.

The expanded value-view driver now covers constructor/adoption rejection,
retry metadata,
cancellation/deadline signal projections,
conversation destinations,
serializer failures,
label/text bounds,
protocol rejection cases,
and the actual preferred-source download.
Print inventory collection now includes figcaption so formatter labels are checked alongside their values.
The view's limits and the distinction between JSON absence and formatting failure are explicit.

`proc_05f2` stopped at an added edited-shop diagnostic check after the representation assertions passed.
The test timed out waiting for the parent snapshot to include the error message.
`proc_3f0c` showed that the child monitor contained the correct error,
but body.innerText and the parent snapshot omitted it.
A missing-monitor hypothesis was wrong:
`shop-preview.html` already supplied a monitor inside a hidden section.
The unused fallback construction was removed.
`report()` now reveals its existing monitor section when it actually has a report;
the shop's section is labelled Preview diagnostics and stays hidden during its ordinary activity.
This changes document-level diagnostic visibility,
not the withdrawn OS-window requirement.

`proc_64db` passed the rebuilt value-view driver and edited-shop diagnostic check in 36 seconds.
Native print and offline fixtures are running as `proc_e039` and `proc_cb53`.
The Firefox ESR fixture is `proc_0ee4`.
`proc_a9d3` passed actual value-view messages,
rejection fields,
undefined async completion,
and the retained Firefox ESR 140.15.0 regression in 26 seconds.

`proc_df6a` passed value,
ownership,
capstone,
shop,
neutral-surface,
and foundation checks,
then stopped at the PDF whole-block inventory assertion.
The supposedly missing settlement-record values remained in both the DOM and extracted PDF text.
Their parallel columns interleaved in the layout-preserving text extraction,
so the whole expected block was no longer a contiguous substring.
Print now stacks the two representations at full width;
browser HTML keeps its responsive side-by-side layout.
`proc_d679` passed the final print inventory and native Print/Cancel checks in 12 seconds.

`proc_422e` passed light/dark 1280/390 CSS-pixel value-card layouts,
neutral code backgrounds,
long-label/text bounds,
retained-row limits,
reset,
and the visible formatting-error state in 5 seconds.
The resulting desktop,
mobile,
and error-state screenshots were inspected.

### Secondary review

The focused Advisor review completed with `openai-codex/gpt-5.3-codex-spark`.
Its purity concern is addressed explicitly:
JSON runs first,
then Showify,
and custom hooks or special properties can change what the second call sees.
The library configuration is not presented as a universal side-effect guarantee.
The reference export now has a check that the teaching inspector runtime was not added to it.

An origin-string check was not substituted for ownership validation:
opaque preview origins are not unique identities.
The driver additionally sends a forged workshop packet from another owned opaque preview,
so the existing exact source-window check is exercised across previews as well as against the parent page.
Run,
sequence,
schema,
and bounded text checks remain in place.

`proc_dfa6` passed the complete combined suite,
reopened exports,
and representative PDF rendering in 324 seconds.
The full stdout log was inspected.
`proc_5be7` then passed the final artifact's value views and retained regression in Firefox ESR 140.15.0
in 25 seconds.
No visible-window completion gate applies.

## Verified artifact and cleanup

Current lesson:
`doc/planning/promises-teaching.local.html`.

- Size:
  655,250 bytes.
- SHA-256:
  `5401b0df5d9b1eee7f987086e1cef6d60d4e109b63c7bd61d818ed7ef8a05002`.
- The pre-value-view version remains preserved with its `319ffcb2…` hash.

The generated PDF has 121 Letter pages and is 2,413,882 bytes.
Whole-content checks cover 714 teaching entries and 66 appendix entries,
including formatter labels,
representations,
sources,
current work,
and notices.
Whitespace-normalized inventory success does not claim every page was visually reviewed.
Pages 11,
12,
20,
21,
23,
24,
25,
30,
31,
113,
and 114 were rendered and inspected.
Desktop/light,
mobile/dark,
and formatting-error value-card screenshots were also inspected.

The actual preferred-source download matched the preserved Showify TypeScript byte-for-byte.
The standalone reference export explicitly excludes the teaching inspector runtime,
and retained independent sending,
cancellation,
and offline checks passed.

`proc_1bca` closed the owned print and general verification browser controllers.
The managed print,
proxy/canary,
and Firefox fixtures were stopped.
`proc_156e` removed only the recorded print/export/Firefox profiles;
its final session list retained `promises-open-chat-present`.
Existing user-facing tabs and drafts were not reloaded or altered for this change.
The verified file is the delivery artifact,
not a mandatory window activation.

The teaching-discovery and proposed acceptance notes now record this representation gap.
The learner-observation veto remains settled,
and the final teaching skill is still unconfirmed.

## Integration direction to verify

Use actual browser representations at the relevant source/observation boundaries,
not prose substitutes or a detached appendix of unrelated values.
Keep raw JSON,
richer inspection text,
and program-observed state visibly distinct.
An inspector's `<state unknown>` is not the pending state.
Any native Node inspector examples must be labeled as recorded Node output,
not presented as live inspection of an edited browser value.

The viewer is lesson infrastructure,
not a new dependency required by the learner's chat.
License/source notices and all displayed representation content must survive export and printing.
