import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  isDecimalText,
  isNegativeWholeNumberText,
  isWholeNumberText,
  WHOLE_NUMBER_RULE,
} from '@monochromatic-dev/translation-repair-markup/ts';
import type { FlagValue, } from './command-line-types.ts';

//region Command flags
// The readers of flag VALUES the package's command lines share, so every
// probe and audit answers a mistyped value the same way (ledger B73). Which
// flags a command reads, and whether each was written once with a value, is
// read before any of these runs (`command-line.ts`, ledger B75), so each
// reader here is handed a value already known to be written and not empty,
// or that the flag was not written at all.
//
// THE PROBES EACH HAD THEIR OWN. The rendering audit's refused a flag written
// last and a cap that was no number; the fidelity and coverage probes read
// both as the default, truncated `4.9` to 4 and took `-3` as a cap, so a
// mistyped flag spent trials nobody asked for and said nothing. One reader
// now holds the refusals, and each probe names only its flags and defaults.
//
// `askedAmong` holds the ids a line names to what its runner can reach, so an
// id naming nothing refuses the line rather than narrowing the run to nothing
// (ledger B76).

/**
 Separator between the ids one flag names.
 */
const ID_SEPARATOR = ',';

/**
 Reads the text written after a flag, or a default when it was not written.

 ONLY AN UNWRITTEN FLAG TAKES THE DEFAULT. Two score reports read a flag
 written last, or followed by an empty argument, as not written and scored
 the default sheet, which is the file the override was typed to replace;
 the command-line reader now refuses those before this runs (ledger B75).

 @param asked - what the flag carried, or that nobody wrote it

 @param unwritten - text to use when the flag was not written

 @returns Text written, or `unwritten`

 @example
 ```ts
 const sheet = writtenOr({ asked: line.flag('sheet',), unwritten: defaultSheet, },);
 ```
 */
export function writtenOr(
  {
    asked,
    unwritten,
  }: {
    readonly asked: FlagValue;
    readonly unwritten: string;
  },
): string {
  return (asked.kind === 'written') ? asked.value : unwritten;
}

/**
 Reads a whole number written after a flag, zero included.

 DIGITS ONLY, by the package's one count rule: `Number` read `fourty` as no
 number, which two probes then replaced with their default, and `4.9`,
 `1e1`, `0x4` and `+4` as numbers nobody typed (ledger B73). A MINUS SIGN
 BEFORE DIGITS is answered as a number below zero, since that is what the
 person typed, and the refusal says what leaving the flag off would do.

 @param asked - what the flag carried, or that nobody wrote it

 @param unwritten - number to use when the flag was not written, which may be
 a sentinel the caller reads as "no limit"

 @param leaveOffTo - what leaving the flag off does, completing "leave it off
 to ..." in the refusal of a number below zero

 @returns Number written, or `unwritten` when the flag was not

 @throws StatedRefusalError when the flag was written with a number below
 zero, or with anything that is not a whole number written in digits

 @example
 ```ts
 const cap = wholeNumberFlag({ asked: line.flag('cap',), unwritten: 16, leaveOffTo: 'run the default of 16 trials', },);
 ```
 */
export function wholeNumberFlag(
  {
    asked,
    unwritten,
    leaveOffTo,
  }: {
    readonly asked: FlagValue;
    readonly unwritten: number;
    readonly leaveOffTo: string;
  },
): number {
  if (asked.kind === 'unwritten')
    return unwritten;

  if (isNegativeWholeNumberText({ text: asked.value, },))
    throw new StatedRefusalError({
      says: `${asked.flag} cannot be below zero, and ${asked.value} is; leave it off to ${leaveOffTo}`,
    },);

  if (!isWholeNumberText({ text: asked.value, },))
    throw new StatedRefusalError({
      says: `${asked.flag} needs ${WHOLE_NUMBER_RULE}, and ${JSON.stringify(asked.value,)} is not one`,
    },);

  return Number(asked.value,);
}

/**
 Reads an amount of US dollars written after a flag.

 A PLAIN DECIMAL, as an operator writes dollars, by the package's one decimal rule: a sign, a leading or trailing
 point, an exponent, a radix, a space or a comma is refused rather than read as an amount nobody wrote that way, and a
 digit run past the largest double, which reads as infinity, is refused with them (ledger B73).

 @param asked - what the flag carried, or that nobody wrote it

 @param unwritten - amount to use when the flag was not written

 @returns Amount written, or `unwritten` when the flag was not

 @throws StatedRefusalError when the flag was written with anything but a plain amount of dollars

 @example
 ```ts
 const maxUsd = usdFlag({ asked: line.flag('max-usd',), unwritten: 2, },);
 ```
 */
export function usdFlag(
  {
    asked,
    unwritten,
  }: {
    readonly asked: FlagValue;
    readonly unwritten: number;
  },
): number {
  if (asked.kind === 'unwritten')
    return unwritten;

  /**
   The text read as a number, which only a plain, finite decimal may be.
   */
  const usd = Number(asked.value,);
  if ((!isDecimalText({ text: asked.value, },)) || (!Number.isFinite(usd,)))
    throw new StatedRefusalError({
      says: `${asked.flag} needs a plain amount of dollars written in digits with at most one decimal point, and ${
        JSON.stringify(asked.value,)
      } is not one`,
    },);

  return usd;
}

/**
 Reads the ids written after a flag, comma separated.

 SPACE AROUND AN ID IS DROPPED, and so is the gap a stray comma leaves: no id
 any of these flags names holds either, so `tabby, ginger` can only mean the
 two ids it shows. The entry filter trimmed and the candidate and provider
 readers did not, so one spelling named two entries to the pass and an
 unknown model to the probes (ledger B75).

 @param asked - what the flag carried, or that nobody wrote it

 @param naming - what one id names, for the refusal: `entry id`,
 `seatable id`, `provider`

 @returns Ids named, in the order written; empty when the flag was not
 written, which callers read as no restriction

 @throws StatedRefusalError when the flag was written with separators and
 space naming no id, which would otherwise read as no restriction one line
 later

 @example
 ```ts
 const onlyIds = idListFlag({ asked: line.flag('only',), naming: 'entry id', },);
 ```
 */
export function idListFlag(
  {
    asked,
    naming,
  }: {
    readonly asked: FlagValue;
    readonly naming: string;
  },
): readonly string[] {
  if (asked.kind === 'unwritten')
    return [];

  /**
   Ids the text actually names.
   */
  const named = asked.value
    .split(ID_SEPARATOR,)
    .map(function trimmed(id,): string {
      return id.trim();
    },)
    .filter(function isNamed(id,): boolean {
      return id !== '';
    },);

  if (named.length === 0)
    throw new StatedRefusalError({
      says: `${asked.flag} needs at least one ${naming}, and ${JSON.stringify(asked.value,)} names none`,
    },);

  return named;
}

/**
 The ids a runner works over: those its line asks for, among those it can
 reach.

 ONE FILTER FOR EVERY RUNNER that narrows its walk by id. The pass, the two
 coverage probes and the settled audit each wrote their own, the same
 predicate in each, and each dropped an id it could not reach without a
 word: an `--only` naming only a stray ran over nothing, and a stray beside a
 real entry ran the real one alone (ledger B76).

 EVERY ID ASKED FOR MUST BE REACHABLE, so a mistyped id refuses the whole
 line, all strays named at once, before anything is spent on the rest.

 @param asked - ids the line names, in the order written; empty when it
 names none, which reads as no restriction

 @param known - ids the runner can reach, in the order it walks them

 @param source - what asked for the ids, as the person typed it: `--only`,
 or the runner's name for ids written by position

 @param within - what `known` is drawn from, completing "which ... does not
 hold": `the corpus at the pin`, `the settled archive`

 @returns The known ids asked for, in `known`'s order, so a filtered run
 walks its entries as an unfiltered one does; every known id when none was
 asked

 @throws StatedRefusalError naming, each once, in the order written and
 quoted, every id asked for that `known` lacks

 @example
 ```ts
 const entryIds = askedAmong({ asked: onlyIds, known: people, source: '--only', within: 'the corpus at the pin', },);
 ```
 */
export function askedAmong(
  {
    asked,
    known,
    source,
    within,
  }: {
    readonly asked: readonly string[];
    readonly known: readonly string[];
    readonly source: string;
    readonly within: string;
  },
): readonly string[] {
  if (asked.length === 0)
    return known;

  /**
   Ids the runner can reach, for lookup.
   */
  const reachable = new Set(known,);

  /**
   Ids asked for that the runner cannot reach, each once, in the order
   written.
   */
  const strays = [...new Set(asked,),].filter(function isStray(id,): boolean {
    return !reachable.has(id,);
  },);
  if (strays.length > 0)
    throw new StatedRefusalError({
      says: `${source} asks for ${
        strays
          .map(function quoted(id,): string {
            return JSON.stringify(id,);
          },)
          .join(', ',)
      }, which ${within} does not hold`,
    },);

  /**
   Ids asked for, for lookup.
   */
  const wanted = new Set(asked,);
  return known.filter(function isAsked(id,): boolean {
    return wanted.has(id,);
  },);
}

//endregion Command flags
