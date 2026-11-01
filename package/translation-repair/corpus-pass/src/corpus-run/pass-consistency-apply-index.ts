import type { ConsistencyRendering, } from '@monochromatic-dev/translation-repair-consistency/ts';
import type {
  PendingEdit,
  SettledGroup,
} from './pass-consistency-types.ts';

//region Pass consistency apply index
// THE GROUPS OF THE PAGE READ ONCE, BY SLICE. The replacements addressed to a slice, the places of
// the groups located in it and the places a group replaces are all questions about one slice at a
// time, and a page of many groups and many slices would pay for a scan of every group on each of
// them. The index (`settledIndexOf`) is built in one pass over the groups, so the plan
// (`pass-consistency-apply-worked.ts`) reads only the entries of the slice it asks about.

/**
 A pending replacement with the position of its group on the page.

 @example
 ```ts
 const pending: GroupEdit = { group: 0, edit, };
 ```
 */
export type GroupEdit = {
  /**
   Position of the replacement's group on the page, from zero.
   */
  readonly group: number;

  /**
   The replacement.
   */
  readonly edit: PendingEdit;
};

/**
 A located place of a group, with the group's position and the place's own.

 @example
 ```ts
 const place: LocatedPlace = { group: 0, position: 1, rendering, };
 ```
 */
export type LocatedPlace = {
  /**
   Position of the place's group on the page, from zero.
   */
  readonly group: number;

  /**
   Position of the place among its group's places, from zero.
   */
  readonly position: number;

  /**
   Where the counter found the place's words and what they are.
   */
  readonly rendering: Extract<ConsistencyRendering, { readonly kind: 'located'; }>;
};

/**
 The page's groups read once, by slice, so that the replacements and the located places of one slice are found without a
 scan of every group.

 @example
 ```ts
 const index: SettledIndex = settledIndexOf({ settled, },);
 ```
 */
export type SettledIndex = {
  /**
   The pending replacements addressed to each slice, with their groups, earlier groups first; no entry for a slice nothing
   addresses.
   */
  readonly edits: ReadonlyMap<number, readonly GroupEdit[]>;

  /**
   The located places in each slice, in the order of the groups and of their places; no entry for a slice with none.
   */
  readonly places: ReadonlyMap<number, readonly LocatedPlace[]>;

  /**
   The places a group replaces in a slice, each as `group:place:slice`.
   */
  readonly replaced: ReadonlySet<string>;
};

/**
 The key a place a group replaces in a slice is kept under in {@link SettledIndex.replaced}.

 @param group - position of the group on the page, from zero

 @param position - position of the place in the group, from zero

 @param sliceIndex - the slice

 @returns The key

 @example
 ```ts
 const key = replacedKeyOf({ group: 0, position: 1, sliceIndex: 3, },); // '0:1:3'
 ```
 */
export function replacedKeyOf(
  {
    group,
    position,
    sliceIndex,
  }: {
    readonly group: number;
    readonly position: number;
    readonly sliceIndex: number;
  },
): string {
  return `${String(group,)}:${String(position,)}:${String(sliceIndex,)}`;
}

/**
 Reads the groups of the page by slice.

 @param settled - the groups in page order, with their pending replacements

 @returns The replacements and the located places of each slice, and the places that groups replace; a place the counter
 could not locate is no place here, since no replacement can change its words

 @example
 ```ts
 const index = settledIndexOf({ settled, },);
 ```
 */
export function settledIndexOf(
  { settled, }: { readonly settled: readonly SettledGroup[]; },
): SettledIndex {
  /**
   The replacements by slice, mutated only inside this function.
   */
  const edits = new Map<number, GroupEdit[]>();
  /**
   The located places by slice, mutated only inside this function.
   */
  const places = new Map<number, LocatedPlace[]>();
  /**
   The places groups replace, mutated only inside this function.
   */
  const replaced = new Set<string>();
  for (const [group, settledGroup,] of settled.entries()) {
    for (const edit of settledGroup.edits) {
      /**
       The slice the replacement is addressed to.
       */
      const { sliceIndex, } = edit;
      /**
       The replacements already addressed to the slice.
       */
      const addressed = edits.get(sliceIndex,) ?? [];
      addressed.push({
        group,
        edit,
      },);
      edits.set(
        sliceIndex,
        addressed,
      );
      for (const position of edit.placePositions) {
        replaced.add(replacedKeyOf({
          group,
          position,
          sliceIndex,
        },),);
      }
    }
    /**
     The group's places and the renderings the counter found for them.
     */
    const {
      places: groupPlaces,
      renderings,
    } = settledGroup;
    for (const [position, place,] of groupPlaces.entries()) {
      /**
       This place's rendering.
       */
      const rendering = renderings[position];
      if (rendering?.kind !== 'located')
        continue;
      /**
       The slice the place stands in.
       */
      const { slice, } = place;
      /**
       The index of that slice.
       */
      const { sliceIndex, } = slice;
      /**
       The located places already found in the slice.
       */
      const located = places.get(sliceIndex,) ?? [];
      located.push({
        group,
        position,
        rendering,
      },);
      places.set(
        sliceIndex,
        located,
      );
    }
  }
  return {
    edits,
    places,
    replaced,
  };
}

//endregion Pass consistency apply index
