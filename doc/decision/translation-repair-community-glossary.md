# The community's words: a corpus glossary, and the archive's rendering weighed on every slate

Decided by the owner on 2026-09-09 ("Corpus glossary file" and "Archive rendering as candidate",
both),
asked with three options after two pages shipped the community's words wrong where the archive had them right:
自切 as `self-harmed by cutting` and `After she began cutting herself`,
where the archive has `attempted self-surgery`;
超天酱 as `Choco-chan` and `Chōten-chan`,
where the archive has `KAngel` of *Needy Streamer Overload*.

## The rule

- `community-glossary.ts`,
    beside the corpus pin (`corpus-source.ts`),
    lists each term,
    the renderings the community accepts (the archive's first) and one line of why.
    Seeded with the two;
    the owner curates it;
    a term the archive got wrong is not entered.
- Terms present in an entry's source are rendered into the entry's identity context under `COMMUNITY TERMS`,
    so every sheet that carries the declared names carries them:
    critics,
    editors,
    refiners,
    translators,
    consolidation producers and every judge.
    The preparation identity hashes the context,
    so a glossary change re-keys the slices that see the term.
- On the sheets where judges compare candidates
    (the select slate,
    the translate slate,
    the lane contest and the consolidation gate),
    a deterministic `COMMUNITY RENDERINGS` block names each candidate whose text lacks every accepted rendering
    where the source carries the term:
    evidence to weigh,
    not a verdict.
    The archive's rendering is the one the glossary records,
    which is how it joins the slate.
    No candidate is barred by the block:
    renderings inflect,
    and the judges decide.

## Options rejected

- Leave it:
    nothing to build,
    and the community's own words ship wrong on a memorial.
- The archive as a whole-text candidate:
    it already sits on every slate that has it
    (the translate incumbent,
    the lane contest,
    the consolidation standing)
    and was chosen against twice for reasons that were not the term;
    the term-level evidence is what was missing.

## What landed

- `community-glossary.ts`:
    `COMMUNITY_GLOSSARY` seeded with the two terms,
    `communityTermsIn`,
    `communityTermLines` for the identity context,
    `communityRenderingDepartures` and `communityRenderingsBlock` for the sheets;
    exported through `sheet-barrel.ts`.
- `document-preparation.ts` appends the `COMMUNITY TERMS` lines to the identity context after the entry notes,
    so the preparation identity re-keys the entries that carry a term and no other.
- `candidate-select-wire.ts` takes `sourceText` and names departures by candidate number after the candidates;
    `candidate-select.ts` threads it,
    and the translate judge,
    both editor selections,
    the refiner selection and the archive-block review pass their original.
    The editor selections' task and criteria moved to `editor-selection-sheet.ts` at the line cap.
- `lane-contest-wire.ts` and `consolidate-gate-wire.ts` add the block after the passages,
    over the archive rendering and the candidates.
- Guards shown to fail on the build before the change:
    `lane-contest-wire.unit.test.ts`,
    `consolidate-gate-wire.unit.test.ts`,
    `document-preparation.unit.test.ts`;
    with them `candidate-select-wire.unit.test.ts` and `community-glossary.unit.test.ts`.

## Addendum 2026-09-27: an organization's proper name keeps its own form

Asked by the whole-package audit:
mikaela_khara names her registered company 天津小药娘网络科技工作室
(「…就以小药娘做字号」 and 这是小药娘网络科技的雏形),
the archive writes "XiaoYaoNiang(XYN)"
and its translator's note gives the registered English name "XYN (Tianjin) Technology".
The class one hundred nineteen floor refused the pinyin there too,
so the archive and every mikaela run that wrote the name were refused.
The owner answered:
"Allow it,
because it's the proper name of an org."

- An entry may list the source contexts in which its term stands inside an organization's proper name
    (`properNameContexts` on `CommunityTerm`);
    an occurrence inside one is not the term,
    so the floor,
    the sheet lines and the departures read the passage as if it did not carry it there.
- 药娘 lists 小药娘网络科技 and 以小药娘做字号.
    The ruling of 2026-09-24 is unchanged everywhere else:
    the word is refused in Han and in pinyin even where the archive keeps it.

## Addendum 2026-09-28: what the glossary enters, and what it refuses

The whole-package audit (ledger C6 in `package/translation-repair/doc/audit-ledger.md`)
found two statements here that later practice left behind.

- "A term the archive got wrong is not entered" stopped holding once runs shipped words the archive had wrong too:
    炸柜 (the archive's "tried coming out"),
    and 阿洛娜 and 亚托莉 (the archive's "Alona and Atori") entered by the classes that found them.
    An entry goes in wherever a run shipped the word wrong,
    and its renderings lead with the archive's only where the archive renders the word well.
- "No candidate is barred by the block" still holds of the `COMMUNITY RENDERINGS` block,
    but since the owner's ruling of 2026-09-24 (class one hundred nineteen)
    a floor refuses a candidate that keeps a term in Han or writes a form its entry refuses,
    so an entry can bar a candidate.
    No listed rendering is ever required.
- An entry may also list longer words that write its characters without being the term
    (`enclosingWords`:
  自切 inside 各自切),
    read as absent the way a proper name is.
