import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { JsoncComment, } from './comment.ts';
import { appendComments, } from './parse-trivia.ts';
import type {
  JsoncRecordEntry,
  JsoncValue,
} from './value.ts';

/**
 Returns the elements with any comment found before the closing `]` attached to
 the last element, or the elements unchanged when there is no such comment.
 Builds a new array rather than mutating the input.
 
 @param elements - Parsed elements.
 
 @param dangling - Comments found before the close (possibly empty).
 
 @returns Elements with the dangling comment folded into the last one.
 
 @example
 ```ts
 attachDanglingToLastElement({ elements: [n], dangling: [c] });
 // => [n with c appended]
 ```
 */
function attachDanglingToLastElement({
  elements,
  dangling,
}: {
  readonly elements: readonly JsoncValue[];
  readonly dangling: readonly JsoncComment[];
},): readonly JsoncValue[] {
  if ((dangling.length === 0) || (elements.length === 0))
    return elements;
  /**
   Last element, which receives the dangling comment.
   */
  const last = nonNullishOrThrow(elements.at(-1,),);
  return [
    ...elements.slice(
      0,
      -1,
    ),
    appendComments({
      node: last,
      comments: dangling,
    },),
  ];
}

/**
 Finishes an array node, attaching any comment found before the closing `]`:
 to the last element when present, otherwise to the empty array node itself.
 
 @param elements - Parsed elements.
 
 @param dangling - Comments found before the close (possibly empty).
 
 @returns Array node.
 
 @example
 ```ts
 closeArray({ elements: [], dangling: [{ type: 'block', text: ' x ' }] });
 // => { kind: 'array', elements: [], comment: { type: 'block', text: ' x ' } }
 ```
 */
export function closeArray({
  elements,
  dangling,
}: {
  readonly elements: readonly JsoncValue[];
  readonly dangling: readonly JsoncComment[];
},): JsoncValue {
  /**
   Comments owned by the array node itself (only when it has no elements).
   */
  const ownComments = (elements.length === 0)
    ? dangling
    : [];
  /**
   Array node before any own-comment attachment.
   */
  const arrayNode: JsoncValue = {
    kind: 'array',
    elements: attachDanglingToLastElement({
      elements,
      dangling,
    },),
  };
  return appendComments({
    node: arrayNode,
    comments: ownComments,
  },);
}

/**
 Returns the entries with any comment found before the closing `}` attached to
 the last entry's value, or the entries unchanged when there is no such comment.
 Builds a new array rather than mutating the input.
 
 @param entries - Parsed entries.
 
 @param dangling - Comments found before the close (possibly empty).
 
 @returns Entries with the dangling comment folded into the last value.
 
 @example
 ```ts
 attachDanglingToLastEntry({ entries: [e], dangling: [c] });
 // => [e with c appended to its value]
 ```
 */
function attachDanglingToLastEntry({
  entries,
  dangling,
}: {
  readonly entries: readonly JsoncRecordEntry[];
  readonly dangling: readonly JsoncComment[];
},): readonly JsoncRecordEntry[] {
  if ((dangling.length === 0) || (entries.length === 0))
    return entries;
  /**
   Last entry, whose value receives the dangling comment.
   */
  const last = nonNullishOrThrow(entries.at(-1,),);
  return [
    ...entries.slice(
      0,
      -1,
    ),
    {
      key: last.key,
      value: appendComments({
        node: last.value,
        comments: dangling,
      },),
    },
  ];
}

/**
 Finishes a record node, attaching any comment found before the closing `}`:
 to the last entry's value when present, otherwise to the empty record node.
 
 @param entries - Parsed entries.
 
 @param dangling - Comments found before the close (possibly empty).
 
 @returns Record node.
 
 @example
 ```ts
 closeRecord({ entries: [], dangling: [] });
 // => { kind: 'record', entries: [] }
 ```
 */
export function closeRecord({
  entries,
  dangling,
}: {
  readonly entries: readonly JsoncRecordEntry[];
  readonly dangling: readonly JsoncComment[];
},): JsoncValue {
  /**
   Comments owned by the record node itself (only when it has no entries).
   */
  const ownComments = (entries.length === 0)
    ? dangling
    : [];
  /**
   Record node before any own-comment attachment.
   */
  const recordNode: JsoncValue = {
    kind: 'record',
    entries: attachDanglingToLastEntry({
      entries,
      dangling,
    },),
  };
  return appendComments({
    node: recordNode,
    comments: ownComments,
  },);
}
