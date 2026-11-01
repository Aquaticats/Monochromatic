# Translation repair: the provider-free preparation layer is torn down

Decision of 2026-09-16,
taken under the owner's authorization of the same day:
"You have full authorization to fix/teardown whatever GPT did and use funds using your own reasonable judgement."

## What was removed

`cbedea357` removes 143 production modules,
47 tests and fixtures,
three build entries (`producer-prepare-app`,
`producer-input-comparison`,
`preparation-input-read`),
the `rolldown.bootstrap.config.ts` and `rolldown.sealed.config.ts` builds,
the `runtime:seal` and `bootstrap:seal` tasks and `producer-input-verification.md`.
The families:
archive naming (`archive-naming-*`,
`archive-blame`,
`archive-diff-hunks`,
`archive-git-*`,
`archive-line-map`,
`archive-use-corroborate`),
the `producer-input-*` container runner and `producer-prepare`,
the preparation attempt,
receipt,
occurrence,
root,
selection and definition records (`preparation-*`,
`create-preparation-attempt`,
`verify-preparation-attempt`,
`read-preparation-*`,
`build-preparation-root-inputs`,
`project-preparation-definition-relations`),
the persisted-input reader (`preparation-input-read-*`),
request capture (`capture-*`,
`preparation-capture-client`),
and the qualified block pairing layer over it (`qualify-*`,
`qualified-block-pairing-*`,
`queried-block-pairing-details`,
`replay-prepared-block-evidence`).

## Why

The evidence is the import graph,
measured on 2026-09-16 with a closure walk from `corpus-pass.ts` and from every build entry that existed at the
2026-09-09 handoff:
of the 282 source files added since `40aba2fdb`,
41 are reached by the pass and stay;
50 production files are reached by some pre-existing entry and stay;
144 production files were reached by none,
only by `index.ts` and their own tests
(one of them,
`fidelity-reference-barrel.ts`,
stays because the reviewed fidelity references it exports are read by the judge fidelity probe).
No page was produced by any of them.
The sessions that built them ran no pass after `Mio12` (2026-09-10),
under a self-imposed "no paid model invocation" constraint that the owner never gave;
the owner's standing instruction is the opposite ("Run now",
"Do not 'play it safe'").

## What stays

The pass-path work of the same sessions,
each verified on a Mio page:
classes twenty-five,
twenty-seven,
twenty-eight,
twenty-nine and thirty,
the footnote relabel family,
source-only breaks and the source display,
the naturalness quorum,
the name-form policy,
the block-pairing protocol and question key,
archive block selection evidence,
the rendered-break prompt,
the panel stage,
the reviewed fidelity references and the V4.1 Flash judge seat.

## What was not removed

171 stopped `podman` containers (`preparation-owner-*`,
`comparison-*`,
image `monochromatic/terminal`) and the evidence directories under
`package/module/translation-repair/node_modules/.monochromatic/`
(`comparison-evidence`,
`preparation-input-reader`,
`preparation-root-order`,
`preparation-root-order-retention`):
the session's permission mode refused the deletion.
Both are safe to delete;
nothing reads them.

## Rejected

Reverting the branch to `40aba2fdb` and replaying the pass-path commits:
615 commits touch the package and the docs since the handoff,
and the main merge of 2026-09-14 sits among them;
a measured removal keeps every verified fix and the merge.
