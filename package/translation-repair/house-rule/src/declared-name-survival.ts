import type { DeclaredIdentity, } from './identity-context.ts';
import {
  carriesName,
  nameProjection,
  type NameProjection,
  projectName,
} from './name-projection.ts';
import {
  arrivesAsWholeName,
  besideWordsOf,
  declaredWordsOf,
  excusedNeighboursOf,
} from './whole-name-arrival.ts';

//region Declared name survival
// CHECKS THAT A NAME THE DOCUMENTS DECLARE SURVIVES AN EDIT, rather than asking
// a model to preserve it.
//
// WHY A GUARD AND NOT A PROMPT. Measured 2026-08-20 against the repair lane's
// own editor-judge sheet and roster: shown two candidates differing only in a
// declared alias, SIX OF SIX judges chose the one that dropped it, every one
// reasoning that the alias "has no basis in the original". Adding a carve-out
// to the criterion moved the reasoning, and judges said in as many words that
// the alias was declared and therefore not an addition, and the vote still did
// not flip: the shorter text simply reads better and naturalness is also on the
// sheet. A stage that loses a person's name six times out of six cannot be
// fixed by explaining the rule to it again.
//
// The translate lane's judges do the same thing at three of six, which
// `doc/audit/glm-4-7-flash-across-every-stage.md` recorded.
//
// CONSERVATIVE BY CONSTRUCTION. The rule is only ever "what was already there
// stays there": a name absent from the passage being edited is never required
// to appear in its replacement. So this can refuse a real loss and can never
// demand an insertion.
//
// COMPARED ON LETTERS AND DIGITS ALONE, because a handle is written several
// ways across one archive and every one of them is the same person. Measured
// over the pinned corpus, 212 declared forms against the English body of their
// own entry: a raw substring comparison finds 111 and MISSES 12 that are
// plainly there, while the projection finds all 123 and loses none of the 111.
// The dozen divide into three shapes, none of them exotic:
//
//   - the archive escapes Markdown, writing `\_` where the declaration writes
//     `_`, which no comparison over raw text can see through;
//   - the two sides disagree about a separator, one writing a space where the
//     other writes an underscore or nothing at all;
//   - the declaration spaces a handle the body runs together, or the reverse.
//
// A LINE BREAK IS THE SAME CASE and is why this is not merely a whitespace
// fold. Our own wrapper writes a break as a newline plus the enclosing block's
// continuation prefix, so a name split inside a blockquote becomes `name` then
// `> rest`, and folding whitespace alone would still not find it. Nothing in
// the pinned corpus is wrapped that way today, measured at zero of 212, but the
// pipeline writes the archive the next pass reads.
//
// CARRIED AS A NAME, NOT AS LETTERS (ledger B23). This used to say that
// widening what counts as carried could only turn a missed loss into a caught
// one or a spurious refusal into an acceptance. It was wrong both ways: the
// projection drops the spaces between words as well as the marks inside a
// handle, so containment found `ann` inside `cannot`. A candidate that lost
// `Ann` but said `cannot` kept the name, and a base that said only `cannot`
// put an `Ann` it never held at stake. A key now counts only where each Latin
// end meets a word edge (`name-projection.ts`), a letter meeting a digit and
// a small letter meeting a capital being edges, since that is how a handle
// joins a name. Measured on 2026-09-29 over every declared and contributor
// form against the texts of its own entry: all 285 carriages in the 92
// archives and all 544 on the 214 settled pages stand as names, and 1,422 of
// 47,389 in the stored artifacts existed only across a word edge.
//
// A SWAP BETWEEN TWO FORMS OF ONE PERSON IS NOT A LOSS. The floor used to read
// every form the base carried and the candidate did not as a loss, so a
// replacement that named the subject by another of her own declared forms was
// refused, and the editor's or translator's whole slice with it. XingZ60 slice
// 80 (corpus pass XingZ6015, 2026-10-19): the archive wrote the display name
// where the original writes the alias, a repair patch resolving twelve served
// issues (a major tense error among them) and a translate replacement both wrote
// the alias, and both were refused for dropping the display name, so the archive
// paragraph shipped with its errors. A declared form the base carried is now
// met by a declared form of the same person that the base did not carry, one
// for one, and nothing else:
//
//   - PER IDENTITY. Only the forms the entry's own front matter declares for its
//     subject can meet each other. A contributor handle is another person, so a
//     contributor name replaced by a subject name, or the reverse, is still a
//     loss. A caller that cannot say which forms name the subject passes none
//     and gets the plain rule.
//   - COUNT FOR COUNT, IN DISTINCT FORMS. Each lost subject form needs its own
//     new subject form: `Mittens (Blossom)` becoming `Mittens` loses `Blossom`
//     with nothing in its place, and `Mittens (Blossom)` becoming `Patch` loses
//     two forms against one. A form the base already carried is no substitute
//     (`Mittens (Blossom)` becoming `Blossom` still loses `Mittens`), and a
//     new form counts once with the shorter forms that only run on inside it,
//     as a lost one does.
//   - AS A WHOLE NAME. A new form counts only where it stands as a whole name
//     (`whole-name-arrival.ts`): not run on from or into a letter, a digit, a
//     hyphen or an underscore, spelled as declared (`Li Li` is not `Lili`,
//     though carriage reads it as the name), and with no capitalised word beside
//     it that is not part of a declared form, an ordinary opener or title, or a
//     word the base held beside the lost form. The first swap rule passed
//     `Xing Lili`, `XingLili` and `Xing Li Li` for `Lili`, a renamed person.
//   - PRESENCE, NOT MENTIONS. The floor has only ever asked whether a form is
//     carried, never how many times, so a candidate that names the person once
//     where the base named her twice is as acceptable as one that turns the
//     second mention into a pronoun. Counting mentions for swaps alone would
//     refuse `Lili ... she` and accept `XingZ60 ... she`.
//   - NOT CHECKED AGAINST THE ORIGINAL'S PASSAGE. Which of the person's names
//     the original writes there is a question of rendering, which the accuracy
//     judges own; every form met here is one the same front matter declares, so
//     none is an invention, and the person is named wherever the base named her.
//     An exact check was measured too and would misfire: of the 43 swaps the
//     replay finds, 36 carry the source form the front matter pairs with the
//     new form, and the other seven are right all the same (a signature written
//     `瞳华` against a declared `瞳華`, a declared full name met where the
//     original writes the alias inside it).
//
// Measured 2026-10-20 over the 331 stored artifacts: 98 refusals, 85 of which
// the old rule reproduces from stored texts (the other 13 are repair patches
// whose text is not stored). The first swap rule passed 43 of the 85; 19 of
// those wrote a fusion (`Xing Lili` 11, `XingLili` 7, `Xing Li Li` 1) and the
// whole-name test refuses them again. The 24 that pass are each a lost subject
// form met by a new subject form written alone, and 61 stay refused. The new
// answer is always a subset of the old one, so no wording that passed before is
// refused now. Tried as an arrival, 67 of the 7,596 whole-name occurrences in
// the 91 pinned archives and 281 settled pages are held back: 48 are written
// with other spacing than declared (`Yi Ran` for `Yiran`), 12 are joined to a
// handle by an underscore, 7 sit beside a capitalised word of a longer name
// (`Yantian Road`, `Susie Glitter`); none is a sentence opener, though the
// opener list was drawn from those same pages, and only two of the 24 swaps
// lean on it (`Since`, `After`).

/**
 Separator between alternate handles inside one declared field.

 Corpus front matter writes `alias` as a comma-joined list, so one field can
 declare several handles and each is a name in its own right.
 */
const HANDLE_SEPARATOR = ',';

/**
 Shortest form worth checking, counted in letters and digits.

 A one or two character handle collides with ordinary words often enough that
 its survival cannot be told from an accident, and this guard is only useful
 while its answer means something.

 COUNTED ON THE PROJECTION rather than the declaration, since the projection
 is what collides. `a_b` reads as three characters and compares as two. No
 form in the pinned corpus falls between the two readings, so this is the
 right threshold in the right place rather than a change of policy.
 */
const SHORTEST_CHECKABLE_FORM = 3;

/**
 Every name form one side declares, as separate strings.

 @param identity - one side's declared identity

 @returns Declared forms, deduplicated, longest first

 @example
 ```ts
 const forms = declaredNameForms({ identity: { name: 'Mittens', alias: 'Blossom, Patch', }, },);
 ```
 */
export function declaredNameForms(
  { identity, }: { readonly identity: DeclaredIdentity; },
): readonly string[] {
  /**
   Declared fields that can carry a name, in declaration order.

   LOCATION IS DELIBERATELY ABSENT. A place is not a name for this purpose,
   and a translation may legitimately render or omit one.
   */
  const fields = [
    identity.name,
    identity.alias,
  ];

  /**
   Forms seen, so a name repeated across fields is checked once.
   */
  const seen = new Set<string>();
  for (const field of fields) {
    if (field === undefined)
      continue;
    for (const raw of field.split(HANDLE_SEPARATOR,)) {
      /**
       One handle without surrounding space.
       */
      const form = raw.trim();

      /**
       Same handle as this guard will compare it.
       */
      const key = nameProjection({ text: form, },);
      if (key.length < SHORTEST_CHECKABLE_FORM)
        continue;
      seen.add(form,);
    }
  }

  // LONGEST FIRST, so a finding names the fullest form that was lost rather
  // than a fragment of it that happens to sort earlier.
  return [ ...seen, ].toSorted(function byLengthDescending(
    left,
    right,
  ): number {
    return right.length - left.length;
  },);
}

/**
 One declared form paired with the key it is compared under.

 BOTH ARE CARRIED because they answer to different readers: the key decides
 survival, and the form as declared is what a finding must name, since an
 operator reading `mittensthecat` cannot look it up anywhere.

 @example
 ```ts
 const keyed: KeyedForm = { form: 'Mittens the Cat', key: 'mittensthecat', };
 ```
 */
type KeyedForm = {
  /**
   Form exactly as the front matter declares it.
   */
  readonly form: string;

  /**
   Same form projected onto letters and digits.
   */
  readonly key: string;

  /**
   Form's own projection, read when one lost form is looked for inside
   another.
   */
  readonly projection: NameProjection;
};

/**
 Pairs each form with the key it is compared under.

 @param forms - declared forms as the front matter writes them

 @returns One keyed form per form, in the order given

 @example
 ```ts
 const keyed = keyedForms({ forms: [ 'Mittens', 'Blossom', ], },);
 ```
 */
function keyedForms(
  { forms, }: { readonly forms: readonly string[]; },
): readonly KeyedForm[] {
  return forms.map(function toKeyed(form,): KeyedForm {
    /**
     Form's own projection.
     */
    const projection = projectName({ text: form, },);
    return {
      form,
      key: projection.key,
      projection,
    };
  },);
}

/**
 Keeps the forms that do not sit inside a longer one of the same set.

 A LONGER FORM CARRYING A SHORTER ONE REPORTS ONCE. Losing
 `Zha Ke (Lilith)` should not read as two separate losses when the shorter
 form only ever appeared inside the longer. Carried as a name, as the
 CARRIED AS A NAME, NOT AS LETTERS note says: `Ann` running on inside
 `Annabel` is its own form (ledger B23).

 @param entries - forms to fold

 @returns Forms none of the others carries, in the order given

 @example
 ```ts
 const outer = outermostForms({ entries: keyed, },);
 ```
 */
function outermostForms(
  { entries, }: { readonly entries: readonly KeyedForm[]; },
): readonly KeyedForm[] {
  return entries.filter(function isNotInsideAnother({ key: lostKey, },): boolean {
    return !entries.some(function contains(
      {
        key: otherKey,
        projection: other,
      },
    ): boolean {
      return (otherKey !== lostKey)
        && (otherKey.length > lostKey.length)
        && carriesName({
          projection: other,
          key: lostKey,
        },);
    },);
  },);
}

/**
 The forms as the front matter declares them.

 @param entries - keyed forms

 @returns Form texts a finding names, in the order given

 @example
 ```ts
 const texts = declaredFormTexts({ entries: keyed, },);
 ```
 */
function declaredFormTexts(
  { entries, }: { readonly entries: readonly KeyedForm[]; },
): readonly string[] {
  return entries.map(function toForm({ form, },): string {
    return form;
  },);
}

/**
 Declared names the base text carried and the candidate does not.

 @param forms - declared name forms to check

 @param subjectForms - the forms that name the entry's own subject, so one of
 them lost can be met by another the base did not carry; absent for the plain
 rule, which every contributor form and every caller without an identity gets

 @param baseText - text being replaced, which sets what must survive

 @param candidateText - proposed replacement

 @returns Forms present in base and absent from candidate, longest first,
 without the subject forms a new subject form stands in for

 @example
 ```ts
 const dropped = findDroppedDeclaredNames({ forms, baseText, candidateText, },);
 ```
 */
export function findDroppedDeclaredNames(
  {
    forms,
    subjectForms = [],
    baseText,
    candidateText,
  }: {
    readonly forms: readonly string[];
    readonly subjectForms?: readonly string[];
    readonly baseText: string;
    readonly candidateText: string;
  },
): readonly string[] {
  /**
   Text being replaced, projected once rather than once per form.
   */
  const base = projectName({ text: baseText, },);

  /**
   Proposed replacement, projected the same way.
   */
  const candidate = projectName({ text: candidateText, },);

  /**
   Every form beside the key it is compared under.
   */
  const keyed = keyedForms({ forms, },);

  /**
   Forms the base text actually carried as names, which are the only ones at
   stake.
   */
  const atStake = keyed.filter(function wasThere({ key, },): boolean {
    return carriesName({
      projection: base,
      key,
    },);
  },);

  /**
   Forms at stake that the candidate no longer carries as names.
   */
  const dropped = atStake.filter(function isGone({ key, },): boolean {
    return !carriesName({
      projection: candidate,
      key,
    },);
  },);

  /**
   Losses with a shorter form that only runs on inside a longer one folded
   into it.
   */
  const lost = outermostForms({ entries: dropped, },);

  /**
   Declared forms of the entry's subject, each beside the key it is compared
   under.
   */
  const subject = keyedForms({ forms: subjectForms, },);

  /**
   Keys of those forms, which decide whether a lost form is the subject's.
   */
  const subjectKeys = new Set(subject.map(function toKey({ key, },): string {
    return key;
  },),);

  /**
   Lost forms of the subject, the only ones a new subject form can meet.
   */
  const lostSubject = lost.filter(function isSubjectForm({ key, },): boolean {
    return subjectKeys.has(key,);
  },);
  if (lostSubject.length === 0)
    return declaredFormTexts({ entries: lost, },);

  /**
   Words allowed directly beside a new form: the words of every declared form,
   the ordinary capitalised words, and the words the base held beside the lost
   forms.
   */
  const excused = excusedNeighboursOf({
    declaredWords: declaredWordsOf({ forms: [
      ...forms,
      ...subjectForms,
    ], },),
    base: {
      before: lostSubject.flatMap(function beforeLost({ key, },): readonly string[] {
        return besideWordsOf({
          projection: base,
          key,
        },)
          .before;
      },),
      after: lostSubject.flatMap(function afterLost({ key, },): readonly string[] {
        return besideWordsOf({
          projection: base,
          key,
        },)
          .after;
      },),
    },
  },);

  /**
   Subject forms the candidate carries as whole names and the base did not carry.

   TOO SHORT A FORM COUNTS FOR NOTHING, for the reason the shortest checkable
   form gives: its presence cannot be told from an accident. A FORM FUSED INTO
   A NAME NOBODY DECLARED COUNTS FOR NOTHING EITHER, as the `WHOLE-NAME ARRIVAL`
   note says.
   */
  const arrived = subject.filter(function isNew({
    form,
    key,
  },): boolean {
    if (key.length < SHORTEST_CHECKABLE_FORM)
      return false;
    if (carriesName({
      projection: base,
      key,
    },))
      return false;
    return arrivesAsWholeName({
      projection: candidate,
      form,
      key,
      excused,
    },);
  },);

  /**
   New subject forms, a longer one and the shorter form that only runs on
   inside it counted once, as the lost forms are.
   */
  const replacements = outermostForms({ entries: arrived, },);

  // ONE NEW FORM FOR EACH LOST ONE.
  if (replacements.length >= lostSubject.length) {
    return declaredFormTexts({
      entries: lost.filter(function isOtherPerson({ key, },): boolean {
        return !subjectKeys.has(key,);
      },),
    },);
  }
  return declaredFormTexts({ entries: lost, },);
}

/**
 Renders a declared-name refusal as a finding.

 @param sliceIndex - slice the refusal names

 @param dropped - declared forms the replacement no longer carries

 @returns Finding in scorecard-stable wording

 @example
 ```ts
 const finding = declaredNameRefusalFinding({ sliceIndex: 3, dropped: [ 'Blossom', ], },);
 ```
 */
export function declaredNameRefusalFinding(
  {
    sliceIndex,
    dropped,
  }: {
    readonly sliceIndex: number;
    readonly dropped: readonly string[];
  },
): string {
  /**
   Lost forms quoted, so a finding cannot be misread as prose.
   */
  const quoted = dropped.map(function quote(form,): string {
    return JSON.stringify(form,);
  },);
  return `translate-refused-declared-name (slice ${String(sliceIndex,)}: archive text carries ${
    quoted.join(', ',)
  } and the replacement does not; keeping the archive text)`;
}

/**
 Everything one refusal contributes to a settled slice.

 GATHERED IN ONE PLACE because a refusal has to show up in three: the record
 field a reader queries, the finding a scorecard counts, and the log an
 operator watches. Spelled out at each call site, the three drift apart, and a
 refusal missing from any one of them is a refusal nobody can audit.

 @example
 ```ts
 const report = declaredNameRefusalReport({ sliceIndex: 3, dropped, },);
 ```

 @internal
 */
export type DeclaredNameRefusalReport = {
  /**
   Fragment to spread into the settled record.

   ALWAYS NAMES THE FIELD, empty array when nothing was refused, because the
   repair outcome requires it: a reader of a settled slice should never have
   to tell "dropped nothing" from "nobody wrote the field".
   */
  readonly record: { readonly droppedDeclaredNames: readonly string[]; };

  /**
   Findings to append, empty when nothing was refused.
   */
  readonly findings: readonly string[];
};

/**
 Gathers what a refusal owes the record, the findings and the log.

 @param sliceIndex - slice the refusal names

 @param dropped - declared forms the replacement no longer carries, empty when
 it dropped none

 @returns Record fragment and findings, both empty when nothing was refused

 @example
 ```ts
 const { record, findings, } = declaredNameRefusalReport({ sliceIndex, dropped, },);
 ```

 @internal
 */
export function declaredNameRefusalReport(
  {
    sliceIndex,
    dropped,
  }: {
    readonly sliceIndex: number;
    readonly dropped: readonly string[];
  },
): DeclaredNameRefusalReport {
  if (dropped.length === 0) {
    return {
      record: { droppedDeclaredNames: [], },
      findings: [],
    };
  }
  return {
    record: { droppedDeclaredNames: dropped, },
    findings: [
      declaredNameRefusalFinding({
        sliceIndex,
        dropped,
      },),
    ],
  };
}

//endregion Declared name survival
