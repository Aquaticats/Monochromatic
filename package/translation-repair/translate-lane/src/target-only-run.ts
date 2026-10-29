import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  type ParsedBlock,
  parsedTopLevelBlocks,
  requireMdxRefusal,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Target-only run
// ENGLISH THE ARCHIVE CARRIES THAT THE CHINESE NEVER SAID, held out of
// translation and spliced back unchanged.
//
// WHAT GOES WRONG WITHOUT THIS. The translate lane writes each slice fresh from
// its source. Where a human translator transcribed an image into the English,
// that transcript has no source counterpart, so the lane produces nothing for
// it and the shipped slice simply does not contain it. Measured on the pool
// settled 2026-08-18: `dogesir_` slice 3 went from 1766 archive characters to
// 215 shipped, losing a 1487-character transcript, and `wangzihao980` slice 4
// went from 1228 to 175, losing 1098. Both are memorial pages and the lost
// blocks are the accessible reading of an image. Nothing recovers them.
//
// THIS IS THE `A` HALF OF THE PICTURE DECISION, whose authorized answer is "B, but best
// effort, fallback to A whenever an image's OCR doesn't make sense". `B` sends
// the image so the transcript has a source that can be CHECKED. `A` protects
// the block structurally. `A` is built first because it is what makes the
// failure of `B` survivable, and because it is correct on its own: a passage
// nobody can check is still a passage nobody may delete.
//
// THE ANCHOR IS A MARKUP BLOCK RATHER THAN A RATIO. Both measured cases end
// their source with a component, `<PhotoScroll photos={[...]} />`, which the
// archive repeats because markup is not prose. The archive then continues past
// where the source ended. So the rule is: find the last archive block that
// repeats the source's OWN LAST BLOCK, and treat what follows in the archive as
// target-only. No length ratio, no baseline, no threshold to tune.
//
// COMPARED WITH WHITESPACE COLLAPSED, and that is not a convenience. A
// byte-identical comparison was tried first and it missed `dogesir_` slice 3,
// the very case that prompted this: the source writes two spaces before `]}`
// and the archive writes one. The markup is the same markup and a copy that
// re-indented it is still a copy, so runs of whitespace are folded to one space
// on both sides before the blocks are compared. Nothing else is normalised, so
// two different components never collide.
//
// BLOCKS ARE READ OFF THE PARSE (ledger B68), not split on blank lines. A
// quoted transcript written on the line after its anchor, with no blank line
// between, joined the anchor in one block under a split; that block matched
// no source block, nothing was protected, and the lane could rewrite the
// transcript away from a source that never had it, the failure this module
// exists to prevent. A transcript is a block the parse reads as a
// blockquote, and the split cuts the ARCHIVE'S OWN BYTES at the anchor's end
// rather than rejoining trimmed blocks with a blank line, so restoring the
// protected run onto the judged part rebuilds the archive exactly.

/**
 Characters that count as whitespace between blocks, named as a set so the
 test reads as membership rather than as a chain of comparisons.
 */
const WHITESPACE: ReadonlySet<string> = new Set([
  ' ',
  '\t',
  '\n',
  '\r',
],);

/**
 One slice split into the part a lane may rewrite and the part it may not.

 @example
 ```ts
 const split: TargetOnlySplit = { judgedText: 'The cat naps.', protectedText: '', };
 ```
 */
export type TargetOnlySplit = {
  /**
   Archive wording that has a source to be judged against, which is what the
   lane translates and what the incumbent enters the ballot as.
   */
  readonly judgedText: string;

  /**
   Archive wording the source cannot account for, empty when there is none:
   the archive's bytes from the anchor's end, the line breaks before the run
   included.

   SPLICED BACK VERBATIM onto whichever wording wins, so it survives a
   replacement and a retention alike.
   */
  readonly protectedText: string;
};

/**
 Folds every run of whitespace in a block to a single space.

 A LINEAR SCAN rather than a pattern, per `RG1`: the rule is "runs of
 whitespace stand as one space", which a scan states directly and in one pass
 over the block.

 @param block - block to fold

 @returns Its text with whitespace runs folded

 @example
 ```ts
 const key = collapsed({ block: 'a  b', },);
 ```
 */
function collapsed({ block, }: { readonly block: string; },): string {
  /**
   Characters kept, with each run of whitespace standing as one space.
   */
  const out: string[] = [];

  /**
   Whether the previous character was whitespace, so a run emits one space.
   */
  const run = { open: false, };

  for (const character of block) {
    if (WHITESPACE.has(character,)) {
      run.open = true;
      continue;
    }
    if (run.open && (out.length > 0))
      out.push(' ',);
    run.open = false;
    out.push(character,);
  }
  return out.join('',);
}

/**
 Separates the archive's wording at the source's last block, given both
 passages read into blocks.

 @param source - source blocks, whose last one is the only possible anchor

 @param archive - archive blocks, among which the anchor is looked for

 @param incumbentText - archive wording the blocks were read from

 @param whole - split kept when nothing is protected

 @returns Wording to judge, and wording to protect

 @example
 ```ts
 const split = splitAtAnchor({ source, archive, incumbentText, whole, },);
 ```
 */
function splitAtAnchor(
  {
    source,
    archive,
    incumbentText,
    whole,
  }: {
    readonly source: readonly ParsedBlock[];
    readonly archive: readonly ParsedBlock[];
    readonly incumbentText: string;
    readonly whole: TargetOnlySplit;
  },
): TargetOnlySplit {
  /**
   Source's final block, which the archive must reproduce exactly for any of
   this to apply.
   */
  const anchor = source.at(-1,);
  if (anchor === undefined)
    return whole;

  /**
   Archive blocks reduced to comparison keys, one per block.
   */
  const keys = archive.map(function key(block,): string {
    return collapsed({ block: block.text, },);
  },);

  /**
   Where the archive repeats the source's last block, searched from the end so
   a component appearing twice in one slice anchors on its last occurrence.
   */
  const anchorAt = keys.lastIndexOf(collapsed({ block: anchor.text, },),);
  if (anchorAt === (-1))
    return whole;

  /**
   The anchor block itself, as the archive carries it.
   */
  const anchorBlock = nonNullishOrThrow(archive[anchorAt],);

  /**
   Archive blocks past the anchor, which no source block follows.
   */
  const trailing = archive.slice(anchorAt + 1,);
  if (trailing.length === 0)
    return whole;

  // A run carrying no blockquote is ordinary trailing prose, which a
  // translator may legitimately reword. Only a transcript is protected.
  // Read off the block's own node kind, not whether its trimmed text starts
  // with `>`, for the same reason every other check here reads the parse.
  if (!trailing.some(function isQuote(block,): boolean {
    return block.kind === 'blockquote';
  },))
    return whole;

  return {
    judgedText: incumbentText.slice(
      0,
      anchorBlock.endOffset,
    ),
    protectedText: incumbentText.slice(anchorBlock.endOffset,),
  };
}

/**
 Separates the archive wording a source can account for from what follows it.

 REQUIRES THE ANCHOR TO BE THE SOURCE'S LAST BLOCK. An identical block in the
 middle of both documents says nothing about what comes after it, since the
 source has more to say there too. Only an archive that continues past the end
 of its source is carrying something the source never had.

 REQUIRES A BLOCKQUOTE IN THE RUN. Every transcript in the enumerated
 population is written as one, and the requirement keeps an ordinary trailing
 sentence, which a translator may legitimately reword or drop, out of the
 protected region.

 A PASSAGE THE SLICE GRAMMAR REFUSES IS KEPT WHOLE: a run this reader cannot
 read into blocks cannot be shown to carry a transcript the source lacks.

 @param sourceText - original passage

 @param incumbentText - archive wording of that passage

 @returns Wording to judge, and wording to protect

 @example
 ```ts
 const split = splitTargetOnlyRun({ sourceText, incumbentText, },);
 ```
 */
export function splitTargetOnlyRun(
  {
    sourceText,
    incumbentText,
  }: {
    readonly sourceText: string;
    readonly incumbentText: string;
  },
): TargetOnlySplit {
  /**
   Whole passage kept when nothing is protected, which is the ordinary case.
   */
  const whole: TargetOnlySplit = {
    judgedText: incumbentText,
    protectedText: '',
  };
  try {
    return splitAtAnchor({
      source: parsedTopLevelBlocks({ text: sourceText, },),
      archive: parsedTopLevelBlocks({ text: incumbentText, },),
      incumbentText,
      whole,
    },);
  }
  catch (error) {
    // Only the grammar's own refusal keeps the passage whole, and the slice
    // grammar reads an exhaustion of its own parse as that refusal (`parseSliceBody`);
    // anything else, an exhaustion of the code that reads the blocks included, is
    // an unexpected state that must keep propagating.
    requireMdxRefusal({ error, },);
    return whole;
  }
}

/**
 Puts a protected run back onto whatever wording won.

 SEPARATED FROM THE SPLIT so both the replacement path and the retention path
 go through one place: a slice that keeps the archive must end up with exactly
 the archive's own bytes, and a slice that replaces it must end up with the new
 wording plus the same protected run.

 @param text - wording that won

 @param protectedText - run held out of judging, its leading line breaks
 included, possibly empty

 @returns Wording with the run restored

 @example
 ```ts
 const whole = restoreTargetOnlyRun({ text, protectedText, },);
 ```
 */
export function restoreTargetOnlyRun(
  {
    text,
    protectedText,
  }: {
    readonly text: string;
    readonly protectedText: string;
  },
): string {
  return `${text}${protectedText}`;
}

//endregion Target-only run
