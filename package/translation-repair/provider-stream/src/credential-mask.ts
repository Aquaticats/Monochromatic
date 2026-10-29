import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { decodedFormsOf, } from './credential-decoded-forms.ts';
import {
  distinctNeedles,
  type Needle,
  needlesOf,
} from './credential-needles.ts';
import {
  findNeedles,
  type NeedleFind,
} from './credential-search.ts';
import {
  decodedViewsOf,
  type TextView,
} from './credential-views.ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Credential mask
// Replaces every copy of a credential a request carried inside the text a
// provider answered with, so the credential never leaves the transport.
//
// WHY AT THE TRANSPORT. A provider may echo a sent credential in a reply body,
// an authentication failure's body being the plausible place, and everything
// above the transport reads that body: the status failure's message and
// excerpt, the raw reply preview a probe prints, a malformed-completion
// message, a cached reply. One mask where the body is assembled closes all of
// them at once.
//
// NO REGULAR EXPRESSION IS BUILT FROM A CREDENTIAL. A key may hold characters a
// pattern reads as syntax, and building the pattern from it is the shape that
// fails on exactly the keys that need masking. Every spelling of every
// credential is found in one pass over the text (`credential-search.ts`), so a
// long body costs its length and the number of copies, never its length times
// the number of spellings.
//
// A CREDENTIAL IS MASKED IN EVERY SPELLING A MECHANICAL DECODER RECOVERS: as it
// stands, as a JSON string writes it (`credential-needles.ts`), percent-encoded
// or JSON-escaped in any mix and nesting (`credential-views.ts`, which reads the
// text as each decoder would and maps a find back to where it stands), and in
// base64 or base64url at any alignment (`credential-needles.ts`). A header sent
// encoded and echoed decoded is listed by `credentialsOfHeaders`
// (`credential-decoded-forms.ts`). Out of reach by rule: a credential a proxy
// split by a line fold or a soft hyphen, and a spelling that no decoder named
// here recovers (a cipher, a hash, a case-folded key, the key written in hex or
// as HTML entities, or in base64 of bytes that are no UTF-8 of it).
//
// THE CLOSED LIST OF CREDENTIAL HEADERS is the one place a new client's
// credential header must be written down: a header named here has its value
// masked, a header not named here is taken for no credential (`content-type`
// and `anthropic-version` are values every reply would otherwise be rewritten
// by).

/**
 Text written where a credential was masked.

 FREE OF `"`, `\` AND EVERY CHARACTER A JSON STRING ESCAPES, so a credential
 standing inside a JSON string leaves that string as valid as it was.
 */
export const CREDENTIAL_MARKER = '[masked: credential sent with this request]';

/**
 Fewest UTF-16 units a secret must hold to be masked.

 A CHOICE, NOT A MEASUREMENT. The stand-in key of this package's cases holds
 16 units; a value shorter than 12 is, by inference, no key a provider
 issues, and masking it would rewrite ordinary words of a reply. A credential
 below the bound is left alone.
 */
export const MINIMUM_CREDENTIAL_UNITS = 12;

/**
 Fewest leading units of a credential that a text ending in them is read as a
 credential cut short, and masked.

 A CHOICE, NOT A MEASUREMENT. A head shorter than this is, by inference, too
 little of a 12-unit secret to be worth the ordinary words a shorter rule
 would rewrite at the end of an excerpt (a text ending in `s` would match
 every key that begins with `s`); the cost is that the first three units of a
 credential may show where a stream was cut inside it.
 */
const MINIMUM_HEAD_UNITS = 4;

/**
 Lower-case names of the request headers that carry a credential: an
 authorization value, an API key and a session token.
 */
const CREDENTIAL_HEADER_NAMES: ReadonlySet<string> = new Set([
  'authorization',
  'proxy-authorization',
  'x-api-key',
  'api-key',
  'x-amz-security-token',
],);

/**
 Span of the text one credential occupies.
 */
type Span = {
  /**
   Index of its first unit.
   */
  readonly start: number;

  /**
   Index one past its last unit.
   */
  readonly end: number;
};

/**
 Lists what a request's headers carry as credential: for each credential
 header, its whole value and, where the value is a scheme word and a token, the
 token alone, since a provider may echo either, and the forms a decoder
 reads out of the token (a `Basic` pair and each side of it, an escaped token
 resolved), since a provider may echo those instead.

 @param headers - request headers, names in any case

 @returns Whole values, tokens and decoded forms that hold at least the minimum
 units; empty where the request carried none

 @example
 ```ts
 credentialsOfHeaders({ headers: { Authorization: 'Bearer whisker-key-7421', }, },);
 // => ['Bearer whisker-key-7421', 'whisker-key-7421']
 ```
 */
export function credentialsOfHeaders(
  { headers, }: { readonly headers: Readonly<Record<string, string>>; },
): readonly string[] {
  return Object.entries(headers,)
    .flatMap(function credentialsOfHeader([name, value,]: ReadonlyEntry<string, string>,): readonly string[] {
      if (!CREDENTIAL_HEADER_NAMES.has(name.toLowerCase(),))
        return [];

      /**
       Value as the wire carries it, without the whitespace around it.
       */
      const whole = value.trim();

      /**
       Where the scheme word ends, or nothing when the value has none.
       */
      const gap = whole.indexOf(' ',);

      /**
       The secret: the token after the scheme word, or the whole value.
       */
      const secret = (gap === (-1))
        ? whole
        : whole.slice(gap + 1,)
          .trimStart();
      if (secret.length < MINIMUM_CREDENTIAL_UNITS)
        return [];

      return [
        ...new Set([
          whole,
          secret,
          ...decodedFormsOf({ secret, },),
        ],),
      ].filter(function longEnough(form,): boolean {
        return form.length >= MINIMUM_CREDENTIAL_UNITS;
      },);
    },);
}

/**
 Lists the needles of the credentials long enough to mask, each once.

 @param credentials - secrets the request carried

 @returns Needles of every spelling of every credential

 @example
 ```ts
 needlesOfAll({ credentials: ['whisker-key-7421',], },);
 ```
 */
function needlesOfAll({ credentials, }: { readonly credentials: readonly string[]; },): readonly Needle[] {
  /**
   Every needle of every credential long enough, some of them repeated.
   */
  const every = [...new Set(credentials,),]
    .filter(function longEnough(credential,): boolean {
      return credential.length >= MINIMUM_CREDENTIAL_UNITS;
    },)
    .flatMap(function needlesOfOne(credential,): readonly Needle[] {
      return needlesOf({ credential, },);
    },);
  return distinctNeedles({ needles: every, },);
}

/**
 Units that carry bits of a base64 text in either alphabet. Padding is not
 among them: `=` only ever ends a run, so the one before a find is no part of
 the run the find is in, and none holds a bit of a credential beside it.
 */
const BASE64_UNITS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/-_';

/**
 Most `=` units that pad a base64 text.
 */
const MOST_PADDING = 2;

/**
 Counts the units beside a find that a base64 find reaches over: up to a number
 of them, each of the base64 units a run is written with.

 @param text - text the find is in

 @param from - index of the first unit counted

 @param direction - one to count forward, minus one to count back

 @param most - most units to count

 @returns How many consecutive base64 units there are beside the find, up to most

 @example
 ```ts
 base64UnitsBeside({ text: 'xa2l0O', from: 1, direction: 1, most: 1, },); // 1
 ```
 */
function base64UnitsBeside(
  {
    text,
    from,
    direction,
    most,
  }: {
    readonly text: string;
    readonly from: number;
    readonly direction: 1 | -1;
    readonly most: number;
  },
): number {
  /**
   Units counted so far.
   */
  const counted = { units: 0, };
  while (
    (counted.units < most)
      && ((from + (direction * counted.units)) >= 0)
      && ((from + (direction * counted.units)) < text.length)
      && BASE64_UNITS.includes(text.charAt(from + (direction * counted.units),),)
  )
    counted.units += 1;

  return counted.units;
}

/**
 Widens a find over the characters beside it that hold bits of the credential.

 THE WIDENING STOPS AT A UNIT A BASE64 RUN IS NOT WRITTEN WITH, so a find that
 stands against a quote, a space or the text's edge hides nothing of what lies
 beyond it: the characters that share bits with the credential are in the run,
 or the find was not a base64 run at that alignment.

 PADDING GOES WITH THE FIND ONLY AFTER A CHARACTER THE CREDENTIAL'S BITS END
 INSIDE, the one place a run the credential ends can be padded; after a find
 whose credential ends on a whole group of three bytes, an `=` pads no run the
 credential ends and stays.

 @param text - text the find is in

 @param find - find

 @param needle - needle found

 @returns Find's span, widened and cut to the text

 @example
 ```ts
 reachOf({ text, find, needle, },);
 ```
 */
function reachOf(
  {
    text,
    find,
    needle,
  }: {
    readonly text: string;
    readonly find: NeedleFind;
    readonly needle: Needle;
  },
): Span {
  /**
   Base64 units after the find that hold bits of the credential.
   */
  const tail = base64UnitsBeside({
    text,
    from: find.end,
    direction: 1,
    most: needle.after,
  },);

  /**
   End of the find with its tail.
   */
  const reached = find.end + tail;

  /**
   `=` units after the tail, which pad the run only where the tail holds the
   credential's last bits.
   */
  const padding = (tail > 0)
    ? paddingAt({
      text,
      from: reached,
    },)
    : 0;
  return {
    start: find.start - base64UnitsBeside({
      text,
      from: find.start - 1,
      direction: -1,
      most: needle.before,
    },),
    end: reached + padding,
  };
}

/**
 Counts the `=` padding units that start at a position.

 @param text - text the padding is in

 @param from - index of the first unit counted

 @returns Units of padding, at most two

 @example
 ```ts
 paddingAt({ text: 'abc==d', from: 3, },); // 2
 ```
 */
function paddingAt({
  text,
  from,
}: {
  readonly text: string;
  readonly from: number
},): number {
  /**
   Units counted so far.
   */
  const counted = { units: 0, };
  while ((counted.units < MOST_PADDING) && (text.charAt(from + counted.units,) === '='))
    counted.units += 1;

  return counted.units;
}

/**
 Finds the spans of every needle in a text, as the text itself and as each
 decoded reading of it, each span in the units of the original text.

 @param text - reply text

 @param needles - needles to find

 @returns Spans in the original text, unordered

 @example
 ```ts
 spansOfNeedles({ text, needles, },);
 ```
 */
function spansOfNeedles(
  {
    text,
    needles,
  }: {
    readonly text: string;
    readonly needles: readonly Needle[];
  },
): readonly Span[] {
  /**
   Texts of the needles, in the order `findNeedles` numbers them.
   */
  const texts = needles.map(function textOf(needle,): string {
    return needle.text;
  },);
  /**
   Every unit some needle holds.
   */
  const alphabet = new Set(texts.flatMap(function unitsOf(needle,): readonly string[] {
    return needle.split('',);
  },),);
  return [
    ...findNeedles({
      text,
      needles: texts,
    },)
      .map(function inOriginal(find,): Span {
        return reachOf({
          text,
          find,
          needle: nonNullishOrThrow(needles[find.needle],),
        },);
      },),
    ...decodedViewsOf({
      text,
      wanted: function inSomeNeedle({ unit, }: { readonly unit: string; },): boolean {
        return alphabet.has(unit,);
      },
    },)
      .flatMap(function inView(view: TextView,): readonly Span[] {
        return findNeedles({
          text: view.text,
          needles: texts,
        },)
          .map(function inOriginal(find,): Span {
            /**
             The find widened in the decoded text.
             */
            const reach = reachOf({
              text: view.text,
              find,
              needle: nonNullishOrThrow(needles[find.needle],),
            },);
            return {
              start: nonNullishOrThrow(view.starts[reach.start],),
              end: nonNullishOrThrow(view.ends[reach.end - 1],),
            };
          },);
      },),
  ];
}

/**
 Replaces every copy of each credential in a text by the marker.

 EVERY CREDENTIAL IS FOUND IN THE ORIGINAL TEXT, in every spelling a decoder
 recovers (see the module comment), and overlapping or touching finds become
 one marker, so a whole header value and the token inside it leave one marker,
 and a marker's own wording is never searched. A credential found inside a
 base64 run hides, with it, the one neighbouring character at each edge that
 holds bits of it and, after a character its bits end inside, the run's
 padding; that is the most it can hide of the text beside it.
 Matching is by UTF-16 unit.

 @param text - reply text, of any shape: JSON, event frames or prose

 @param credentials - secrets the request carried; one holding fewer than the
 minimum units, or none, is left unmasked

 @returns Text with every copy masked, the very same string when none was found

 @example
 ```ts
 maskCredentials({ text: 'bad key whisker-key-7421', credentials: ['whisker-key-7421',], },);
 ```
 */
export function maskCredentials(
  {
    text,
    credentials,
  }: {
    readonly text: string;
    readonly credentials: readonly string[];
  },
): string {
  /**
   Spans of every spelling of every credential, in the original text.
   */
  const found = spansOfNeedles({
    text,
    needles: needlesOfAll({ credentials, },),
  },)
    .toSorted(function byStart(
      left,
      right,
    ): number {
      return left.start - right.start;
    },);
  if (found.length === 0)
    return text;

  /**
   Pieces of the result, in order.
   */
  const parts: string[] = [];
  /**
   Where the unmasked text resumes, and the end of the last masked stretch (nothing before the first).
   */
  const cursor = {
    resume: 0,
    maskedTo: -1,
  };
  for (const span of found) {
    if (span.start <= cursor.maskedTo) {
      cursor.maskedTo = Math.max(
        cursor.maskedTo,
        span.end,
      );
      cursor.resume = cursor.maskedTo;
      continue;
    }
    parts.push(
      text.slice(
        cursor.resume,
        span.start,
      ),
      CREDENTIAL_MARKER,
    );
    cursor.resume = span.end;
    cursor.maskedTo = span.end;
  }
  parts.push(text.slice(cursor.resume,),);
  return parts.join('',);
}

/**
 Length of the longest ending of a text that is the opening of one of the
 needles, a credential the text was cut inside.

 @param text - text that may end inside a credential

 @param needles - spellings of the credentials to look for

 @returns Units of the ending, zero when none is as long as the minimum head

 @example
 ```ts
 trailingHeadUnits({ text: 'says whisker-k', needles: ['whisker-key-7421',], },); // 9
 ```
 */
function trailingHeadUnits(
  {
    text,
    needles,
  }: {
    readonly text: string;
    readonly needles: readonly string[];
  },
): number {
  return Math.max(
    0,
    ...needles.map(function headOf(needle,): number {
      for (let units = Math.min(
        needle.length - 1,
        text.length,
      ); units >= MINIMUM_HEAD_UNITS; units -= 1) {
        if (text.endsWith(needle.slice(
          0,
          units,
        ),))
          return units;
      }
      return 0;
    },),
  );
}

/**
 Replaces every copy of each credential in a text that may have been cut
 short, and the opening of a credential the text ends inside, by the marker.

 FOR TEXT CUT OFF BEFORE ITS END, an excerpt of a stream that was torn down:
 the credential the stream was saying when it stopped has no copy to find, and
 its first units would show. The mask is applied to the whole text first and
 the ending read after it, so a caller cuts its excerpt from what this
 returns and never masks an excerpt already cut. The ending is read for the
 spellings a text can be cut inside, which are the written ones, the JSON
 ones and the base64 ones; a cut inside a percent escape shows at most the
 escapes before it.

 @param text - generated text so far, of any shape

 @param credentials - secrets the request carried; one holding fewer than the
 minimum units, or none, is left unmasked

 @returns Text with every copy masked and any ending that is at least the
 minimum head of a credential masked too, the very same string when neither
 was found

 @example
 ```ts
 maskCredentialsInCutText({ text: 'says whisker-k', credentials: ['whisker-key-7421',], },);
 ```
 */
export function maskCredentialsInCutText(
  {
    text,
    credentials,
  }: {
    readonly text: string;
    readonly credentials: readonly string[];
  },
): string {
  /**
   Text with every whole copy masked.
   */
  const masked = maskCredentials({
    text,
    credentials,
  },);

  /**
   Units of the ending that is a credential's opening.
   */
  const head = trailingHeadUnits({
    text: masked,
    needles: needlesOfAll({ credentials, },)
      .map(function textOf(needle,): string {
        return needle.text;
      },),
  },);
  if (head === 0)
    return masked;

  return masked.slice(
    0,
    masked.length - head,
  ) + CREDENTIAL_MARKER;
}

//endregion Credential mask
