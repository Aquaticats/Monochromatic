import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  anchorsInsideForeignRegion,
  type TargetRegion,
} from './coverage-foreign-region.ts';
import type {
  CoverageDegree,
  CoverageReportWire,
} from './coverage-wire.ts';
import type {
  AnchorTarget,
  SpanAnchor,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { locateQuote, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { HeardVoice, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Coverage verdict
// What a roster's coverage replies add up to, and what they are not allowed to
// add up to.
//
// THE TWO ANSWERS ARE NOT SYMMETRIC. A voice claiming the translation carries a
// passage must point at the English that carries it, and that pointing is
// checkable. A voice claiming nothing carries it cannot exhibit anything,
// because absence has no witness. So the verdict says which of the two it
// reached rather than flattening both into a boolean.
//
// WHAT AN ANCHORED QUOTE PROVES IS PROVENANCE, NOT CORRESPONDENCE, and that
// distinction is measured rather than theoretical: on one corpus section two of
// six voices claimed coverage and quoted a real sentence belonging to a
// different passage of the same document. Locating a quote proves those words
// occur; it does not prove they render the passage that was asked about. The
// tallies here are therefore evidence, not proof, and a second
// field would be needed to close the gap.
//
// AN UNANCHORABLE CLAIM IS DROPPED RATHER THAN BELIEVED, the same rule the
// critic stage has used since quote anchoring landed, AND IT IS NOT A VOTE FOR
// ABSENCE. Treating it as agreement with "nothing carries this" would turn a
// bad quote into evidence for inserting text, which is the expensive direction.
//
// THE THRESHOLD IS THE ROSTER, NOT THE VOICES HEARD, which is the correction a
// review made after the first measurement. Counting a majority of those who
// answered means silence LOWERS the bar: one voice heard saying it found
// nothing decided absence, with five models lost and quorum unmet. Silence is
// then more dangerous than a fabricated quote, which is backwards. A verdict now
// needs a majority of every model asked, and an unmet quorum is inconclusive
// whatever the answers say.
//
// SINCE THE FAN-OUT WINDOW OF 2026-09-09 a round asks quorum plus one seat
// first (`stage-fanout-window.ts`), so `asked` is what the gather sent the
// question to, not the bench: a seat the window spared was never silent, and
// the majority is taken over the seats that were actually asked.

/**
 What a roster concluded about one passage.

 @example
 ```ts
 const verdict: CoverageVerdict = { kind: 'carried', ... };
 ```
 */
export type CoverageVerdict = {
  /**
   `carried` when a majority of the seats ASKED anchored full coverage;
   `partly-carried` when a majority anchored coverage but not all of it, which
   still forbids inserting the passage whole;
   `absent` when a majority found none;
   `split` when neither side reached a majority of the seats asked;
   `inconclusive` when too few models answered to decide anything.
   */
  readonly kind: 'carried' | 'partly-carried' | 'absent' | 'split' | 'inconclusive';

  /**
   Voices that anchored FULL coverage in the document.
   */
  readonly anchoredFull: number;

  /**
   Voices that anchored partial coverage.
   */
  readonly anchoredPartial: number;

  /**
   Voices reporting nothing renders the passage.
   */
  readonly absent: number;

  /**
   Voices claiming coverage whose quote is not in the document.
   */
  readonly unanchored: number;

  /**
   Partial claims whose quote sits inside a target region the pairing
   assigned to another source slice (class fifty-one, shi_Yumiaoya4): that
   English renders a different original, so the claim is dropped, neither
   coverage nor a vote for absence.
   */
  readonly misattributed: number;

  /**
   Voices heard at all.
   */
  readonly heard: number;

  /**
   Models asked and reachable, which is what the majority is taken over.
   */
  readonly asked: number;

  /**
   DOCUMENT'S OWN TEXT for each anchored region, in roster order, so a reader
   can find every one of these by searching the translation.

   It used to hold the submitted quote, which reads the same until a fallback
   pass does the matching: a quote anchored across a soft wrap, or through
   normalized punctuation, is by definition text the document does not hold
   literally, so the field contradicted the promise made here.
   */
  readonly evidence: readonly string[];

  /**
   Quotes that were claimed and not found, kept because a near miss and an
   invention are different failures and the counts alone cannot tell them
   apart.
   */
  readonly unanchoredQuotes: readonly string[];

  /**
   Quotes of the partial claims dropped for pointing at another slice's
   rendering, kept so a reading can tell a misattribution from an invention.
   */
  readonly misattributedQuotes: readonly string[];
};

/**
 One voice's claim, once its quote has been looked for.
 */
type WeighedVoice = {
  /**
   How much coverage it claimed.
   */
  readonly degree: CoverageDegree;

  /**
   Whether its quote was found in the document outside every foreign region.
   */
  readonly anchored: boolean;

  /**
   Whether its quote was found, but inside a region paired to another source
   slice, which a partial claim cannot draw coverage from.
   */
  readonly misattributed: boolean;

  /**
   Quote it offered, empty when it claimed no coverage.
   */
  readonly quote: string;

  /**
   Document's OWN bytes for the region its quote matched, empty when nothing
   matched. Differs from `quote` whenever a fallback pass did the matching,
   which is exactly when the submitted text does not occur in the document.
   */
  readonly matched: string;
};

/**
 Reads back the document's own text for a located region.

 WHY NOT THE SUBMITTED QUOTE: a match may come from a fallback pass, which is
 exactly when the submitted text does NOT occur in the document, so storing it
 as evidence produces a string a reader cannot find. Anchors span from the
 first to the last, covering any inter-block bytes between them, so the result
 is a literal substring of the document rather than a reassembly.

 @param document - side the region was located in

 @param anchors - located spans in document order, never empty by their type
 (ledger T8: a located quote's anchors are a non-empty list, so the throw
 that stood here for none could not run, and went with its class)

 @returns Document text from first anchor start to last anchor end

 @example
 ```ts
 const matched = matchedRegion({ document, anchors, },);
 ```
 */
function matchedRegion(
  {
    document,
    anchors,
  }: {
    readonly document: AnchorTarget;
    readonly anchors: readonly [
      SpanAnchor,
      ...SpanAnchor[]
    ];
  },
): string {
  /**
   Earliest span, whose start opens the region.
   */
  const [first,] = anchors;

  /**
   Latest span, whose end closes it; the list holds at least the first.
   */
  const last = nonNullishOrThrow(anchors.at(-1,),);
  return document.text
    .slice(
    first.startOffset,
    last.endOffset,
    );
}

/**
 Looks for one voice's quote in the translation it describes.

 @param voice - heard coverage reply

 @param document - translation the quote should occur in

 @param foreignRegions - target regions paired to other source slices

 @param original - passage whose comments are the original's own on the
 sheet the voice read, so the quote is located in the translation as that sheet
 showed it; absent when the sheet cut nothing

 @returns That claim with its anchoring resolved

 @example
 ```ts
 const weighed = weighVoice({ voice, document, foreignRegions: [], },);
 ```
 */
function weighVoice(
  {
    voice,
    document,
    foreignRegions,
    original,
  }: {
    readonly voice: HeardVoice<CoverageReportWire>;
    readonly document: AnchorTarget;
    readonly foreignRegions: readonly TargetRegion[];
    readonly original?: string;
  },
): WeighedVoice {
  /**
   How much of the passage this voice says the translation carries.
   */
  const degree = voice.value
    .coverage;
  if (degree === 'none') {
    return {
      degree,
      anchored: false,
      misattributed: false,
      quote: '',
      matched: '',
    };
  }

  /**
   Where its quote sits in the translation, if anywhere.
   */
  const located = locateQuote({
    document,
    side: 'target',
    quote: voice.value
      .copiedSpan,
    ...((original === undefined) ? {} : { original, }),
  },);
  if (!located.located) {
    return {
      degree,
      anchored: false,
      misattributed: false,
      quote: voice.value
        .copiedSpan,
      matched: '',
    };
  }

  /**
   Whether a partial claim points at another slice's rendering (class
   fifty-one); a full claim there reports an original merged into its
   neighbour and is kept.
   */
  const misattributed = (degree === 'partial') && anchorsInsideForeignRegion({
    anchors: located.anchors,
    foreignRegions,
  },);
  return {
    degree,
    anchored: !misattributed,
    misattributed,
    quote: voice.value
      .copiedSpan,
    matched: misattributed ? '' : matchedRegion({
      document,
      anchors: located.anchors,
    },),
  };
}

/**
 Whether a claim proved full coverage.

 @param claim - weighed reply

 @returns Whether it claimed full coverage and its quote was found

 @example
 ```ts
 const full = isFull(claim,);
 ```
 */
function isFull(claim: WeighedVoice,): boolean {
  return (claim.degree === 'full') && claim.anchored;
}

/**
 Whether a claim proved partial coverage.

 @param claim - weighed reply

 @returns Whether it claimed partial coverage and its quote was found

 @example
 ```ts
 const partial = isPartial(claim,);
 ```
 */
function isPartial(claim: WeighedVoice,): boolean {
  return (claim.degree === 'partial') && claim.anchored;
}

/**
 Whether a claim reported no coverage at all.

 @param claim - weighed reply

 @returns Whether it found nothing

 @example
 ```ts
 const absent = isAbsent(claim,);
 ```
 */
function isAbsent(claim: WeighedVoice,): boolean {
  return claim.degree === 'none';
}

/**
 Whether a claim of coverage could not be found in the document.

 @param claim - weighed reply

 @returns Whether it claimed coverage and its quote was absent

 @example
 ```ts
 const unanchored = isUnanchored(claim,);
 ```
 */
function isUnanchored(claim: WeighedVoice,): boolean {
  if (claim.degree === 'none')
    return false;
  return (!claim.anchored) && (!claim.misattributed);
}

/**
 Whether a partial claim pointed at another slice's rendering.

 @param claim - weighed reply

 @returns Whether its quote was found only inside a foreign region

 @example
 ```ts
 const misattributed = isMisattributed(claim,);
 ```
 */
function isMisattributed(claim: WeighedVoice,): boolean {
  return claim.misattributed;
}

/**
 Reads one claim's quote.

 @param claim - weighed reply

 @returns Quote it offered

 @example
 ```ts
 const quote = claimQuote(claim,);
 ```
 */
function claimQuote(claim: WeighedVoice,): string {
  return claim.quote;
}

/**
 Reads one claim's matched document text.

 @param claim - weighed reply

 @returns Document's own text for the region it matched

 @example
 ```ts
 const matched = claimMatched(claim,);
 ```
 */
function claimMatched(claim: WeighedVoice,): string {
  return claim.matched;
}

/**
 Counts the weighed voices matching one predicate.

 @param weighed - every reply with its anchoring resolved

 @param matches - predicate deciding membership

 @returns How many match

 @example
 ```ts
 const absent = countVoices({ weighed, matches: isAbsent, },);
 ```
 */
function countVoices(
  {
    weighed,
    matches,
  }: {
    readonly weighed: readonly WeighedVoice[];
    readonly matches: (claim: WeighedVoice,) => boolean;
  },
): number {
  return weighed.filter(matches,)
    .length;
}

/**
 Weighs a roster's coverage replies into one verdict.

 @param voices - replies heard from the roster

 @param document - translation every quote is checked against

 @param foreignRegions - target regions the pairing assigned to other source
 slices, which a partial claim cannot draw coverage from; none by default

 @param asked - reachable models the question went to, which the majority is
 taken over

 @param quorumMet - whether enough of them answered to decide at all

 @param original - passage whose comments are the original's own on the
 sheet the voices read: each quote is located in the translation as that sheet
 showed it, without the notes between translators, and the evidence kept is
 still the document's own stored slice; absent when the sheet cut nothing

 @returns Verdict plus the tallies and evidence behind it

 @example
 ```ts
 const verdict = judgeCoverage({ voices, document, asked: 6, quorumMet: true, },);
 ```
 */
export function judgeCoverage(
  {
    voices,
    document,
    foreignRegions,
    asked,
    quorumMet,
    original,
  }: {
    readonly voices: readonly HeardVoice<CoverageReportWire>[];
    readonly document: AnchorTarget;
    readonly foreignRegions?: readonly TargetRegion[];
    readonly asked: number;
    readonly quorumMet: boolean;
    readonly original?: string;
  },
): CoverageVerdict {
  /**
   Regions a partial claim may not point into.
   */
  const regions = foreignRegions ?? [];
  /**
   Every reply with its quote resolved against the document.
   */
  const weighed = voices.map(function toWeighed(voice,): WeighedVoice {
    return weighVoice({
      voice,
      document,
      foreignRegions: regions,
      ...((original === undefined) ? {} : { original, }),
    },);
  },);

  /**
   Voices that proved full coverage.
   */
  const anchoredFull = countVoices({
    weighed,
    matches: isFull,
  },);

  /**
   Voices that proved partial coverage.
   */
  const anchoredPartial = countVoices({
    weighed,
    matches: isPartial,
  },);

  /**
   Voices reporting the passage is rendered nowhere.
   */
  const absent = countVoices({
    weighed,
    matches: isAbsent,
  },);

  /**
   Votes a side needs: more than half of every model ASKED, so a lost voice
   withholds a vote rather than lowering the bar.
   */
  const majority = Math.floor(asked / 2,) + 1;

  /**
   Voices proving coverage of any degree, which is what forbids inserting the
   passage whole.
   */
  const anchoredAny = anchoredFull + anchoredPartial;

  /**
   Which side, if either, reached the threshold.
   */
  const decided = (anchoredFull >= majority)
    ? 'carried'
    : ((anchoredAny >= majority)
      ? 'partly-carried'
      : ((absent >= majority) ? 'absent' : 'split'));
  return {
    kind: quorumMet ? decided : 'inconclusive',
    anchoredFull,
    anchoredPartial,
    absent,
    unanchored: countVoices({
      weighed,
      matches: isUnanchored,
    },),
    misattributed: countVoices({
      weighed,
      matches: isMisattributed,
    },),
    heard: voices.length,
    asked,
    // ONLY THE FULL VOTES ARE EVIDENCE. A partial voter quotes what it found
    // instead, which may sit anywhere on the page; the sixth yuki418330012 pass of
    // 2026-09-09 carried a credits line on seven full votes and one partial vote
    // quoting a sentence about suicide attempts, a lane rewrote that sentence,
    // and the carried-insertion guard stopped the entry over a region no full
    // vote ever named.
    evidence: weighed.filter(isFull,)
      .map(claimMatched,),
    unanchoredQuotes: weighed.filter(isUnanchored,)
      .map(claimQuote,),
    misattributedQuotes: weighed.filter(isMisattributed,)
      .map(claimQuote,),
  };
}

//endregion Coverage verdict
