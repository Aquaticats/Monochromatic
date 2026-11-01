# Translation repair history, first handover document: corpus facts and the seed bank of real errors

What was verified about the memorial corpus:
its layout and licence,
the pinned commit,
MDX parsing limits,
the editing guides,
and the real errors,
quoted-speech cases and archive transcription errors that the design had to survive.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Corpus facts (verified)

- Memorial corpus:
  `one-among-us/data`,
  `people/<id>/` entries with `info.yml`,
  `page.md` (zh source,
  YAML front matter,
  `##` section headings),
  `page.en.md` (translation under repair),
  `page.zh_hant.md` (script conversion,
  out of scope).
- The data repo is `UNLICENSED` (its `package.json`),
  explicitly all rights reserved:
  corpus content must never be committed to this repository.
- Fixture strategy (task 6,
  done):
  the user cloned the repo to `~/one-among-us/data`;
  `corpus-source.ts` reads it via `git show` at pinned `CORPUS_COMMIT_SHA`
  (`a41fc607ea5a70d8a7625cc67d5ed8c444f53379`,
  upstream `main` on 2026-07-16;
  92 zh/en page pairs,
  footnoted entries include Huasheng and DarlinChit).
  Blob reads use byte-exact `execFile` capture (nano-spawn strips the final newline);
  git resolves through `resolveGit` from `@monochromatic-dev/git-policy-cli/ts`
  (newly exported),
  because PATH exposes the policy shim
  (`node_modules/.bin/git` rejects bulk staging even in throwaway repos).
  Boundary-verified against the real clone:
  listing plus two entries parsed
  cleanly through `parseDocument` with resolved footnote graphs.
- Pages are MDX:
  upstream `scripts/mdx.ts` compiles with `@mdx-js/mdx`
  (Vue pragma,
  `remarkMath` only,
  no `remark-gfm`).
  `[^1]` renders quasi-literally on the live site;
  emitted repairs must preserve the exact textual footnote convention byte-for-byte.
- NOT every entry parses as MDX:
  of twelve sampled zh/en pairs,
  four failed
  `parseDocument` (`interrgned`,
  `windward0032`,
  `XingZ60`,
  `mikaela_khara`),
  at least one on an HTML comment (`<!--`,
  illegal in MDX) at body start.
  Open question how upstream renders those;
  the pipeline needs a skip-or-
  preprocess decision before corpus-wide runs.
  Parse-clean large pairs besides Huasheng/DarlinChit:
  `shihai4h`,
  `aiyysk`,
  `hulicaijia`,
  `NIGHT81473140`,
  `Xu_Yushu`,
  `zhangyubaka`.
- Deletion seeds can break the seeded MDX parse when the deleted sentence
  holds half of a paired construct (seen live:
  acorn "Unterminated string
  constant" after a seed removed the closing half of an `{'...'}` expression).
  `deriveOmissionSeeds` therefore skips delimiter-bearing sentences
  (commit `735e1b34e`),
  and the driver preflights `parseDocument` over the
  seeded text before spending quota.
- `page.en.md` files are plausibly Google-Translate seeded
  (`google-translate-api-x` plus a `translate` script upstream) with uneven human editing.
- Some memorial texts carry footnotes (`[^1]`,
  definitions `[^1]:[text](url)` link-wrapped);
  the archive class (Lu Xun,
  later) uses plain-text `〔N〕` markers.
- Editing guides are the policy files:
  `one-among-us/about-site` `content/{zh-Hans,en}/docs/memorial.md`.
  Deterministically checkable rules:
  「」/『』 quote conventions in zh,
  curly quotes in en.
  Hard content rules:
  soften suicide methods,
  never drug names or doses;
  third person;
  `ta`/`they` for unstated pronouns;
  archive links for external references.
- Real error seed bank from corpus inspection:
  fabricated year ("November 13th,
  2023" where source has no year;
  timeline implies 2019),
  meaning inversion ("never taught anything about gender" vs 接触过...相关概念),
  load-bearing omission (father's 「就像你们女孩子总希望找到一个强壮的男朋友保护自己」),
  flattened irony (心灵干洗机 -> "spiritual baptism"),
  policy-violating addition ("her committing suicide" for 她的离开 in the lxy entry).
- Quoted misgendering is content to preserve (narration she/her,
  father's 「儿子」「他」 stays);
  terminology-consistency guards need quoted-speech exemptions.
- Archive texts contain transcription errors against canonical editions
  (缘愁似棍长 for 缘愁似个长,
  春秋焚梁传 for 穀梁传,
  一九三三月三月十五日):
  `suspected-source-error` must be able to block "corrections" toward corruption.
