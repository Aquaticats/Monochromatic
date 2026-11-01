# Work titles and established vocabulary

Decided by the owner on 2026-09-02,
in answer to the question whether an archive's rendering of a work
title counts as a declared name that translators keep (raised by the Toka_ls reading,
slice 10,
where
the archive's "Life of Aiden" for 《奇妙漂流》（或称「Aiden 的奇幻漂流」） was re-rendered as "Wondrous
Voyage").
The owner's words:

> If something has an official English translation,
> use that.
> Some entries contain established vocabulary in footnotes.
> If that exists,
> use that.

## The rule

- A work named in the ORIGINAL is rendered by its official English title when one exists:
  a published
  English edition,
  an official localisation,
  or a title the work carries in English.
- Where the entry itself establishes vocabulary in a footnote,
  that vocabulary is used.
- Where neither exists,
  nothing changes:
  the title is translated like the rest of the passage and kept
  consistent across the page.
  The owner did not decide that an archive's rendering is protected on its
  own,
  so this rule does not protect it.

## How the rule is applied here (my readings, veto invited)

Three readings of the two sentences,
each mine rather than the owner's:

- Allusions count as naming.
  「Aiden 的奇幻漂流」 reads as a play on 少年 Pi 的奇幻漂流,
  whose English
  title is "Life of Pi",
  so the archive's "Life of Aiden" follows the official title of the work
  alluded to and the re-rendering at Toka_ls slice 10 was the defect.
  (The source names 《奇妙漂流》 as a
  work the person created with friends,
  and the contributor credit names 奇妙漂流 as a contributor,
  so
  the group may carry its own English name;
  the first clause covers that too.)
- Editor comments count as footnotes.
  Seventeen sources carry HTML comments,
  and yulianNyanner's are a
  glossary in all but name (波奇酱（后藤一里）：Bocchi-chan,
  结束乐队：Kessoku Band,
  亚托莉：Atri,
  "这里标题
  对应的英文词是 dysphoria"),
  so "footnotes" is read as the notes an entry carries,
  whichever syntax.
- The archive's own footnote definitions count as established English for the same notes:
  "That Time I
  Got Reincarnated as a Slime",
  "The Unbearable Lightness of Being ... translated by Michael Henry Heim"
  are renderings a translator should not re-derive.

## Evidence the rule stands on

Measured over the pinned corpus (`a41fc607ea5a70d8a7625cc67d5ed8c444f53379`,
92 entries) and the
Carena0442 fixture:

- 23 sources carry footnote definitions,
  52 definitions in all;
  21 archives carry them.
  Fifteen of the
  definitions carry Latin letters:
  10 name a term or a work (即 Transcend Lights and Eli,
  Eli,
  lama
  sabachthani in XingZ60;
  即 Google App Engine in hakureico;
  《CLANNAD》 in NIGHT81473140;
  「Blunt
  Rotation」 in gqt;
  CPTSD and DID in LCG_Akiball;
  《假面骑士 Build》 in hulicaijia;
  MtF in Futajuhuacha;
  Android slots in aiyysk;
  Twitter in mikaela_khara),
  4 are links (yingying twice,
  lxy,
  hulicaijia),
  and 1 is an attribution note (homoyamakaze).
  The other 37 are Chinese explanatory notes,
  which still
  establish how a passage is to be read.
- 17 sources carry HTML comments,
  the yulianNyanner glossary among them;
  the source credit comment
  (`条目贡献`) is one of them and is handled separately (see Mechanism).
- Archives already render some notes by official titles:
  "That Time I Got Reincarnated as a Slime",
  "The Unbearable Lightness of Being ... translated by Michael Henry Heim",
  "Gitanjali No. 60".
- Of tonight's entries,
  XIEPT2 and Carena0442 carry one source footnote each;
  Toka_ls and keyword233
  carry none,
  so the Toka_ls case is decided by the official-title clause alone.

## Mechanism

Landed as commit `12ed82cee` (house-policy bullet,
`entry-notes.ts`,
`work-title-scan.ts`,
`work-title-search.ts`,
`lookup-cache.ts`,
`work-title-lookup.ts`,
the preparation and corpus-run
plumbing,
the critic sheet's bullet),
with the suite green and each new test shown to fail without its
code.
Carena0442 is the first live run of it.

- The house policy (`src/house-policy.ts`,
  shared by every sheet) gains a bullet stating the rule with
  the Life of Pi example,
  so translators,
  judges,
  editors and the consolidation gate all read it.
- Preparation collects the source's footnote definitions and HTML comments and the archive's footnote
  definitions as lines of the identity context,
  each prefixed by side and kind (`ORIGINAL note [^1]:`,
  `ORIGINAL editor comment:`,
  `ARCHIVE note [^1]:`),
  so a translator working one slice sees notes that
  sit at the end of the page.
  The identity context is already hashed into the preparation identity and
  every cache key,
  so no scheme version changes.
  The fence that introduces the context is reworded to
  say which lines are declared identity (authoritative) and which are notes the entry carries,
  which
  establish vocabulary for the terms they name and nothing else:
  a note is document content,
  and
  `identity-context.ts` already refuses to feed document content as authoritative.
- Footnote definitions are read off the parsed document's footnote nodes (the GFM convention,
  keeping
  multi-line definitions) and the full-width bracket convention the footnote graph already handles;
  HTML
  comments with one linear scan,
  comments spanning lines included.
  The source credit comment
  (`条目贡献`) already feeds the contributor channel and is left out of the notes.
- Nothing is protected by bytes:
  a judge still decides,
  with the rule and the notes in front of it.
- Size is measured before the shape is accepted:
  the notes block rides every call for every slice of
  its entry,
  so the largest block in the corpus (hulicaijia carries the most definitions) is printed
  and recorded before the commit.

## Web lookup of official titles, cached

At 10:00 UTC the owner added `TRANSLATION_REPAIR_EXA_API_KEY` to the encrypted secrets file ("You know
what to do.
If you don't,
ask me.")
and then:
"Looking up official translations and the like can and
should be cached."
Read as:
the pipeline looks up,
through the Exa search API,
the official English
title of every work the ORIGINAL names,
and the findings reach the sheets as evidence lines beside the
notes;
every lookup is cached durably,
across runs and entries,
so a title is bought once and a resumed
run keeps its preparation identity.
My readings,
veto invited:

- What is looked up:
  every 《…》 span in the source (32 of the 92 pinned entries carry one or more,
  118
  spans in all,
  at most 13 in one entry,
  XingZ60).
  "And the like" is read as the terms the entry's notes
  name;
  those come after titles,
  the same channel and cache.
- What reaches the sheet:
  `WEB lookup for 《X》: <result title> (<url>): <highlight>` for the top few
  results,
  labelled as evidence the rule is applied to,
  not as authority:
  a search result can be wrong,
  and the house rule says what an official title is.
- Where the cache lives:
  a homedir-derived cache directory (`XDG_CACHE_HOME` or `~/.cache`,
  under
  `translation-repair/lookup/`),
  overridable by environment for tests,
  keyed by the query's digest,
  never
  under the per-launch runs directory,
  which is exactly what would re-buy every title per launch.
- No SDK:
  the workspace bans `exa-js`;
  one `fetch` against `https://api.exa.ai/search` with the
  `x-api-key` header,
  the request shape read off the current reference on 2026-09-02.
  No key set:
  no
  lookups,
  one log line,
  nothing else changes.

Positive control,
run by hand at 10:05 UTC with the drafted request (`type: auto`,
five results,
highlights of at most 300 characters asked with the same query),
results only ever printed:

- 《活着》:
  "To Live" in every result,
  the fifth quoting that the title is Yu Hua's authorised official
  translation.
  1.6 s,
  $0.007.
- 《魔法少女小圆》:
  "Puella Magi Madoka Magica" from Wikipedia,
  Anime News Network,
  MyAnimeList,
  aniSearch,
  and the Taiwanese licensee's naming notice.
  1.3 s,
  $0.007.
- 《奇妙漂流》（或称「Aiden 的奇幻漂流」）:
  no page about the group's own work;
  the results are
  奇幻漂流 neighbours,
  少年 Pi 的奇幻漂流 among them,
  which is the allusion a judge needs to see.
  1.7 s,
  $0.007.

So the corpus's 118 spans cost about a dollar once,
and the cache makes it once.

## What it does not decide

- Titles with no official English rendering and no note:
  translated,
  consistent,
  not protected.
- Names of people:
  unchanged,
  the declared-names rule.
