# Why the semantic wrap starts its linter synchronously

Written 2026-10-13,
when the wrap moved from the TypeScript Markdown linter package to the Rust `monochromatic-lint`.

## What is suppressed

`startProgram` in `src/semantic-wrap-binary.ts` calls `spawnSync` from `node:child_process`.
The repository's oxlint configuration bans that call with `no-restricted-syntax/no-sync`
(`package/config/oxlint/src/rule/restriction.ts`,
set to `error`),
so the function carries a scoped `oxlint-disable` and `oxlint-enable` pair around its TSDoc and body,
and the disable comment cites this file.

## Why the rule's own advice does not fit

The rule says to use the asynchronous API.
`wrapReplacementText` is a plain function that returns a string,
and its callers use it inside functions that are not asynchronous:
`wordingKey` (`package/translation-repair/translate-floor/src/wording-key.ts`) builds a comparison key from it,
`wrapTranslateRecords`,
`wrapRepairOutcomes`,
`wrapConsolidation` and `wrapConsolidationProposals`
map it over records inside callbacks,
and the unit tests that pin the wrap assert on its return value directly.
There are seven call sites in six source files,
and the functions around them are called from many more.
Making the start asynchronous would make each of those functions asynchronous,
and every caller of those after them,
which restructures the package around one cosmetic step.
The task that carried this change set a limit for restructuring callers:
only a measured cost above two seconds for the largest page of the corpus would justify it.

## What it costs

Measured over the 92 English pages of the corpus clone
(each handed over as its heading-bounded chunks,
which is how the callers hand text over),
a start of the program took a median of 5.4 milliseconds and at most 8.3 milliseconds,
for 285 chunk calls in all.
The slowest page took 80 milliseconds in total across its 14 chunks.
The in-process wrap it replaced took 564 seconds on one passage of 1.54 million characters,
where the program took 0.2 seconds,
so the synchronous start is also the faster of the two on large input.

## Why configuration does not handle it

The only override in `package/config/oxlint/src/overrides.ts` that turns the rule off is the one for test and benchmark files.
The rule itself takes no option that allows a single call:
`package/oxlint-plugin/no-restricted-syntax/src/rule/no-sync.ts` declares none.
Turning the rule off for this package would loosen a lint rule,
which needs the owner's approval,
and a scoped suppression on the one function that needs it asks for less.

## What would remove the suppression

Making the callers asynchronous removes the need,
at the cost named in the section on why the rule's own advice does not fit.
Standard input of the program takes one source per start,
and linting many sources in one start means files on disk,
which a synchronous caller holding one passage cannot batch.
