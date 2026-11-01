import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { hanItemsClause, } from './identity-han-items.ts';
import {
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { isLatinLetterOrMark, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Identity context
// Corpus pages declare who the entry is about in YAML front matter, and the two
// sides declare it in their own languages: `name: 委委-fairy` against
// `name: Acheron`, `name: 岁月封华` against `name: Suigetsu Houka`. Front matter
// is parsed onto `RepairDocument` but reached no downstream consumer, so critics
// judged names with the declaration withheld and reported correct, sourced
// renderings as unsubstantiated substitutions. Milestone three's graded sample
// scored three such false positives (Acheron, BI4PBV, Toka_ls). This module
// turns both sides' declarations into one compact block the critic prompt can
// carry, so a declared correspondence is evidence the model has rather than
// context it must guess.
//
// Only declaration fields are surfaced. Front matter carries free prose too
// (`desc`), which is document content rather than identity, and feeding it as
// authoritative would license real defects inside it.

/**
 Front matter keys carrying identity, in the order a reader wants them.
 `name` is the display name, `info.alias` the alternate handles, and
 `info.location` the place name, which transliterates the same way names do.
 */
const IDENTITY_FIELDS = [
  'name',
  'alias',
  'location',
] as const;

/**
 One side's declared identity, every field optional because corpus metadata
 shapes vary per entry and many pages declare only a name.

 @example
 ```ts
 const declared: DeclaredIdentity = { name: 'Acheron', alias: 'Fairy, Acheron', };
 ```
 */
export type DeclaredIdentity = {
  /**
   Display name as declared, absent when the page declares none.
   */
  readonly name?: string;

  /**
   Alternate handles as declared, comma-joined by the corpus itself.
   */
  readonly alias?: string;

  /**
   Place name as declared.
   */
  readonly location?: string;

  /**
   Date of death the corpus entry records (`info.died` in its `info.yml`), as
   written there. Only the source side carries it.
   */
  readonly died?: string;
};

/**
 Narrows a raw front matter value to a usable declaration. Corpus metadata is
 `unknown` by type and hand-edited in practice, so a number, list, or null in
 a name field must be rejected rather than coerced: a coerced value would
 enter the prompt as an authoritative correspondence.

 @param value - raw front matter value

 @returns Whether the value is a string carrying non-whitespace content

 @example
 ```ts
 if (isDeclared('Mittens',)) { }
 ```
 */
export function isDeclared(value: unknown,): value is string {
  return ((typeof value) === 'string')
    && (value.trim()
      .length
      > 0);
}

/**
 Reads one field into a fragment that spreads into a {@link DeclaredIdentity}.
 An undeclared field yields no key at all, which is what `?:` optionality
 means under `exactOptionalPropertyTypes`, so absence never has to travel as
 a value.

 @param record - candidate record to read from

 @param key - field name to read

 @returns Single-entry fragment, empty when the field is not declared

 @example
 ```ts
 const fragment = declaredFragment({ record: { name: 'Mittens', }, key: 'name', },);
 ```
 */
function declaredFragment(
  {
    record,
    key,
  }: {
    readonly record: Readonly<Record<string, unknown>>;
    readonly key: string;
  },
): Readonly<Record<string, string>> {
  /**
   Raw field value, still untyped.
   */
  const value = record[key];
  return isDeclared(value,)
    ? { [key]: value.trim(), }
    : {};
}

/**
 Extracts declared identity from one document's parsed front matter data.
 Reads `name` at the top level and `alias`/`location` from the nested `info`
 record, which is the shape the pinned corpus uses.

 @param data - parsed front matter value, `unknown` because shapes vary

 @returns Declared identity, empty when nothing is declared

 @example
 ```ts
 const identity = extractDeclaredIdentity({
   data: { name: 'Suigetsu Houka', info: { alias: 'Suigetsu', }, },
 },);
 ```
 */
export function extractDeclaredIdentity(
  { data, }: { readonly data: unknown; },
): DeclaredIdentity {
  if (!isJsonRecord(data,))
    return {};

  /**
   Nested `info` record when present; alias and location live here.
   */
  const info = isJsonRecord(data.info,)
    ? data.info
    : {};

  return {
    ...declaredFragment({
      record: data,
      key: 'name',
    },),
    ...declaredFragment({
      record: info,
      key: 'alias',
    },),
    ...declaredFragment({
      record: info,
      key: 'location',
    },),
  };
}

/**
 Reads the date of death from one corpus entry's parsed `info.yml`, the
 nested `info.died` field the pinned corpus records. Front matter never
 declares it, so nothing here competes with a front matter declaration.

 A BARE YEAR IS A DATE. YAML reads `died: 2025` as the number 2025, and a
 date written in full (`2025-02-26`) as text, so the year alone would have
 switched the feature off for its entry without a word. A finite number is
 recorded as the text of that number; a number with no text of its own (not a
 number, infinity) and any other non-text value stay undeclared, as they do
 for every other identity field; `unreadableDeathKindOf` names such a value, so
 the reader of the entry's `info.yml` can say so once instead of this function
 saying it at every call.

 @param entryInfo - parsed `info.yml` of the entry, `unknown` because corpus metadata is hand-edited

 @returns Declared identity carrying `died` when the entry records one, empty otherwise

 @example
 ```ts
 const recorded = recordedDeathOf({ entryInfo: { info: { died: '2025-02-26', }, }, },);
 ```
 */
export function recordedDeathOf(
  { entryInfo, }: { readonly entryInfo: unknown; },
): DeclaredIdentity {
  if (!isJsonRecord(entryInfo,))
    return {};

  /**
   Nested `info` record of the entry, empty when the entry has none.
   */
  const info = isJsonRecord(entryInfo.info,)
    ? entryInfo.info
    : {};
  /**
   Raw `died` value, still untyped.
   */
  const { died, } = info;
  if (((typeof died) === 'number') && Number.isFinite(died,))
    return { died: String(died,), };
  return declaredFragment({
    record: info,
    key: 'died',
  },);
}

/**
 Names the kind of a `died` value that {@link recordedDeathOf} leaves
 unrecorded without it being a blank, so a reader of the entry's `info.yml` can
 say so: a hand-edited `died: true` would otherwise switch the feature off for
 its entry in silence.

 NO WORD FOR WHAT IS NO MISTAKE. A `died` that is absent, text (blank text
 included) or a finite number is recorded or is an entry with no death, and one
 left blank (`died:` with no value, which YAML reads as null) is the same
 absence; none of them is named here.

 @param entryInfo - parsed `info.yml` of the entry, `unknown` because corpus metadata is hand-edited

 @returns One phrase naming the kind of the value, or none when the value is recorded or is no mistake

 @example
 ```ts
 const kinds = unreadableDeathKindOf({ entryInfo: { info: { died: true, }, }, },);
 ```
 */
export function unreadableDeathKindOf(
  { entryInfo, }: { readonly entryInfo: unknown; },
): readonly string[] {
  if (!isJsonRecord(entryInfo,))
    return [];

  /**
   Nested `info` record of the entry, empty when the entry has none.
   */
  const info = isJsonRecord(entryInfo.info,)
    ? entryInfo.info
    : {};
  /**
   Raw `died` value, still untyped.
   */
  const { died, } = info;
  // Absent, blank (YAML reads `died:` with no value as null) and text are recorded or no mistake.
  if (died === undefined)
    return [];
  if (died === null)
    return [];
  if ((typeof died) === 'string')
    return [];
  if ((typeof died) === 'number')
    return Number.isFinite(died,) ? [] : ['a number that is not finite',];
  if ((typeof died) === 'boolean')
    return ['a truth value',];
  if (isJsonArray(died,))
    return ['a list',];
  if (isJsonRecord(died,))
    return ['a record',];
  return [`a value of type ${typeof died}`,];
}

/**
 Whether an entry's parsed `info.yml` records a death, by the same declaration
 rule every identity field follows.

 @param entryInfo - parsed `info.yml` of the entry, `unknown` because corpus metadata is hand-edited

 @returns True when `info.died` is a declared, non-blank string

 @example
 ```ts
 const ended = lifeEndedOf({ entryInfo: { info: { died: '2025-02-26', }, }, },);
 ```
 */
export function lifeEndedOf(
  { entryInfo, }: { readonly entryInfo: unknown; },
): boolean {
  return isDeclared(recordedDeathOf({ entryInfo, },)
    .died,);
}

/**
 Renders the line that gives the recorded date of death, present only when the
 original side's entry records one.

 @param source - original side's declared identity

 @returns One line naming the date, or nothing when no death is recorded

 @example
 ```ts
 const lines = deathLines({ source: { died: '2025-02-26', }, },);
 ```
 */
function deathLines(
  { source, }: { readonly source: DeclaredIdentity; },
): readonly string[] {
  if (!isDeclared(source.died,))
    return [];
  return [
    // Written as a JSON string, so a quotation mark or a line break in a hand-edited
    // value cannot close the line or start another; a date reads the same.
    `- died: the date this person died, as the corpus records it: ${JSON.stringify(source.died,)}.`,
  ];
}

/**
 Renders one field's declared correspondence, kept only when at least one
 side declares it. A one-sided declaration still earns its line: it tells the
 critic the handle is sourced metadata rather than invention, which is exactly
 the judgment that produced the graded false positives.

 @param field - identity field to render

 @param source - original side's declaration

 @param target - translation side's declaration

 @returns Single-line list for a declared field, empty when neither side
 declares it, so callers flatten instead of filtering absent values

 @example
 ```ts
 const lines = renderField({ field: 'name', source, target, },);
 ```
 */
function renderField(
  {
    field,
    source,
    target,
  }: {
    readonly field: typeof IDENTITY_FIELDS[number];
    readonly source: DeclaredIdentity;
    readonly target: DeclaredIdentity;
  },
): readonly string[] {
  /**
   Original side's declared value for this field, when declared at all.
   */
  const sourceValue = source[field];

  /**
   Translation side's declared value for this field, when declared at all.
   */
  const targetValue = target[field];
  if ((!isDeclared(sourceValue,)) && (!isDeclared(targetValue,)))
    return [];

  return [
    `- ${field}: ORIGINAL declares ${
      isDeclared(sourceValue,)
        ? `"${sourceValue}"`
        : '(nothing)'
    }, TRANSLATION declares ${
      isDeclared(targetValue,)
        // LEDGER F-3: an item still in Han is named as no English rendering.
        ? `"${targetValue}"${hanItemsClause({ value: targetValue, },)}`
        : '(nothing)'
    }`,
  ];
}

/**
 Third-person singular pronouns a Chinese original can use for its subject,
 in the order a tie is broken.

 `TA` IS THE NEUTRAL FORM this corpus writes for a person who did not specify,
 in any casing; the word boundary keeps `ta` inside a romanised handle out.
 */
const SOURCE_PRONOUNS = [
  '她',
  '他',
  'TA',
] as const;

/**
 What the pronoun line says where the original writes more than one form:
 the house rule's as-written sentence, so a sheet reading the line alone
 cannot take the dominant form as overriding a pronoun the original wrote
 (class eighty-one).
 */
const AS_WRITTEN_CLAUSE = 'each form is rendered as written where it stands (她 is she, 他 is he, TA is singular they), and';

/**
 Compounds that contain a pronoun character without being that pronoun:
 plurals and the "other" words. Removed before counting, longest first so
 其他人 is not left as 人 after 其他 goes.
 */
const COMPOUNDS_HIDING_A_PRONOUN = [
  '其他人',
  '其他',
  '他们',
  '他人',
  '她们',
] as const;

/**
 Counts how often one han pronoun occurs in a text, compounds removed first.

 AN INDEX SCAN RATHER THAN A PATTERN, since the needle is a fixed string and
 the count is all that is wanted.

 @param text - original document

 @param pronoun - fixed form to count

 @returns Occurrences outside the compounds

 @example
 ```ts
 countHanPronoun({ text: '她走了。她们笑了。', pronoun: '她', },);
 // => 1
 ```
 */
function countHanPronoun(
  {
    text,
    pronoun,
  }: {
    readonly text: string;
    readonly pronoun: string;
  },
): number {
  /**
   Text with every compound cut out, so what remains of the character is the
   pronoun itself.
   */
  const bare = COMPOUNDS_HIDING_A_PRONOUN.reduce(
    function without(
      remaining,
      compound,
    ): string {
      return remaining.replaceAll(
        compound,
        '',
      );
    },
    text,
  );
  return bare.split(pronoun,)
    .length
    - 1;
}

/**
 What `indexOf` answers when the form is not found.
 */
const NOT_FOUND = -1;

/**
 Spellings the sources give the neutral pronoun.

 ALL THREE CASINGS ARE THE PRONOUN. Measured over the pinned corpus on
 2026-09-04, after the SS3B_0016 page shipped a bare "Ta" the counter had not
 seen: sources write `TA` in 2 entries, `Ta` in 7 and `ta` in 8, every
 occurrence the pronoun. The word boundary, not the casing, keeps DATA,
 STATION and a romanised handle out of the count.
 */
const NEUTRAL_SPELLINGS = [
  'TA',
  'Ta',
  'ta',
] as const;

/**
 Character that makes the neutral pronoun plural: `TA 们` is "they" for
 several people, not the subject's own pronoun, and six sources write it.
 */
const PLURAL_SUFFIX = '们';

/**
 Whether an occurrence is followed, spaces aside, by the plural suffix.

 @param text - original document

 @param from - position just after the occurrence

 @returns Whether 们 is the next non-space character

 @example
 ```ts
 isPluralAfter({ text: 'TA 们来了', from: 2, },);
 // => true
 ```
 */
function isPluralAfter(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): boolean {
  // One forward cursor over the spaces; the first other character decides.
  for (let cursor = from; cursor < text.length; cursor += 1) {
    /**
     Character under the cursor.
     */
    const character = text.charAt(cursor,);
    if (character !== ' ')
      return character === PLURAL_SUFFIX;
  }
  return false;
}

/**
 Counts how often one spelling of the neutral pronoun stands as a word of
 its own, singular.

 ONE LINEAR PASS over the text with the string API: each occurrence is found
 from the previous one and its neighbours are read once.

 @param text - original document

 @param spelling - fixed form to count

 @returns Occurrences bounded by non-letters and not made plural

 @example
 ```ts
 countNeutralSpelling({ text: 'TA来了。DATA', spelling: 'TA', },);
 // => 1
 ```
 */
function countNeutralSpelling(
  {
    text,
    spelling,
  }: {
    readonly text: string;
    readonly spelling: string;
  },
): number {
  /**
   Occurrences and the position to search from, advanced together.
   */
  const scan = {
    count: 0,
    from: 0,
  };
  for (
    let at = text.indexOf(
      spelling,
      scan.from,
    );
    at !== NOT_FOUND;
    at = text.indexOf(
      spelling,
      scan.from,
    )
  ) {
    /**
     Whether a Latin letter sits either side, or a combining mark after it,
     which makes this a longer word: `éTA` or the tone-marked `TA\u{0300}`
     is not the pronoun (ledger B18).
     */
    const insideWord = isLatinLetterOrMark({ character: text.charAt(at - 1,), },)
      || isLatinLetterOrMark({ character: text.charAt(at + spelling.length,), },);
    /**
     Whether the occurrence is the plural, which is not the subject's pronoun.
     */
    const plural = isPluralAfter({
      text,
      from: at + spelling.length,
    },);
    if ((!insideWord) && (!plural))
      scan.count += 1;
    scan.from = at + spelling.length;
  }
  return scan.count;
}

/**
 Counts how often the neutral pronoun stands as a word of its own, in any of
 its spellings.

 @param text - original document

 @returns Occurrences across TA, Ta and ta

 @example
 ```ts
 countNeutralPronoun({ text: 'TA来了。Ta 走了。DATA', },);
 // => 2
 ```
 */
function countNeutralPronoun(
  { text, }: { readonly text: string; },
): number {
  return NEUTRAL_SPELLINGS.reduce(
    function summed(
      total,
      spelling,
    ): number {
      return total + countNeutralSpelling({
        text,
        spelling,
      },);
    },
    0,
  );
}

/**
 Names the pronoun the original uses for its subject, as one identity line.

 WHY THIS LINE EXISTS. The Toka_ls relaunch of 2026-09-02 rendered a
 subjectless sentence (偶尔灵感迸发，左右推敲，留下工整的格律) with "they" for a
 person the page calls "she" throughout, and all eight judges passed it
 reasoning that "the Chinese gives no pronoun". The original uses 她 twenty
 times on sixteen lines. Chinese leaves subjects unstated freely, so the pronoun is a fact
 about the document, not about the sentence, and this states it once where
 every sheet already reads the declared identity.

 ONE LINE OR NONE. The dominant form is named with its count; a document that
 uses no third-person singular pronoun at all yields nothing, which leaves
 the house rule on neutral pronouns to speak for itself.

 EVERY FORM THE ORIGINAL WRITES, WHERE IT MIXES THEM (class eighty-one,
 XingZ621 slice 14, 2026-09-22). The line named 她 fifty-one times alone on a
 ten-part page whose Part One writes TA forty times and discusses the
 pronoun, and the consolidation bench split between "she" on this line and
 "they" on the house rule's TA sentence until the entry stopped. The
 dominant form settles only the subjects the original leaves unstated; a
 pronoun the original writes is rendered as written where it stands, so the
 line now names the other forms with their counts and says so.

 @param text - whole original document

 @returns Single-line list naming the dominant pronoun and its count, with
 every other form the original writes and the as-written rule where there
 is one, empty when the original uses none

 @example
 ```ts
 sourcePronounLines({ text: '她睁开双眼。她笑了。', },);
 // => ['- pronoun: ORIGINAL refers to this person as "她" (2 times)']
 ```
 */
export function sourcePronounLines(
  { text, }: { readonly text: string; },
): readonly string[] {
  /**
   Each form with its count, in tie-break order.
   */
  const counts = SOURCE_PRONOUNS.map(function counted(pronoun,): {
    readonly pronoun: string;
    readonly count: number;
  } {
    return {
      pronoun,
      count: (pronoun === 'TA')
        ? countNeutralPronoun({ text, },)
        : countHanPronoun({
          text,
          pronoun,
        },),
    };
  },);

  /**
   The form used most, the earlier form winning a tie.
   */
  const dominant = counts.reduce(function moreUsed(
    best,
    candidate,
  ) {
    return (candidate.count > best.count) ? candidate : best;
  },);
  if (dominant.count === 0)
    return [];

  /**
   How the dominant form is named.
   */
  const dominantLine = `- pronoun: ORIGINAL refers to this person as "${dominant.pronoun}" (${
    String(dominant.count,)
  } ${
    wordForCount({
      count: dominant.count,
      one: 'time',
      many: 'times',
    },)
  })`;

  /**
   Other forms the original writes too, in the fixed order of the forms.
   */
  const others = counts
    .filter(function alsoWritten(candidate,): boolean {
      return (candidate !== dominant) && (candidate.count > 0);
    },)
    .map(function named(candidate,): string {
      return `"${candidate.pronoun}" (${String(candidate.count,)} ${
        wordForCount({
          count: candidate.count,
          one: 'time',
          many: 'times',
        },)
      })`;
    },);
  if (others.length === 0)
    return [dominantLine,];

  return [
    `${dominantLine}, and also writes ${others.join(' and ',)}; ${AS_WRITTEN_CLAUSE} "${dominant.pronoun}" `
      + 'supplies the pronoun only where the ORIGINAL leaves the subject unstated',
  ];
}

/**
 Collects the identity lines both sides' front matter declares, ready to
 embed in a critic prompt. An empty result means neither side declared
 anything, so callers omit the block rather than emit an empty heading;
 absence stays a zero-length list instead of travelling as a nullish value.

 @param sourceData - original document's parsed front matter value

 @param targetData - translation document's parsed front matter value

 @param sourceEntryInfo - parsed `info.yml` of the original's corpus entry, absent when the caller has none; its recorded death joins the original's declared identity, and front matter keeps precedence for any field it declares

 @returns Rendered lines, empty when no field is declared on either side

 @example
 ```ts
 const lines = collectIdentityLines({
   sourceData: { name: '委委-fairy', },
   targetData: { name: 'Acheron', },
 },);
 ```
 */
export function collectIdentityLines(
  {
    sourceData,
    targetData,
    sourceEntryInfo,
  }: {
    readonly sourceData: unknown;
    readonly targetData: unknown;
    readonly sourceEntryInfo?: unknown;
  },
): readonly string[] {
  /**
   Original side's declared identity, its front matter and its recorded death.
   */
  const source: DeclaredIdentity = {
    ...recordedDeathOf({ entryInfo: sourceEntryInfo ?? {}, },),
    ...extractDeclaredIdentity({ data: sourceData, },),
  };

  /**
   Translation side's declared identity.
   */
  const target = extractDeclaredIdentity({ data: targetData, },);

  return [
    ...IDENTITY_FIELDS
      .flatMap(function toLines(field,) {
        return renderField({
          field,
          source,
          target,
        },);
      },),
    ...deathLines({ source, },),
  ];
}

//endregion Identity context
