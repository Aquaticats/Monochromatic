# Translation repair handover 2026-09-06: checkpoints of 2026-09-24 and 2026-09-29

The two checkpoints that stood between the takeover of 2026-09-16 and the handover to a trial model,
one saying where the pipeline stood at class one hundred ten with eight seats and two wet providers,
the other recording classes one hundred eleven to one hundred eighty-seven,
the whole-package audit and the coverage batches of its T8 entry.

Part of [Translation repair handover snapshot: 2026-09-06](translation-repair-handover-2026-09-06.md),
which lists every file of this record.

## 2026-09-29: the whole-package audit, and no pass since 2026-09-28

The checkpoint of 2026-09-24 (next heading) left the pipeline at class one hundred ten;
this one says what changed after it.

- CLASSES.
  Classes one hundred eleven to one hundred eighty-seven were found between 2026-09-24 and 2026-09-28,
  most on read pages;
  the pass log records each under the section that found it,
  with what was done about it.
  The newest,
  class one hundred eighty-seven:
  a patch left a quote nested two levels deep where neither the original nor the archive nests,
  and patch application now clamps each replacement line to the depth its context allows
  (`58287ebe3`,
  `05a18ed02`;
  guards `d661bb51f`,
  `9e4d7dafb`).
- THE AUDIT.
  On 2026-09-27 the owner asked for an audit of the whole package
  ("Mistakes made,
  ever,
  for this pkg,
  not just today.")
  and allowed the next launch to wait for its fixes.
  Every finding with its measurement,
  fix and guard,
  and every mistake made while fixing,
  is in the package's
  [audit ledger](../../package/translation-repair/doc/audit-ledger.md);
  the rule that prevents each family of mistakes is in
  [`mistake-prevention.md`](../../package/translation-repair/doc/mistake-prevention.md).
  Open work is every ledger entry whose status reads open or recurring;
  L6 waits for the next launch by its own entry.
- THE LAST PASS.
  `TianqiChen66621` on `.frozen-dist-9a3f28b30` settled on 2026-09-28 in 3,169,499 ms
  and was read the same day
  (pass log,
  "TianqiChen66621 read,
  2026-09-28").
  No pass has launched since.
- ROSTER.
  The checker seats were measured on 2026-09-27 and Mimo v2.6 Flash joined them;
  [`roster-changes.md`](../../package/translation-repair/doc/roster-changes.md)
  has every change with its date.
- NEXT.
  The open ledger entries,
  then the prevention doc's "Before a run launches",
  then `TianqiChen666` again on the audit build,
  so that class one hundred eighty-seven and every fix since `9a3f28b30` are read live.
  Of the open entries,
  B21 is the code-reading pass over the recurring families:
  its first family,
  text read by UTF-16 unit,
  closed on 2026-09-29 as B22,
  with `fixed-length-cuts.unit.test.ts` and the Latin-twin cases in `canadian-forms.unit.test.ts` among its guards.
  The second,
  words found inside longer words (class one hundred sixty-three),
  closed on 2026-09-29 as B23:
  every site its census classed as a finding reads its word,
  name or address as one,
  the last being the unwrapped-link floor (`28a4ceb63`)
  and the neutral pronoun floor on a new shared token reading (`2c0cc08d8`),
  and no cache version moves.
  The third,
  straight against curly quotes,
  closed on 2026-09-29 as B24:
  a model's quote against its document now reads through the evidence fold,
  two renderings through the typography fold
  (the slate collapse,
  the page-title lexicon,
  the Latin title floor,
  the title-reference pass),
  guillemets are refused by a floor,
  and each change rides inside its cache version;
  the attestation also reads each reference quote in one reference line (B25).
  The fourth,
  trimmed text compared with raw text,
  closed on 2026-09-29 as B26:
  every site that ships,
  demotes,
  collapses or offers a wording now asks `sameWording`,
  which reads a paragraph's soft line breaks as the site renders them,
  so a lane text that is the archive in all but layout keeps the archive's bytes
  (263 such lane texts in the stored runs).
  The fifth,
  quorum denominators counting unreachable seats,
  closed on 2026-09-29 as B27:
  the archive block review now holds its anchored voices to the reachable quorum its gather closed on,
  where it had kept the whole bench's,
  a defect no stored run shows firing.
  The sixth,
  sheets missing a block their role needs,
  closed on 2026-09-29 as B28:
  the archive block review and its slate,
  coverage,
  the page-title lexicon and the refine slates now carry the declared names their house rules point to
  (the review and its slate the cited references too),
  two sheets gained the narrative bound beside the apparatus kinds,
  and `rendered-sheets-context.unit.test.ts` fails on a rendered sheet naming a block it does not carry,
  now that the fixtures render every sheet with the context production gives it.
  The seventh,
  silent fallbacks,
  closed on 2026-09-29 as B29:
  the final polish refuses the slice's disputed wordings,
  reference attestation needs two voices,
  a checker bench holds none of the chunk's writers when it is re-read at the stage,
  two writers' sheets state their candidates' break counts,
  and `floor-inputs-stated.unit.test.ts` fails on a production call leaving a floor input or gate flag to its default unnamed;
  three names exported twice are renamed,
  and `exported-function-names.unit.test.ts` keeps them apart.
  The rendering audit's auditors now read the cited references the producers had
  (`4b02cff72`),
  under a rule worded for the audit's own categories.
  B29 leaves one gap open:
  a Han character beyond the first plane that pinyin-pro cannot read passes through the handle reading as written,
  which no pinned original carries.
  D33 (references by position) is closed:
  every reference in the package and the living docs names its target,
  and `position-references.unit.test.ts` guards both (`9426d91be`).
  Both gaps found on the way are closed:
  `code-points.ts` and `cased-letters.ts` have unit tests of their own,
  which found and fixed two lone-surrogate faults
  (`f6e93ed5f`,
  ledger B22),
  and the cache version check is the read-only mise task `cache-account-audit`,
  which the checklist's cache step now runs
  (ledger M28).
  T8 (code no unit test runs) is the open entry the launch waits for,
  an order taken under the owner's standing directive to prefer the quality of the end result,
  which the owner may veto to launch first.
  The package task `mise run coverage-census` measures it
  by block coverage mapped to source lines.
  Its whole-suite census at `e22373347` found 72 functions of library source never called,
  but recorded each cold stretch under the source of its first character alone,
  leaving out every module a stretch ran on into (ledger M67,
  fixed in `25dc7e9f6`),
  so its stretch and line counts are superseded,
  and a whole-suite census of the new format is the baseline each batch of tests is read against;
  the ledger entry has the method,
  the classes and what closes it.
  Reading T8's cold stretches,
  B30 found 57 functions only tests reached,
  which the dead-functions guard had passed because it counted a test or a re-export as a use;
  it closed on 2026-09-29
  (`eda99b687` to `32e07afb7`):
  the guard counts reach from production code,
  and each function was removed,
  or moved to a `.test-fixture.ts` file.
  B31,
  the 25 top-level values the same reach left unread,
  closed the same day
  (`e91ce423b` to `e28dd54f2`):
  the guard,
  now `src/dead-code.unit.test.ts`,
  tracks classes and values too,
  the alignment's gap penalty is declared once,
  where it is documented,
  and the walk reads it,
  and the role-named test seats left the package build.
  B32 closed the same day
  (`40add2b70`,
  `c077cc881`):
  no configured check reports an unused import
  (the package's `mistake-prevention.md` names the issue),
  three of the audit's own removals had left ten,
  and `src/unused-imports.unit.test.ts` now fails on any.
  The whole-suite census at `6e631988b` (`census-wNMuym`,
  1,387 PASS lines) is the baseline T8's batches resume against:
  library source holds 1,104 stretches in 423 files,
  with 29 functions never called.
  On 2026-09-30 (UTC) the first batch tested the census's own placement (`5e27208c7`),
  and the census now names each claimed source its baseline holds no stretch in (`297c72fd5`),
  since a source the baseline never loaded had read as all zeros.
  The second to fourth batches took the naturalness review,
  polish,
  and contest,
  lane,
  comparison and consolidation readers
  (`a564eccc7`,
  `09b76e1b4`,
  `74387b886`).
  B33 reworded six artifact refusal reasons that did not read as what the reader expected
  (`74387b886`,
  `ebaa97908`),
  and B34 stopped marked refusals from carrying a caught error's text,
  two of which had quoted a stored digest or identity
  (`7360aae58`,
  `a1e4accbd`);
  `message-names-only.unit.test.ts` now checks every such construction.
  The census reading also stopped counting stretches of sources edited since the baseline as run,
  since it matches by line
  (`4ab6436bf`,
  `d27a89dd0`),
  and refuses a baseline taken with uncommitted changes.
  The fifth batch took the comparison,
  coherence,
  digest and identity modules (`fc9e80ca8`).
  Every census through `census-VdHVJb` measured a compressed build,
  which folds unreached guards into expressions coverage gives no range;
  the coverage build is now unminified and the census refuses a minified one
  (ledger M79,
  `aeab4cfad`).
  The sixth batch read against the whole suite at `aeab4cfad` (`census-xBFoZt`,
  1,396 PASS lines)
  and took the translate modules,
  closing all 73 of their stretches in 28 files
  (`62ad2c084` to `0fcb75c7b`)
  and fixing B35 to B42 on the way;
  B43,
  what the translate stage should do where the floor can compare nothing,
  is fixed (it settles such a slice before any call),
  and so is B45,
  the consolidation's writers asked there;
  neither reached a pinned slice.
  The seventh batch read against the whole suite at `d780418f7`,
  B45's last commit (`census-v7Dz0r`,
  1,417 passes),
  and took the repair modules,
  closing all 46 of their stretches in 17 files
  (`da99d5b69` to `06f231981`).
  The eighth batch read against the whole suite at `06f231981` (`census-dUOMFS`,
  1,421 passes)
  and took the consolidation modules,
  closing all 55 of their stretches in 17 files
  (`62a745069` to `0fba3dea4`;
  ledger B46 to B52).
  The ninth batch read against the whole suite at `0fba3dea4` (`census-SfAXnI`,
  1,429 passes)
  and took the corpus-run passes,
  closing all 31 of their stretches in 16 files
  (`b7cab3bbb` to `51b888ac2`;
  ledger B53 to B55).
  The tenth batch read against the whole suite at `51b888ac2` (`census-VW2tHL`,
  1,437 passes)
  and took the corpus-run title modules,
  closing all 26 of their stretches in 4 files
  (`7e3772ba8` to `95ca288cd`;
  ledger B57 to B60).
  The eleventh batch read against the whole suite at `95ca288cd` (`census-BF4g7o`,
  1,445 passes)
  and took the alignment modules,
  closing all 26 of their stretches in 5 files
  (`c5fd0b03e` to `791bc45fe`);
  reading its census found ledger B61 and B62,
  gaps in the census reading itself,
  both fixed since (`48c580d16` to `4304d167d`):
  the reading names each stretch cold since its baseline,
  and asks git about every file of the work tree.
  The twelfth batch read against the whole suite at `4304d167d` (`census-7D4Vp9`,
  1,447 passes)
  and took the corpus-run artifact modules,
  closing all 26 of their stretches in 8 files
  (`21d3654d5` to `99eed862e`;
  ledger B63 and B64).
  Ledger B65,
  run-directory readers taking entries their writer never wrote,
  was fixed before the thirteenth batch (`e987110a1` to `6583c32c4`),
  with a source scan that holds every listing to `directory-listing.ts`.
  The thirteenth batch read against the whole suite at `6583c32c4` (`census-EiesJ9`,
  1,464 passes)
  and took the lane modules,
  closing all 21 of their stretches in 7 files
  (`da74173a1` to `de9ced3f0`;
  ledger B66,
  a count printed before a fixed plural,
  whose wider family is queued as one change).
  The fourteenth batch read against the whole suite at `de9ced3f0` (`census-lPGpK8`,
  1,465 passes)
  and took the insertion modules of `corpus-run`,
  closing all 21 of their stretches in 7 files
  (`1af921d31` to `bc15e9a28`;
  ledger B67,
  a container half cut at a text search for its tag rather than at the tag).
  Its census at `bc15e9a28` (`census-o4RZ7w`,
  1,466 passes)
  leaves library source at 805 stretches over 1,636 lines in 293 files,
  with 17 functions never called.
  Ledger B68,
  the same file counting a container's blocks by blank-line split rather than off the parse,
  is fixed too (`8ec62ce62` and `19eb118ab`).
  The fifteenth batch read against the whole suite at `19eb118ab` (`census-VmhQQH`,
  1,467 passes)
  and took the prompt modules,
  closing all 20 of their stretches in 2 files
  (`2d3b02c45` to `f7838ffb3`;
  ledger B69,
  prompt payload store refusals that named nothing that refused).
  Its census at `f7838ffb3` (`census-OGAQiJ`,
  1,468 passes)
  leaves library source at 785 stretches over 1,593 lines in 291 files,
  with 17 functions never called,
  and was the sixteenth batch's baseline.
  That batch took the pairing modules,
  closing all 19 of their stretches in 9 files
  (`b0924bd4d` and `c2974bea9`;
  no entry of its own,
  and ledger M85 to M87 for the three slips its cases corrected before their commit).
  Its census at `c2974bea9` (`census-PgyHZ8`,
  1,470 passes)
  leaves library source at 766 stretches over 1,554 lines in 282 files,
  with 16 functions never called.
  A fold that copied its answers so far at every step turned up in that batch,
  and reading the package's folds for the shape found ledger B70
  (`42d7a0f08` to `66b77b040`):
  seven folds and loops copying a growing list,
  and four splices writing text one edit at a time,
  one of which cut a title quoted inside a quoted title one unit late.
  The lists now grow by appending,
  the splices write in one pass,
  and `fold-copies.unit.test.ts` fails on a new fold that copies.
  Its census at `66b77b040` (`census-DapwaX`,
  1,472 passes)
  leaves library source at 765 stretches over 1,547 lines in 282 files,
  with 15 functions never called,
  and was the seventeenth batch's baseline.
  That batch took the slice modules,
  closing all 18 of their stretches in 7 files
  (`774710cb0` to `3bc823c13`;
  ledger B71,
  a slice-cost reader that read an empty or misspelt count as a number).
  Its census at `3bc823c13` (`census-vi1FHB`,
  1,472 passes)
  leaves library source at 747 stretches over 1,435 lines in 275 files,
  with 15 functions never called.
  Ledger B72 followed
  (`db01484f7` to `3429c8229`):
  the italic-title pass now reads the archive's italics and the page's off the parse,
  leaving a title inside italics quoted,
  with no current output changed.
  Its census at `3429c8229` (`census-apleN5`,
  1,475 passes)
  leaves library source at 746 stretches over 1,432 lines in 274 files,
  with 15 functions never called.
  Ledger B73 followed
  (`51dc87e82` to `93a0d06b1`):
  the package's number reads from text,
  in its own records,
  an operator's dials and flags,
  provider listings and log stamps,
  read spellings no writer here writes,
  among them a log stamp without its zone read as local time;
  each now reads through a rule held to its writer's spelling,
  and `number-reads.unit.test.ts` fails on a new read until it names its rule.
  Ledger B74 followed
  (`9e01f2650` to `81ec51782`):
  five loops that grouped or queued by key,
  the cap census's queue among them,
  set a map entry to a copy of what it held at every repeat;
  they append in place now,
  and `fold-copies.unit.test.ts` fails on a new one.
  Ledger B75 followed
  (`6035e9b03` to `5a3e38037`):
  every runner found its own flags as exact tokens,
  so `--cap=0`,
  a mistyped flag,
  a second `--only` or an argument to a runner that reads none
  read as not written;
  each runner now declares its line in `corpus-run/command-lines.ts`,
  `reportingRefusals` reads the whole line against it before the runner starts,
  and the task descriptions and the living docs' invocations are held to the declarations.
  Ledger B76 followed
  (`28cbadd6a` to `4f9b79155`):
  an `--only` naming an entry a runner cannot reach was dropped without a word,
  so the runner ran over nothing or over the rest;
  it now refuses the line,
  naming every stray.
  Ledger B77 followed
  (`f0322ce6f` to `2293b0ded`):
  plain-object tables looked up by text from outside the package (found in B73)
  answered the names every object inherits,
  so a stream delta,
  an asset name,
  a stored status or disposition,
  or an entry id spelled `constructor` or `__proto__` read as a prototype value;
  every table looked up by text is now a map,
  and `text-keyed-tables.unit.test.ts` fails on a new one.
  Ledger B78 followed
  (`a3f1f2c55` to `e2b0962fd`):
  durations,
  holds,
  paces and budgets were differences of system-clock readings (found in the seventeenth batch),
  so setting the clock moved them by the hour it was set;
  every difference inside one process now reads `monotonicMs`,
  and `wall-clock-reads.unit.test.ts` fails on a new read of the system clock as a number.
  Its red cases' stub first replaced `Date.now` for the whole test process,
  and a guard-off hung a file instead of failing it;
  the stub now holds the clock for its own case only (ledger M100).
  The eighteenth batch's baseline is `census-dKGOhZ` at `36e3f36cc`
  (730 library stretches over 1,404 lines in 274 files),
  that batch taking the archive modules
  (`archive`,
  18 stretches over 36 lines in 9 files,
  the most cold lines of the two clusters tied at 18)
  and the two usage-line arms of `corpus-run/command-line.ts` that went cold with B75.
  Ledger B79 followed the census
  (`b9edc1542` to `cf17f1dd0`):
  13 test files wrote an environment variable,
  a `console` method or the working directory in suites that ran their cases at once;
  each such suite now runs one case at a time,
  and `global-writes-sequenced.unit.test.ts` fails on a write outside one.
  The eighteenth batch took the archive modules and the two usage-line arms
  (`9a362ba09` to `f9babaec4`)
  and found four defects behind their cold lines:
  the footnote relabel's ambiguity detail named a label by its folded key without its side,
  the revision footnote floor passed a block the archive did not carry,
  a definition move that would change the parse kept a sound rename back with it,
  and the revision shape refusals listed shapes where a count belongs.
  Its census,
  `census-JXU9Js` at `67f85061a`
  (711 library stretches over 1,367 lines in 265 files),
  names the nineteenth batch:
  `corpus-run/run`
  (18 stretches over 18 lines in 2 files).
  That batch reads against the census taken once the two findings the eighteenth left open close,
  and both have closed.
  B80:
  archive revisions were checked one at a time though they compose,
  and each block is now reviewed in the page the revisions already applied leave
  (`308bbe206` to `ccc2df6f2`).
  B81:
  the editorial-apparatus check vouched for a block on one of its lines,
  and now every line a reader sees must be apparatus;
  over the pinned archives it accepts the same 40 blocks as before
  (`d6d72cbcf` to `3788e8223`).
  The nineteenth batch took `corpus-run/run` against `census-EJl6yz`
  and closed B82:
  the run timing readers read less than their writers write
  (a round that never reached quorum,
  a round line short of its fields,
  a completion line without its outcome,
  calls spanning no time),
  and each is now read or refused by name
  (`71f7720a9` to `37f637675`).
  Its census,
  `census-IqLEk4` at `8175dc866`
  (693 library stretches in 263 files),
  names the twentieth batch:
  `corpus-run/heading`
  (17 stretches over 25 lines in 3 files).
  That batch closed B83 and B84:
  page-assembly passes paired headings by position without checking the count,
  so a page slice that dropped or gained a heading could ship an untouched section as empty text,
  give a section the wrong number,
  or write a signer's name over another section's title
  (`66927af39` to `c6464ab59`).
  Its census,
  `census-STKkse` at `4c0039a57`
  (666 library stretches in 260 files),
  names the twenty-first batch:
  `assembly`
  (16 stretches over 27 lines in 8 files),
  taken against `census-zkJwfY` at `b9da490fb`.
  That batch closed B85 and B86:
  the assembly guard's advancing withdrawal compared refusal offsets on two different pages,
  so a whole replacement that only changed the length of the text before a break read as moving it,
  and the reader of those offsets placed a refusal at the start of what the parser left behind,
  so a page whose remaining break was an element left open read as stopping at its first character;
  either way the guard could withdraw every replacement
  (`9c007f857` to `0e7353948`).
  Its census,
  `census-dnr81C` at `9244d67d3`
  (650 library stretches in 251 files),
  names the twenty-second batch:
  `anthropic`
  (16 stretches over 22 lines in 2 files),
  taken against `census-lxT62v` at `73bd4426c`.
  That batch closed B87 to B91 and B93 to B94 on the Charm Hyper reader:
  a body cut inside its last frame passed the retry ladder as whole,
  an error event read as a cut stream,
  the prompt count left out the cache,
  a missing answer fragment was dropped without a word,
  a refused body named the wrong wire's contract,
  the delta scanner filed or passed over frames it does not read,
  and a count the stream never sent was reported as 0
  (`57ffcf2dd` to `a41e6218b`);
  B92,
  `isJsonRecord` passing arrays,
  is queued as its own family.
  Its census,
  `census-Hq0z31` at `a41e6218b`
  (633 library stretches in 249 files),
  names the twenty-third batch:
  `front`
  (15 stretches over 20 lines in 3 files).
  The library batches go on one triage cluster each,
  each throw and nullish fallback read for reachability before a case is written,
  and each batch ending with a whole-suite census at its committed head as the next baseline;
  the ledger's T8 entry has the triage counts.
  Every commit touching the package's source,
  docs,
  README or task file also runs `mise run source-scans`,
  the package-wide scans a new file can fail,
  four of which read the docs (ledger M59).

## 2026-09-24, 04:30 UTC: one hundred ten classes, eight seats, two wet providers

The snapshot of 2026-09-16 (next heading) describes the takeover;
everything after it is in the pass log
([`translation-repair-openrouter-2026-09-03.md`](../planning/translation-repair-openrouter-2026-09-03.md),
one heading per run or read,
oldest first up to the Mio23 launch of 2026-09-16 and newest first after it)
and in step 1 of "What to do next".
This checkpoint says where those eight days left the pipeline,
so a reader does not have to walk eighty headings to find out.

- CLASSES.
  Classes twenty-six to one hundred ten were found on read pages and fixed,
  each with a guard shown red first,
  each recorded in the pass log under the read that found it,
  the decision addenda in
  [`translation-repair-absence-verdict.md`](../decision/translation-repair-absence-verdict.md)
  (the admission of source-only passages:
  halves,
  definitions,
  the tail,
  the container deficit,
  and on 2026-09-24 the carried fold)
  and
  [`translation-repair-ineligible-standing.md`](../decision/translation-repair-ineligible-standing.md)
  (nine addenda:
  what ships when the archive's rendering cannot),
  and the slice rules in
  [`slice-context.md`](../../package/translation-repair/doc/slice-context.md).
  The latest,
  class one hundred ten (`457765221`):
  a source-only passage the roster found carried,
  whose evidence sat inside the neighbouring paired slice's archive span,
  is folded into that carrier's source at the admission so both lanes write it,
  instead of the publish guard stopping the entry when the translate lane dropped it.
  Not exercised live yet;
  `mikaela13` runs on it.
- ENTRIES READ.
  Every entry in the read set has shipped SETTLED at least once on a recent build:
  `XingZ60` (the longest,
  119 slices,
  43 to 76 minutes on Bedrock and OpenRouter,
  8.6 to 9.1 USD),
  `hulicaijia` (27 to 37 minutes,
  about 4 USD),
  `mikaela_khara` (11 to 18 minutes,
  about 1.1 USD),
  `shi_Yumiaoya` (6 to 9 minutes,
  about 0.5 USD),
  `zheermao101`,
  `CuspariaKLSY`,
  `yingying` (4 to 20 minutes,
  0.1 to 0.5 USD).
  The seven-hour ceiling has not been reached since XingZ607 (2026-09-19).
  A run is read on the day it ends and the next entry launched the same hour;
  since 2026-09-23 a read entry with no class is followed by the next entry in the rotation,
  and a class is fixed before the next launch.
- ROSTER.
  Eight seats since `hf:openai/gpt-oss-120b` left every role on 2026-09-24 at the owner's instruction
  ([`roster-changes.md`](../../package/translation-repair/doc/roster-changes.md),
  the blocklist decision):
  `deepseek-v4.1-flash`,
  `hf:zai-org/GLM-5.3-Flash`,
  `hf:Qwen/Qwen3.8-27B`,
  `hf:moonshotai/Kimi-K3`,
  `minimax-m3`,
  `glm-5.3`,
  `gemma-4-26b-a4b-it`,
  `google.gemma-4-e2b`,
  with `inception/mercury-2.5` OpenRouter-only in the writing and consolidation seats and
  `typesafe/jev-1.13` as a select judge through the OpenRouter decisions endpoint
  (78 of 78 usable on mikaela12).
  The JUDGE SEATS line of a launch on Bedrock and OpenRouter alone reads
  `wide=6 select=7 late=7 slate=7 checkers=3 translators=7 readers=5 writers=8 roster=8`.
  Qwen and glm-5.3 are dark whenever Synthetic and Hyper are dry (by design;
  their seats have no other provider).
- PROVIDERS AND MONEY.
  Bedrock 47.86 USD of the 200 the owner will never top up
  (152 spent since 2026-09-07,
  about 4.7 USD on a XingZ60 pass,
  0.3 on a mikaela pass);
  OpenRouter 94.68 USD of the final 273.99 top-up of 2026-09-09.
  Synthetic runs out its weekly allowance most days
  (dry since 02:56 UTC on 2026-09-24)
  and returns as usage ages out;
  Hyper is dry and never recharged.
  A dry-out inside a pass is handled (classes one hundred three and nine);
  a launch on two providers is normal operation.
- LOGGING.
  Since 2026-09-24 every critic claim is logged with its author at the critic stage,
  every adjudicated issue with its proposers,
  and the artifact carries the proposers inline (owner:
  "we're not logging enough then");
  every slate abstention is logged with its reason (2026-09-20);
  every gate ballot with its reason (2026-09-18).
  A reading that names a model's error can now cite the line.
  Still missing:
  the coverage admission prints `coverage=carried` without the evidence regions,
  so a carried fold that stands aside cannot be judged from the log alone;
  to add after `mikaela13` exits.
- OPEN,
  NOT BUILT.
  The placeholder block "**Come back later!**" on XingZ60:
  the archive block review removes it on some runs and keeps or rewrites it on others
  (six treatments in seven runs);
  whether it is apparatus the page owes is an owner question never put.
  The coined 螐 ships in two treatments on one XingZ60 page.
  The repair lane's own floor let a three-line closing quote into the contest on shi_Yumiaoya16
  (class one hundred two closed the consolidation side only).
  An empty standing with no valid lane text still fails at publish rather than at once
  (never reached).
  The seven reading steps and the three checks are unchanged;
  every read since 2026-09-19 also counts unendorsed,
  gate-kept and slate-declined standings,
  abstention lines and the Jev seat.
