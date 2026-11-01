//region Repeat groups types
// The shapes a repeat-group lookup reads and returns, shared by the reading,
// the run finders and the public entry.

/**
 One place a repeated wording stands in the source.

 @example
 ```ts
 const occurrence: RepeatOccurrence = { sliceIndex: 0, startOffset: 0, endOffset: 12, line: 1, };
 ```
 */
export type RepeatOccurrence = {
  /**
   Index of the slice holding the whole occurrence.
   */
  readonly sliceIndex: number;

  /**
   Absolute offset of the occurrence's first character.
   */
  readonly startOffset: number;

  /**
   Absolute exclusive end offset of the occurrence.
   */
  readonly endOffset: number;

  /**
   One-based line the occurrence starts on.
   */
  readonly line: number;
};

/**
 Why a repeat's occurrence stands in no slice: no slice holds its start, or the
 slice holding its start ends inside it.
 */
export type UnplacedReason = 'in-no-slice' | 'crosses-slice-end';

/**
 One place a repeated wording stands that no slice holds whole, which a
 sealed original, a section the alignment refused, or a cut inside the wording
 leaves.

 @example
 ```ts
 const unplaced: UnplacedOccurrence = { startOffset: 22, endOffset: 34, line: 3, reason: 'in-no-slice', };
 ```
 */
export type UnplacedOccurrence = {
  /**
   Absolute offset of the occurrence's first character.
   */
  readonly startOffset: number;

  /**
   Absolute exclusive end offset of the occurrence.
   */
  readonly endOffset: number;

  /**
   One-based line the occurrence starts on.
   */
  readonly line: number;

  /**
   Why no slice holds the occurrence whole.
   */
  readonly reason: UnplacedReason;
};

/**
 A run of non-space characters that repeats at two or more places.

 @example
 ```ts
 const group: MaximalRepeatGroup = { kind: 'maximal', length: 12, occurrences: [occurrence,], };
 ```
 */
export type MaximalRepeatGroup = {
  /**
   Kind of repeat: a run repeated as it stands.
   */
  readonly kind: 'maximal';

  /**
   Non-space characters in the repeated run.
   */
  readonly length: number;

  /**
   Every place the run stands that a slice holds whole, in page order, none
   overlapping another.
   */
  readonly occurrences: readonly RepeatOccurrence[];

  /**
   Every place the run stands that no slice holds whole, in page order. The
   group's full count is both lists together.
   */
  readonly unplaced: readonly UnplacedOccurrence[];
};

/**
 One occurrence of a refrain, with the span of its varying slot.

 @example
 ```ts
 const occurrence: RefrainOccurrence = { ...occurrence, slotStartOffset: 7, slotEndOffset: 8, };
 ```
 */
export type RefrainOccurrence = RepeatOccurrence & {
  /**
   Absolute offset of the slot's first character.
   */
  readonly slotStartOffset: number;

  /**
   Absolute exclusive end offset of the slot.
   */
  readonly slotEndOffset: number;
};

/**
 Clauses that share a prefix and a suffix and differ in a slot between them.

 @example
 ```ts
 const group: RefrainGroup = { kind: 'refrain', prefix, suffix, slotBounds, occurrences, };
 ```
 */
export type RefrainGroup = {
  /**
   Kind of repeat: a frame with a varying slot.
   */
  readonly kind: 'refrain';

  /**
   Characters every occurrence begins with.
   */
  readonly prefix: string;

  /**
   Characters every occurrence ends with.
   */
  readonly suffix: string;

  /**
   Bounds the slot's length is held to.
   */
  readonly slotBounds: {
    /**
     Fewest slot characters.
     */
    readonly minimum: number;

    /**
     Most slot characters.
     */
    readonly maximum: number;
  };

  /**
   Every clause carrying the frame that a slice holds whole, in page order.
   */
  readonly occurrences: readonly RefrainOccurrence[];

  /**
   Every clause carrying the frame that no slice holds whole, in page order.
   The group's full count is both lists together.
   */
  readonly unplaced: readonly UnplacedOccurrence[];
};

/**
 One group of repeated source wording.

 @example
 ```ts
 const groups: readonly RepeatGroup[] = repeatGroupsOf({ text, slices, },);
 ```
 */
export type RepeatGroup = MaximalRepeatGroup | RefrainGroup;

/**
 The wording of a page's body paragraphs, a character at a time, with the
 place each came from. A character is a code point, so one outside the basic
 plane is one element though it spans two UTF-16 units of the page.

 @example
 ```ts
 const wordingChar: WordingStream = { chars: ['猫',], offsets: [0,], depths: [0,], };
 ```
 */
export type WordingStream = {
  /**
   Wording characters in page order, with a break wherever markup or a
   paragraph's end interrupts them.
   */
  readonly chars: readonly string[];

  /**
   Absolute offset of each character's first UTF-16 unit, or minus one at a break.
   */
  readonly offsets: readonly number[];

  /**
   Open quotation depth at each character, zero at a break. An opening mark
   already counts as inside its quotation and a closing mark as outside it;
   a straight double quotation mark opens a quotation when none is open in the
   paragraph and closes the open one otherwise.
   */
  readonly depths: readonly number[];
};

/**
 A stretch of a page that no reader sees, such as an HTML comment or the tag of
 a container the parse dissolved.

 @example
 ```ts
 const range: HiddenRange = { startOffset: 4, endOffset: 16, };
 ```
 */
export type HiddenRange = {
  /**
   Absolute offset of the range's first unit.
   */
  readonly startOffset: number;

  /**
   Absolute exclusive end offset of the range.
   */
  readonly endOffset: number;
};

/**
 A place a repeat stands, measured in the source.

 @example
 ```ts
 const span: Span = { startOffset: 0, endOffset: 12, quoted: false, };
 ```
 */
export type Span = {
  /**
   Absolute offset of the first character.
   */
  readonly startOffset: number;

  /**
   Absolute exclusive end offset.
   */
  readonly endOffset: number;

  /**
   Whether any character of the span is quoted speech or a quotation mark.
   */
  readonly quoted: boolean;
};

/**
 A refrain clause, with the slot it leaves open once its frame is known.

 @example
 ```ts
 const clause: Clause = { start: 0, end: 11, points: [...'所以这只猫是个胖猫咪吧',], text: '所以这只猫是个胖猫咪吧', quoted: false, };
 ```
 */
export type Clause = {
  /**
   Index in the wording stream of the clause's first character.
   */
  readonly start: number;

  /**
   Index in the wording stream one past the clause's last character.
   */
  readonly end: number;

  /**
   The clause's characters, one code point each.
   */
  readonly points: readonly string[];

  /**
   The same characters as one string, for equality and for keys.
   */
  readonly text: string;

  /**
   Whether any character of the clause is quoted speech or a quotation mark.
   */
  readonly quoted: boolean;
};

/**
 A repeat found before it is filtered or placed in a slice.

 @example
 ```ts
 const found: FoundRepeat = { kind: 'maximal', length: 12, spans: [], };
 ```
 */
export type FoundRepeat = {
  /**
   Kind of repeat found.
   */
  readonly kind: 'maximal' | 'refrain';

  /**
   Characters the repeat's first occurrence holds before the slot, for a refrain.
   */
  readonly prefix: string;

  /**
   Characters the repeat's first occurrence holds after the slot, for a refrain.
   */
  readonly suffix: string;

  /**
   Non-space characters in the run, for a maximal repeat.
   */
  readonly length: number;

  /**
   Every place the repeat stands, each with its slot for a refrain.
   */
  readonly spans: readonly FoundSpan[];
};

/**
 A place a repeat stands, with the slot a refrain leaves open there.

 @example
 ```ts
 const found: FoundSpan = { startOffset: 0, endOffset: 12, quoted: false, width: 12, slotStartOffset: 0, slotEndOffset: 12, };
 ```
 */
export type FoundSpan = Span & {
  /**
   Wording characters the place holds, one per code point, counting no space and
   nothing a reader does not see, so a character beyond the basic plane is one
   and a comment inside the place adds none.
   */
  readonly width: number;

  /**
   Offset of the slot's first character, or the span's start for a maximal repeat.
   */
  readonly slotStartOffset: number;

  /**
   Exclusive end offset of the slot, or the span's end for a maximal repeat.
   */
  readonly slotEndOffset: number;
};

/**
 Where the slices put one occurrence: in the slice that holds it whole, or
 outside every slice.

 @example
 ```ts
 const placement: Placement = { kind: 'placed', sliceIndex: 0, };
 ```
 */
export type Placement =
  | {
    /**
     A slice holds the occurrence whole.
     */
    readonly kind: 'placed';

    /**
     Index of that slice.
     */
    readonly sliceIndex: number;
  }
    | {
    /**
     No slice holds the occurrence whole.
     */
      readonly kind: 'unplaced';

    /**
     Why none does.
     */
      readonly reason: UnplacedReason;
    };

/**
 A found place with where the slices put it and the line it starts on.

 @example
 ```ts
 const located: LocatedSpan = { span, placement: { kind: 'placed', sliceIndex: 0, }, line: 1, };
 ```
 */
export type LocatedSpan = {
  /**
   The place in the source.
   */
  readonly span: FoundSpan;

  /**
   Where the slices put it.
   */
  readonly placement: Placement;

  /**
   One-based line it starts on.
   */
  readonly line: number;
};

//endregion Repeat groups types
