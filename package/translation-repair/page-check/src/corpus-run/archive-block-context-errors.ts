import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Archive block context errors
// A BLOCK'S SOURCE SECTION IS EITHER FOUND OR A FAULT (ledger B717). Both lookups of it once read a miss as an empty
// section: `repairArchiveBlocks` read `sourceContexts.get(identity) ?? ''`, and `archiveBlockSourceContexts` read
// `alignment.pairs.at(pairIndex)?.source.text ?? ''`. An empty section is a real answer (a section only the archive
// has is filed as empty), so the two could not be told apart from a miss, and a miss checked every quote against
// nothing: each reply that cited the original was dropped as unanchored, and the log said only that the reply
// failed an anchor test. The map is built from the same blocks the review is asked about, so a miss means the
// caller and the map disagree about the block's identity, or the preparation and the alignment disagree about its
// place, which no input of the review can repair.

/**
 Refusal when the map of source sections holds no entry for a block the review is asked about.

 NAMES THE BLOCK AND WHAT THE MAP HOLDS, so a reader sees at once whether the map is empty, holds the block under an
 identity cut from another archive text, or holds other blocks only. An identity holds the block's location, its
 id and a digest of its wording, never the wording.

 @example
 ```ts
 throw new ArchiveBlockContextMissingError({ identity, held: [...sourceContexts.keys(),], });
 ```
 */
export class ArchiveBlockContextMissingError extends Error {
  /**
   Message names the block and the identities the map holds, never archive wording.
   */
  readonly messageNamesOnly: true = true;

  /**
   Identity the review looked the block up under.
   */
  readonly identity: string;

  /**
   Identities the map holds, in its order.
   */
  readonly held: readonly string[];

  /**
   Builds refusal.

   @param identity - identity the review looked the block up under

   @param held - identities the map holds
   */
  public constructor(
    {
      identity,
      held,
    }: {
      readonly identity: string;
      readonly held: readonly string[];
    },
  ) {
    /**
     What the map holds, as the message says it.
     */
    const heldText = (held.length === 0)
      ? 'holds no block'
      : `holds ${String(held.length,)} ${
        wordForCount({
          count: held.length,
          one: 'block',
          many: 'blocks',
        },)
      }: ${held.join('; ',)}`;
    super(
      `no source section is filed for archive block ${identity}; the map ${heldText}, so the caller and the map `
        + 'disagree about the block\'s identity (its location, id or wording differ); reviewing it against an empty '
        + 'source would drop every reply that cites the original as unanchored',
    );
    this.name = 'ArchiveBlockContextMissingError';
    this.identity = identity;
    this.held = held;
  }
}

/**
 Refusal when a block names an aligned pair the page's alignment does not have.

 @example
 ```ts
 throw new ArchiveBlockAlignmentMissingError({ blockId: 'block/3', pairIndex: 9, pairCount: 2, });
 ```
 */
export class ArchiveBlockAlignmentMissingError extends Error {
  /**
   Message names the block, the pair and the pair count, never archive wording.
   */
  readonly messageNamesOnly: true = true;

  /**
   Id of the block whose pair is missing.
   */
  readonly blockId: string;

  /**
   Index of the aligned pair the block names.
   */
  readonly pairIndex: number;

  /**
   How many pairs the alignment holds.
   */
  readonly pairCount: number;

  /**
   Builds refusal.

   @param blockId - id of the block whose pair is missing

   @param pairIndex - index of the aligned pair the block names

   @param pairCount - how many pairs the alignment holds
   */
  public constructor(
    {
      blockId,
      pairIndex,
      pairCount,
    }: {
      readonly blockId: string;
      readonly pairIndex: number;
      readonly pairCount: number;
    },
  ) {
    super(
      `archive block ${blockId} sits in aligned pair ${String(pairIndex,)}, and the alignment of this page holds ${
        String(pairCount,)
      } ${
        wordForCount({
          count: pairCount,
          one: 'pair',
          many: 'pairs',
        },)
      }, so the preparation and the alignment disagree about the block's place; reviewing it against an empty `
        + 'source would drop every reply that cites the original as unanchored',
    );
    this.name = 'ArchiveBlockAlignmentMissingError';
    this.blockId = blockId;
    this.pairIndex = pairIndex;
    this.pairCount = pairCount;
  }
}

//endregion Archive block context errors
