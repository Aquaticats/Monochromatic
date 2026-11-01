import { normalizePunctuation, } from '@monochromatic-dev/translation-repair-typography/ts';
import { carriesWord, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Refusal detection
// The models refuse benign requests often enough that refusal handling is a
// first-class outcome, handled reactively: a refusal-shaped reply reroutes to
// another vendor family and lands on the scorecard. Detection is deterministic
// and deliberately shallow: refusals lead with the apology, so only the opening
// window is scanned, keeping legitimate content that quotes refusal phrasing
// (a critic citing translated dialogue) from being misclassified.
//
// A MARKER IS MATCHED AS WORDS, across typographic quotes (ledger B23). A raw
// substring found "as an ai" inside "as an aide", and a marker written with a
// straight apostrophe missed "I can’t help", which is how models often write
// it. The window's quotes are folded to ASCII (`quote-normalize.ts`) and each
// marker's Latin edges stand at word boundaries (`word-bounds.ts`).

/**
 Character count of the opening window scanned for refusal markers.
 Refusals lead with the apology;
 markers deeper than this are treated as quoted content, not refusal.
 */
export const REFUSAL_SCAN_WINDOW = 400;

/**
 Lowercase markers whose presence in the opening window classifies a reply as
 refusal-shaped.
 Grown from observed failures; scorecard records which marker fired so the list
 stays evidence-driven.

 @example
 ```ts
 REFUSAL_MARKERS.includes("i can't assist",);
 ```
 */
export const REFUSAL_MARKERS: readonly string[] = [
  "i can't help",
  'i cannot help',
  "i can't assist",
  'i cannot assist',
  "i can't comply",
  'i cannot comply',
  "i won't be able to",
  'i am unable to',
  "i'm unable to",
  "i'm sorry, but",
  'i am sorry, but',
  'as an ai',
  'cannot fulfill this request',
  "can't fulfill this request",
  'against my guidelines',
  '我不能协助',
  '我无法协助',
  '我不能帮助',
  '我无法帮助',
  '无法满足这个请求',
];

/**
 Classification of one model reply's opening window.

 @example
 ```ts
 const scan: RefusalScan = { refusalShaped: true, marker: 'as an ai', };
 ```
 */
export type RefusalScan =
  | {
    /**
     Reply opens like a refusal.
     */
    readonly refusalShaped: true;

    /**
     First marker that fired; feeds the scorecard.
     */
    readonly marker: string;
  }
    | {
    /**
     No marker fired in the opening window.
     */
      readonly refusalShaped: false;
    };

/**
 Scans one reply's opening window for refusal markers.
 Case-insensitive; first marker in list order wins so results are deterministic.

 @param text - full model reply

 @returns Whether the opening reads as refusal, and which marker fired

 @example
 ```ts
 const scan = detectRefusalShape({ text: reply, },);
 if (scan.refusalShaped) reroute(scan.marker,);
 ```
 */
export function detectRefusalShape(
  { text, }: { readonly text: string; },
): RefusalScan {
  /**
   Lowercased opening window with its quotes folded to ASCII; markers are
   stored lowercase with straight apostrophes.
   */
  const opening = normalizePunctuation({
    text: text.slice(
      0,
      REFUSAL_SCAN_WINDOW,
    ),
  },)
    .toLowerCase();

  /**
   First marker standing as words in the opening window, when any.
   */
  const marker = REFUSAL_MARKERS.find(function firesIn(candidate,) {
    return carriesWord({
      text: opening,
      needle: candidate,
      end: 'word',
    },);
  },);

  if (marker === undefined)
    return { refusalShaped: false, };

  return {
    refusalShaped: true,
    marker,
  };
}

//endregion Refusal detection
