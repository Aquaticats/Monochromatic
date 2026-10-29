import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import { errorOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import { parse as parseYaml, } from 'yaml';

import {
  type CorpusPin,
  isMissingCorpusObject,
  type readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  refusalText,
  yamlRefusalSite,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { unreadableDeathKindOf, } from '@monochromatic-dev/translation-repair-house-rule/ts';

//region Entry info record
// THE ENTRY'S `info.yml` AT THE PIN, read where a preparation reads the entry's
// pages, because the date of death lives only there (`info.died`) and never in
// front matter. A missing file is a known state of the corpus (one entry has
// none), so it is named at info and recorded as no death, never thrown. A file
// that is there and refuses to parse is recorded as no death too, as its own kind
// so a later reader can tell it from the absence, and named at warn: the owner's
// rule is that a run always ships, so a record the models were never told is
// the entry's page shipped without the tense block, not the entry stopped. The
// YAML is read with the same `yaml` parser the package reads front matter with.
// A `died` of a kind the feature cannot read is recorded as no death too, named at
// warn and on the artifact's findings, for the same reason: the artifact must tell
// that entry from one that records no death.

/**
 What the pinned corpus holds for an entry's `info.yml`.

 @example
 ```ts
 const record: EntryInfoRecord = { kind: 'read', info: { info: { died: '2025-02-26', }, }, };
 ```
 */
export type EntryInfoRecord = {
  /**
   The entry has no `info.yml` at the pin, so it records no death.
   */
  readonly kind: 'absent';
} | {
  /**
   The entry's parsed `info.yml`, `unknown` because corpus metadata is hand-edited.
   */
  readonly kind: 'read';
  readonly info: unknown;
} | {
  /**
   The entry has an `info.yml` at the pin that refuses to parse as YAML, so it
   records no death; kept apart from the absence so the artifact can say the
   record was unreadable.
   */
  readonly kind: 'unreadable';

  /**
   The refusal's sentence: it names the entry and where the parser stopped, and
   quotes nothing the file says.
   */
  readonly refusal: string;
};

/**
 Signals an entry's `info.yml` that refuses to parse as YAML; corpus metadata
 parses upstream, so failure here indicates corruption. {@link readEntryInfoRecord}
 turns it into the unreadable record, and the sentence it carries is the one the
 warning and the artifact's finding repeat.

 @example
 ```ts
 throw new EntryInfoParseError({ entryId: 'tabby', cause: error, },);
 ```
 */
export class EntryInfoParseError extends Error {
  /**
   Declares this message safe to forward: it names the entry and states where
   the parser stopped and what it called the fault, and repeats nothing the
   file says about the person.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming the entry and stating where YAML stopped, never what
   it read.

   DOES NOT CARRY THE PARSER ERROR AS `cause`, deliberately, for the reason
   `FrontMatterParseError` does not: a `YAMLParseError` message embeds a source
   code frame, and the file records a person's dates and handles.

   @param entryId - directory name of the entry, which the operator handed in

   @param cause - underlying YAML parser error, read for position and code

   @example
   ```ts
   new EntryInfoParseError({ entryId: 'tabby', cause: error, },);
   ```
   */
  public constructor(
    {
      entryId,
      cause,
    }: {
      readonly entryId: string;
      readonly cause: Readonly<Error>;
    },
  ) {
    super(
      `The info.yml of entry ${entryId} at the pin refused to parse ${
        yamlRefusalSite({ cause, },)
      }; corpus metadata parses upstream, so this signals corruption.`,
    );
    this.name = 'EntryInfoParseError';
  }
}

/**
 Parses an entry's `info.yml`, converting the parser's refusal into one that
 names the entry and quotes nothing.

 @param entryId - directory name of the entry

 @param text - raw `info.yml` text

 @returns Parsed value; corpus shapes vary, so no schema is imposed here

 @throws {@link EntryInfoParseError} when YAML refuses to parse

 @example
 ```ts
 const info = parseEntryInfoYaml({ entryId: 'tabby', text: 'info:\n  died: 2025-02-26\n', },);
 ```
 */
function parseEntryInfoYaml(
  {
    entryId,
    text,
  }: {
    readonly entryId: string;
    readonly text: string;
  },
): unknown {
  try {
    return parseYaml(text,);
  }
  catch (error) {
    // A REFUSAL IS AN ERROR: the parser throws a YAMLError, or a ReferenceError
    // or TypeError for an anchor fault, and nothing else, so a caught value that
    // is no Error is an unexpected state `errorOrThrow` fails on loudly.
    throw new EntryInfoParseError({
      entryId,
      cause: errorOrThrow(error,),
    },);
  }
}

/**
 The clause the warning and the finding share for a `died` of a kind the feature
 cannot read: it names the key and the kind the value held and quotes none of the
 value, which is a person's metadata.

 @param kind - phrase naming the kind of the value, as `unreadableDeathKindOf` words it

 @returns Clause, without closing punctuation, which each sentence supplies

 @example
 ```ts
 const clause = unreadableDeathClause({ kind: 'a truth value', },);
 ```
 */
function unreadableDeathClause({ kind, }: { readonly kind: string; },): string {
  return `info.died is ${kind}, which is neither text nor a finite number`;
}

/**
 Reads one entry's `info.yml` at the pin and parses it, saying so when its
 `died` holds a value the feature cannot read or the file refuses to parse.

 @param pin - corpus checkout and commit the entry is read at

 @param entryId - directory name of the entry

 @param readFile - document reader at the pin

 @param l - logger the absence, a file that refuses to parse and a `died` of an unreadable kind
 are named on, left out by a command whose terminal output is pinned (the settled
 replays), which then reads all three in silence

 @returns Parsed record, absence when the entry has no `info.yml`, or
 unreadable record when the file is there and is not YAML; the last two record no death

 @throws {@link Error} when the file exists but cannot be read

 @example
 ```ts
 const record = await readEntryInfoRecord({ pin, entryId: 'tabby', readFile: readCorpusFile, l, },);
 ```
 */
export async function readEntryInfoRecord(
  {
    pin,
    entryId,
    readFile,
    l,
  }: {
    readonly pin: CorpusPin;
    readonly entryId: string;
    readonly readFile: typeof readCorpusFile;
    readonly l?: Logger;
  },
): Promise<EntryInfoRecord> {
  try {
    /**
     Raw `info.yml` text at the pin.
     */
    const text = await readFile({
      pin,
      relPath: `people/${entryId}/info.yml`,
    },);
    /**
     The file as YAML parsed it.
     */
    const info = parseEntryInfoYaml({
      entryId,
      text,
    },);
    // A `died` OF A KIND THE FEATURE CANNOT READ is said here, once per entry,
    // where the entry and the logger are known: `recordedDeathOf` runs several
    // times per preparation and in replays with pinned output, and leaves the
    // value unrecorded either way.
    for (const kind of unreadableDeathKindOf({ entryInfo: info, },)) {
      l?.warn(
        `ENTRY INFO entry=${entryId}: ${unreadableDeathClause({ kind, },)}, so no death is recorded for it`,
      );
    }
    return {
      kind: 'read',
      info,
    };
  }
  catch (error) {
    if (error instanceof EntryInfoParseError) {
      /**
       The refusal's sentence, which the class declares safe to forward.
       */
      const refusal = refusalText({ error, },);
      l?.warn(
        `ENTRY INFO entry=${entryId}: ${refusal} No death is recorded for the entry, and it goes on as one whose `
          + 'info.yml records none',
      );
      return {
        kind: 'unreadable',
        refusal,
      };
    }
    if (!isMissingCorpusObject(error,))
      throw error;
    l?.info(`ENTRY INFO entry=${entryId}: no info.yml at the pin (${refusalText({ error, },)}), so no death is recorded for it`,);
    return { kind: 'absent', };
  }
}

/**
 The findings the artifact records for what the entry's `info.yml` came to, so a
 later reader can tell an entry that records no death from one whose record was
 unreadable, or whose `died` held a value of a kind the feature cannot read. They
 ride the preparation's alignment findings, the channel every artifact already
 records.

 @param entryInfo - what the pass read of the entry's `info.yml`

 @returns One finding for an unreadable record, one per unreadable kind of `died`
 in a parsed record (naming the key and the kind and quoting no value), none for
 an absence or a record whose `died` is recorded or blank

 @example
 ```ts
 const findings = entryInfoFindingsOf({ entryInfo: { kind: 'absent', }, },);
 ```
 */
export function entryInfoFindingsOf(
  { entryInfo, }: { readonly entryInfo: EntryInfoRecord; },
): readonly string[] {
  if (entryInfo.kind === 'absent')
    return [];
  if (entryInfo.kind === 'unreadable')
    return [`entry info: ${entryInfo.refusal} No death is recorded for the entry.`,];
  return unreadableDeathKindOf({ entryInfo: entryInfo.info, },)
    .map(function findingOfKind(kind,): string {
      return `entry info: ${unreadableDeathClause({ kind, },)}. No death is recorded for the entry.`;
    },);
}

//endregion Entry info record
