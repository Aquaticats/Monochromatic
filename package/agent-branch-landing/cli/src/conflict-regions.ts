/**
 The conflict regions Git leaves in a file, and keeping one side of each of them, which is what settles a merge whose one side has
 already landed elsewhere.

 The sides are named by their role in the merge, not by a branch name: `current` is the side the checkout holds, which Git labels
 after `<<<<<<<`, and `incoming` is the side being merged in, labelled after `>>>>>>>`. In a merge of `main` into an agent's
 worktree, `current` is the agent's branch and `incoming` is `main`; in a merge of a branch into `main`, it is the other way
 round.

 @module
 */

/**
 Which side of each conflict region to keep.
 */
export type SideToKeep =
  /**
   The side the checkout holds, which is what a merge's `<<<<<<<` region holds.
   */
  | 'current'
  /**
   The side merged in, which is what a `>>>>>>>` region holds.
   */
    | 'incoming'
  /**
   Both sides, the checkout's first, for two additions that both stay.
   */
    | 'current-then-incoming'
  /**
   Both sides, the incoming one first, for the same two additions the other way round.
   */
    | 'incoming-then-current';

/**
 The sides a caller may ask to keep, as a command line spells them.
 */
export const SIDES_TO_KEEP: readonly SideToKeep[] = [
  'current',
  'incoming',
  'current-then-incoming',
  'incoming-then-current',
];

/**
 What keeping one side of each region left.
 */
export type ResolvedText = {
  /**
   The file as it stands once every region is settled.
   */
  readonly text: string;

  /**
   How many regions the file held.
   */
  readonly regions: number;
};

/**
 The marker that opens a region and names the checkout's own side.
 */
const OPENING_MARKER = '<<<<<<< ';

/**
 The marker that separates the two sides, which is a line of its own.
 */
const SEPARATOR = '=======';

/**
 The marker that ends a region and names the side merged in.
 */
const CLOSING_MARKER = '>>>>>>> ';

/**
 The marker that starts the common ancestor a diff3 conflict shows, which neither side keeps.
 */
const ANCESTOR_MARKER = '||||||| ';

/**
 Where the lines of a file being read belong.
 */
type Place = 'outside' | 'current-side' | 'ancestor' | 'incoming-side';

/**
 What has been read so far.

 @internal
 */
type Reading = {
  /**
   Where the reader is.
   */
  readonly place: Place;

  /**
   The lines of the checkout's own side of the region being read.
   */
  readonly currentSide: readonly string[];

  /**
   The lines of the side merged in.
   */
  readonly incomingSide: readonly string[];

  /**
   The lines kept so far.
   */
  readonly kept: readonly string[];

  /**
   How many regions were settled.
   */
  readonly regions: number;
};

/**
 What a reader holds before the first line.
 */
const START: Reading = {
  place: 'outside',
  currentSide: [],
  incomingSide: [],
  kept: [],
  regions: 0,
};

/**
 Reads the next line into what has been read so far.

 @param reading - what has been read so far

 @param line - the next line

 @param side - the side to keep

 @returns what has been read once this line is read

 @example
 ```ts
 const after = after({ reading: START, line: 'plain', side: 'current', },);
 ```
 */
function after({
  reading,
  line,
  side,
}: {
  readonly reading: Reading;
  readonly line: string;
  readonly side: SideToKeep;
},): Reading {
  if ((reading.place === 'outside') && line.startsWith(OPENING_MARKER,)) {
    return {
      ...reading,
      place: 'current-side',
      currentSide: [],
      incomingSide: [],
      regions: reading.regions + 1,
    };
  }
  if ((reading.place === 'current-side') && line.startsWith(ANCESTOR_MARKER,)) {
    return {
      ...reading,
      place: 'ancestor',
    };
  }
  if (((reading.place === 'current-side') || (reading.place === 'ancestor')) && (line === SEPARATOR)) {
    return {
      ...reading,
      place: 'incoming-side',
    };
  }
  if ((reading.place === 'incoming-side') && line.startsWith(CLOSING_MARKER,)) {
    return {
      ...reading,
      place: 'outside',
      kept: [
        ...reading.kept,
        ...keptSides({
          currentSide: reading.currentSide,
          incomingSide: reading.incomingSide,
          side,
        },),
      ],
    };
  }
  // The common ancestor of a diff3 region belongs to neither side, so its lines are read and dropped.
  if (reading.place === 'current-side') {
    return {
      ...reading,
      currentSide: [
        ...reading.currentSide,
        line,
      ],
    };
  }
  if (reading.place === 'incoming-side') {
    return {
      ...reading,
      incomingSide: [
        ...reading.incomingSide,
        line,
      ],
    };
  }
  return reading.place === 'outside'
    ? {
      ...reading,
      kept: [
        ...reading.kept,
        line,
      ],
    }
    : reading;
}

/**
 Picks the sides of one region in the order the caller asked for.

 @param currentSide - the lines of the checkout's own side

 @param incomingSide - the lines of the side merged in

 @param side - the side to keep

 @returns the lines that stay, in order

 @example
 ```ts
 const lines = keptSides({ currentSide: ['a',], incomingSide: ['b',], side: 'incoming', },);
 ```
 */
function keptSides({
  currentSide,
  incomingSide,
  side,
}: {
  readonly currentSide: readonly string[];
  readonly incomingSide: readonly string[];
  readonly side: SideToKeep;
},): readonly string[] {
  if (side === 'current-then-incoming') return [
    ...currentSide,
    ...incomingSide,
  ];
  if (side === 'incoming-then-current') return [
    ...incomingSide,
    ...currentSide,
  ];
  return side === 'incoming' ? incomingSide : currentSide;
}

/**
 Keeps one side of every conflict region of a text, and changes nothing else about the file.

 @param text - the file's text

 @param side - the side to keep

 @returns the settled text and how many regions it held

 @throws Error when a region does not close

 @example
 ```ts
 const settled = keepOneSideOfEachRegion({ text, side: 'incoming', },);
 ```
 */
export function keepOneSideOfEachRegion({
  text,
  side,
}: {
  readonly text: string;
  readonly side: SideToKeep;
},): ResolvedText {
  /**
   The file, read line by line into what has been kept.
   */
  const read = text.split('\n',)
    .reduce(
    function readLine(
      reading: Reading,
      line: string,
    ) {
      return after({
        reading,
        line,
        side,
      },);
    },
    START,
    );
  if (read.place !== 'outside') throw new Error('a conflict region does not close',);
  return {
    text: read.kept
      .join('\n',),
    regions: read.regions,
  };
}
