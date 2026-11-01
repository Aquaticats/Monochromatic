# Front-matter publication guard: structural checks only

Decided by the owner on 2026-09-02,
in answer to the question whether a page whose metadata equals the
archive's should be refused,
published when a panel chose the keep,
or published outright.
The owner's
answer was to the premise:
"why are we caring about metadata being different vs Chinese source at all?"
Landed as commit `34e5c7ecd` in `package/translation-repair/page-check/src/corpus-run/front-matter-completeness.ts`.

## The rule

`assertFrontMatterComplete` refuses a page only on structural grounds:

- `missing-slice`:
  the preparation carries no metadata slice,
  or carries it anywhere but slice zero over
  exactly the two sides' front-matter bytes.
- `invalid-page`:
  the page's metadata does not parse,
  or breaks the identity rule (source name equal to
  alias requires the same on the page) or the contributor-attribution rule in `validateFrontMatterTranslation`.
- `directory-id-name`:
  the page's visible `name` is the entry's directory id while the source's is not,
  checked on the assembled page whether or not it equals the archive byte for byte.
  Narrowed by
  `6d85b619a` on a census of the pinned corpus:
  23 of 92 archives name the directory,
  and 8 of them
  (Anilovr,
  Arita,
  ArtsEpiphany,
  Hangmster,
  keyword233,
  Mio,
  mone,
  s5ehfr9) do so in the source too,
  because the handle is the person's name;
  the page-only form of `34e5c7ecd` would have refused those eight forever.

Whether the lanes kept the archive's metadata or replaced it is not a question the guard asks.
The lanes,
the lane contest and the consolidation gate judge the metadata slice like every other slice,
and the
artifact keeps their records for the reading.

## What it replaces

The rule of 2026-08-28 (`69df7d881`,
"review visible front matter",
written for
archives whose
metadata was never translated and still named the directory id) refused any page whose metadata equalled
the archive's while the source's differed,
reading that as nobody having reviewed the slice.
Chinese and
English metadata always differ,
so the trigger fired on every kept incumbent:
it discarded the Carena0442
pass of 2026-09-02 after 94 minutes,
and would have discarded the Toka_ls relaunch,
whose consolidation
gate kept the archive six ballots to two with reasons.

Three commits that night (`daaf0ffa0`,
`6f70a2085`,
`1160ebb4c`) replaced the byte comparison with a
reading of which panel had chosen the keep (translate judges,
every heard translator,
lane contest,
consolidation slate,
consolidation gate).
They answered the question well,
and the question only existed
because of the proxy.
They were removed with the proxy.
The reading of the Toka_ls run that motivated them
is in `doc/planning/translation-repair-toka-ls-reading-2026-09-02.md`;
the night's record is in
`doc/planning/translation-repair-roster-calibration-2026-09-01.md`.

## Consequences

- A hold-starved judge indecision on the metadata slice publishes the archive's metadata as it stands,
  as
  it does for any other slice.
  The artifact records the indecision;
  the reading catches it.
- The directory-id refusal is wider than before in one direction and narrower in another:
  it fires on any
  assembled page whose visible name is the directory id,
  not only on a byte-equal keep,
  and it does not
  fire where the source names the person by that same handle.
  The 15 archives that show the directory id
  where the source has a name of its own (the directory-id shape) stay refused until a lane renders the name.
- `FrontMatterCompletenessError` carries no decision detail any more;
  its message names the entry and the
  structural reason only.

## Addendum 2026-09-04: the identity rule reads containment, not equality

Decided by the owner ("Alias may carry the name among other renderings"),
asked with three options after
the luxuanwen3 pass of 2026-09-04 ended `INCOMPLETE` on `invalid-page`:
its source declares `name: 鲵鲵`,
`alias: 鲵鲵`,
and the archive's own front matter has `name: Nini`,
`alias: 鲵鲵, Nini`,
which the equality
rule refused.
Measured over the pinned corpus:
14 of 92 sources declare name equal to alias,
and in 7 of
them the archive renders the alias with more than the name (MizuharaNagisa,
SevenBird,
Weideriche_,
gaoyanger "Gaoyang,
Lamb",
interrgned,
luxuanwen3,
noname "noname,
no name,
anonymous,
...").
Under
equality none of the seven could ship an archive-shaped front matter.

The rule now:
where the ORIGINAL declares name and alias the same identity,
the translated name must appear
among the comma-separated renderings of the translated alias (`aliasCarriesName` in
`front-matter-translation.ts`;
the archives separate alias lists with a comma in 70 cases and a slash in one,
never a Chinese comma).
A name absent from its alias is still refused.
The judges' decision rule,
the
translators' front matter rule and the selection criteria say the same.
Guard:
the
`front-matter-completeness` test accepting `alias: 猫猫, Maomao` for `name: Maomao` and refusing
`alias: 猫猫, Kitty`,
shown to fail with equality put back.

Options rejected:
strict equality (forces the seven archives to drop the original-script alias they
publish);
dropping the identity rule (name and alias could diverge freely).

## Addendum 2026-09-07: the directory id as a rendering of the name

Decided by the owner ("pinyin check and alias exemption;
see both the original Chinese and the original
English,
there's gotta be an English rendering in the frontmatter"),
asked with four options after the
third Huasheng pass ended `INCOMPLETE` on `directory-id-name`:
the source names the person 椛笙,
whose
pinyin is Huasheng,
which is the directory id,
and the judges had chosen it by the identity rule.
Measured over the pinned corpus:
22 archives name their directory;
8 do so in the source too;
of the other
14,
the id is a rendering of the name for 7 (Huasheng,
lintong,
Kotori,
MioCardMeow,
MocaKawai,
noname,
donotexist_A) and a handle standing where the name goes for 7 (DarlinChit,
dogesir_,
homoyamakaze,
interrgned,
lxyddice,
Weideriche_,
XingZ60).

Landed in `package/translation-repair/glossary/src/corpus-run/directory-id-name.ts`.
The id-equal name
stands when any of three holds,
read on the original Chinese and the original English front matter:

- the id,
  lower-cased and reduced to its Latin letters,
  is a pinyin reading of the source's `name`,
  every
  heteronym of every character allowed,
  through `pinyin-pro` (new dependency,
  MIT,
  no dependencies of its own);
- the source itself carries the id among its aliases,
  so the handle is the person's own;
- the page or the archive carries an alias in Latin script other than the id,
  the English rendering the
  front matter has to have.

What stays refused is a page whose visible name is the folder and whose front matter,
on both sides,
carries no Latin rendering but the folder itself:
the directory-id shape as it was literally described.
All seven handles carry a Latin rendering in the archive alias (Sakuya,
Lan Gou,
Qian Yu Mao Tou,
Danpian,
lxy,
Zihe,
Lili) and so stand under the third clause,
which the owner's words accept.

## Addendum 2026-09-08: the archive's front matter stands

Decided by the owner ("We're not supposed to change front matter though?",
then option 1 of three:
"publish the archive's front matter as is;
render it only where the archive never translated it"),
asked after the ninth hakureico pass shipped `name: Kagurazaka Chika` in slice zero
(the translate lane's faithful rendering of 神楽坂千歌,
six ballots to three at the contest,
endorsed nine of nine at the consolidation)
while the body kept the archive's `Hanasaka` and the letter's signature `Kagurazaka Hanasaka`,
since the declared-name guard takes its forms from the archive's front matter and exempts slice zero on purpose.
The census that went with the question:
13 of the 127 pages shipped under `~/temp/agent` changed the archive's front matter,
among them 9 of the last 10 read pages,
none of which the read-page checks had diffed;
TLL1122's `desc` "Lulu loves you!"
became "This is Tian Lulu" followed by an em-dash and "hope every day brings you happiness!",
yulianNyanner's "In the end,
we heard her voice."
became "Together,
we resist this unjust world even after death.",
Hangmster's "They had brought us laughter."
became "Everyone's joy.",
and aliases gained forms in the original script.

Options offered,
ranked 1 over 2 over 3:
1 publish the archive's front matter as is and render it only where the archive never translated it;
2 freeze `name` and `desc` and leave `alias` and `location` repairable;
3 keep the lanes' rendering and add the front matter to the read-page checks.
1 over 2 because the owner's expectation is no change and a rewritten `desc` is an editorial change nobody asked for;
2 over 3 because 3 keeps rewriting `desc` on most pages.

THE RULE,
landed in `package/translation-repair/page-check/src/corpus-run/archive-front-matter.ts` (`20e5135a6`):
the archive's front matter stands unless the archive shows the directory id as the visible name while the source names
the person and none of the three clauses of 2026-09-07 makes the id stand,
which is the directory-id shape as literally described.
Where it stands,
the preparation makes no metadata slice
(`prepareDocumentPair({ frontMatterAuthority: 'archive' })`,
decided once per entry in `pass-prepare.ts` and logged as `FRONT MATTER entry=<id> authority=archive`),
so no lane spends on it and no judge is asked to prefer a rendering over the memorial's choice;
the page carries the archive's bytes;
and `assertFrontMatterComplete` recomputes the answer from the two documents and refuses,
as `archive-front-matter`,
a page whose front matter differs from the archive's or a preparation that rendered a standing archive.
Where it does not stand,
everything is as the 2026-09-02 decision and its addenda left it.
The artifact records it:
generation eleven's preparation carries `frontMatterAuthority: 'archive'` and no slice zero,
and the rebuild reads that off the file rather than recomputing the rule,
so a later change to the rule cannot re-slice an older artifact.

MEASURED OVER THE PINNED CORPUS on the day of the decision:
every one of the 92 archives stands.
23 name their directory;
8 of those because the source does too;
the other 15 (Acheron,
DarlinChit,
Huasheng,
Kotori,
MioCardMeow,
MocaKawai,
Weideriche_,
XingZ60,
dogesir_,
donotexist_A,
homoyamakaze,
interrgned,
lintong,
lxyddice,
noname) all stand by the clauses of 2026-09-07.
So on this corpus the lanes render no front matter at all,
and the rendering path is kept for the directory-id shape a future archive may still have.
A consequence the owner should know:
the front-matter defects the earlier decisions were written for
(an archive still naming its folder where the source names the person)
are no longer corrected by the pipeline where the archive also carries a Latin alias;
they are the archive's to correct by hand.

Guards:
`archive-front-matter.unit.test.ts` (the ninth pass's shape stands,
the directory-id shape does not,
each 2026-09-07 clause,
the two one-sided cases),
`front-matter-completeness.unit.test.ts` (the standing archive accepted as it is with no slice,
refused with a rendered slice,
refused when the page changes it,
the hakureico rendering refused and the archive's bytes accepted;
every rendered-branch case moved onto a folder-named archive with no Latin alias),
`document-preparation.unit.test.ts` (no slice zero under the archive's authority,
no legacy identity),
`artifact-two-lane-rebuild.unit.test.ts` (generation eleven rebuilt off the record),
and the pass-entry case that carries a translated archive through the whole pass.
Shown to fail first:
the predicate neutralised fails 4,
7 and 2 cases across three suites,
the preparation neutralised 2,
2 and 2,
the guard's archive branch neutralised 7 and 2;
restored,
0.

Options rejected:
letting the body follow a corrected front matter (re-derives the declared forms exactly where six of six judges once dropped
a declared alias);
freezing the whole front matter to the archive even where it names the folder (the directory-id shape would then never be rendered).
