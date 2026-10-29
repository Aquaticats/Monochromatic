import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

//region Credential search
// Finds every copy of any of several needles in a text in one pass over the
// text (Aho-Corasick), so the credential mask costs the length of the text plus
// the number of copies found, never the text times the number of spellings it
// looks for. Units are compared as one-unit strings, so a text and a needle
// are read the same way whatever characters they hold.

/**
 One copy of a needle found in a text.
 */
export type NeedleFind = {
  /**
   Position of the needle in the list searched.
   */
  readonly needle: number;

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
 One state of the automaton: where each unit leads, where a miss falls back
 to, and which needles end here.
 */
type State = {
  /**
   State each unit leads to.
   */
  readonly next: Map<string, number>;

  /**
   State a unit with no transition falls back to; the root's own is the root.
   */
  fallback: number;

  /**
   Needles ending at this state, those ending at its fallbacks included.
   */
  readonly ending: number[];
};

/**
 Makes a state with no transition, no fallback and no ending.

 @returns Empty state

 @example
 ```ts
 const state = emptyState();
 ```
 */
function emptyState(): State {
  return {
    next: new Map(),
    fallback: 0,
    ending: [],
  };
}

/**
 Reads a state of the automaton, which every index it holds names.

 @param states - automaton's states

 @param at - index of the state

 @returns State

 @throws Error when no state has the index, which no transition leads to

 @example
 ```ts
 const root = stateAt({ states, at: 0, },);
 ```
 */
function stateAt({
  states,
  at,
}: {
  readonly states: readonly State[];
  readonly at: number
},): State {
  /**
   The state the index names.
   */
  const state = states[at];
  if (state === undefined)
    throw new Error(`unreachable: the automaton holds no state ${String(at,)}, though every transition and fallback leads to one it built`,);

  return state;
}

/**
 Builds the trie of the needles, one state per distinct prefix.

 @param needles - non-empty texts to find

 @returns States with transitions and ends set, fallbacks not yet

 @example
 ```ts
 const trie = trieOf({ needles: ['ab', 'abc',], },);
 ```
 */
function trieOf({ needles, }: { readonly needles: readonly string[]; },): State[] {
  /**
   States being built, the root first.
   */
  const states: State[] = [emptyState(),];
  needles.forEach(function addNeedle(
    needle,
    index,
  ): void {
    /**
     State the needle's prefix read so far leads to.
     */
    const cursor = { state: 0, };
    for (const unit of needle.split('',)) {
      /**
       State this unit already leads to, when an earlier needle shares the prefix.
       */
      const known = stateAt({
        states,
        at: cursor.state,
      },)
        .next
        .get(unit,);
      if (known !== undefined) {
        cursor.state = known;
        continue;
      }
      /**
       State created for the new prefix.
       */
      const created = states.length;
      states.push(emptyState(),);
      stateAt({
        states,
        at: cursor.state,
      },)
        .next
        .set(
        unit,
        created,
        );
      cursor.state = created;
    }
    stateAt({
      states,
      at: cursor.state,
    },)
      .ending
      .push(index,);
  },);
  return states;
}

/**
 Follows fallbacks from a state to the first that has a transition for a unit.

 @param states - automaton's states

 @param from - state the unit was read in

 @param unit - unit read

 @returns State the unit leads to from the nearest state that has a
 transition for it, the root when none does

 @example
 ```ts
 const to = stepFrom({ states, from: 0, unit: 'a', },);
 ```
 */
function stepFrom(
  {
    states,
    from,
    unit,
  }: {
    readonly states: readonly State[];
    readonly from: number;
    readonly unit: string;
  },
): number {
  /**
   State being tried.
   */
  const cursor = { state: from, };
  while ((cursor.state !== 0) && (!stateAt({
    states,
    at: cursor.state,
  },)
    .next
    .has(unit,)))
    cursor.state = stateAt({
      states,
      at: cursor.state,
    },)
      .fallback;

  return stateAt({
    states,
    at: cursor.state,
  },)
    .next
    .get(unit,)
    ?? 0;
}

/**
 Sets every state's fallback, breadth first, and gathers into each state the
 needles that end at its fallbacks.

 @param states - trie to complete

 @mutates states - fallbacks and ends are filled in

 @example
 ```ts
 linkFallbacks({ states, },);
 ```
 */
function linkFallbacks({ states, }: { readonly states: readonly State[]; },): void {
  /**
   States whose fallbacks are set, in the order their depth gives.
   */
  const queue: number[] = [...stateAt({
    states,
    at: 0,
  },)
    .next
    .values(),];
  for (const state of queue) {
    for (
      const [unit, child,] of stateAt({
        states,
        at: state,
      },)
        .next
    ) {
      /**
       State the child falls back to, the root when nothing shorter matches.
       */
      const target = stepFrom({
        states,
        from: stateAt({
          states,
          at: state,
        },)
          .fallback,
        unit,
      },);
      /**
       The child's own state.
       */
      const own = stateAt({
        states,
        at: child,
      },);
      // The queue holds states one or more units deep, so `child` is two or
      // more deep, and `target` is at most one deeper than the parent's
      // fallback, which is shallower than the parent: never the child itself.
      // A state falling back to itself would keep `stepFrom` in its loop.
      if (target === child)
        throw new Error(`unreachable: state ${String(child,)} of the automaton fell back to itself, though every fallback is shallower than the state it belongs to`,);
      own.fallback = target;
      own.ending
        .push(...stateAt({
          states,
          at: own.fallback,
        },)
          .ending,);
      queue.push(child,);
    }
  }
}

/**
 Finds every copy of any needle in a text.

 @param text - text searched

 @param needles - non-empty texts to find, matched by UTF-16 unit

 @returns Finds in the order their ends fall, overlapping ones included; none
 for an empty needle list

 @example
 ```ts
 findNeedles({ text: 'xxabxx', needles: ['ab', 'xa',], },);
 // => [{ needle: 1, start: 1, end: 3, }, { needle: 0, start: 2, end: 4, }]
 ```
 */
export function findNeedles(
  {
    text,
    needles,
  }: {
    readonly text: string;
    readonly needles: readonly string[];
  },
): readonly NeedleFind[] {
  if (needles.length === 0)
    return [];

  /**
   States of the automaton over the needles.
   */
  const states = trieOf({ needles, },);
  linkFallbacks({ states, },);

  /**
   Units that can begin a needle, so a text unit that begins none is skipped
   without a lookup.
   */
  const openers = new Set(stateAt({
    states,
    at: 0,
  },)
    .next
    .keys(),);

  /**
   Finds so far.
   */
  const finds: NeedleFind[] = [];
  /**
   State the text read so far leads to.
   */
  const cursor = { state: 0, };
  for (let at = 0; at < text.length; at += 1) {
    /**
     Unit read at this position.
     */
    const unit = text.charAt(at,);
    if ((cursor.state === 0) && (!openers.has(unit,)))
      continue;

    cursor.state = stepFrom({
      states,
      from: cursor.state,
      unit,
    },);
    for (const needle of stateAt({
      states,
      at: cursor.state,
    },)
      .ending) {
      /**
       Units of the needle that ends here.
       */
      const { length, } = nonNullishOrThrow(needles[needle],);
      finds.push({
        needle,
        start: (at + 1) - length,
        end: at + 1,
      },);
    }
  }
  return finds;
}

//endregion Credential search
