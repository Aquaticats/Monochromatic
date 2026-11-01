/**
 Edits to a plain JSON value addressed by path, each returning a changed copy,
 for tests that break one valid record in exactly one way without a typed view
 of the record.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { isJsonRecord, } from '../../dist/final/node/index.mjs';

//region JSON edits

/**
 Where a value sits in a JSON record: object keys and array positions, in order.

 @example
 ```ts
 const path: JsonPath = ['groups', 0, 'verdict', 'usable',];
 ```
 */
type JsonPath = readonly (string | number)[];

/**
 One change to a JSON record.

 @example
 ```ts
 const edit: JsonEdit = { kind: 'set', path: ['findings',], value: [], };
 ```
 */
export type JsonEdit =
  | {
    /**
     Puts a value at the path, replacing what is there or adding the key.
     */
    readonly kind: 'set';

    /**
     Where the value goes.
     */
    readonly path: JsonPath;

    /**
     The value.
     */
    readonly value: unknown;
  }
    | {
    /**
     Takes the key at the path out of its object.
     */
      readonly kind: 'remove';

    /**
     Where the key sits.
     */
      readonly path: JsonPath;
    }
    | {
    /**
     Adds a value to the end of the array at the path.
     */
      readonly kind: 'append';

    /**
     Where the array sits.
     */
      readonly path: JsonPath;

    /**
     The value.
     */
      readonly value: unknown;
    }
    | {
    /**
     Takes the last value off the array at the path.
     */
      readonly kind: 'dropLast';

    /**
     Where the array sits.
     */
      readonly path: JsonPath;
    };

/**
 Builds an edit that puts a value at a path.

 @param path - where the value goes

 @param value - the value

 @returns The edit

 @example
 ```ts
 const edit = setAt({ path: ['groups', 0, 'verdict', 'usable',], value: 3, },);
 ```
 */
export function setAt({
  path,
  value,
}: {
  readonly path: JsonPath;
  readonly value: unknown
},): JsonEdit {
  return {
    kind: 'set',
    path,
    value,
  };
}

/**
 Builds an edit that takes a key out of its object.

 @param path - where the key sits

 @returns The edit

 @example
 ```ts
 const edit = removeAt({ path: ['groups', 0, 'repeat', 'wording',], },);
 ```
 */
export function removeAt({ path, }: { readonly path: JsonPath; },): JsonEdit {
  return {
    kind: 'remove',
    path,
  };
}

/**
 Builds an edit that adds a value to an array.

 @param path - where the array sits

 @param value - the value

 @returns The edit

 @example
 ```ts
 const edit = appendAt({ path: ['slices',], value: { sliceIndex: 0, text: 'x', }, },);
 ```
 */
export function appendAt({
  path,
  value,
}: {
  readonly path: JsonPath;
  readonly value: unknown
},): JsonEdit {
  return {
    kind: 'append',
    path,
    value,
  };
}

/**
 Builds an edit that takes the last value off an array.

 @param path - where the array sits

 @returns The edit

 @example
 ```ts
 const edit = dropLastAt({ path: ['slices',], },);
 ```
 */
export function dropLastAt({ path, }: { readonly path: JsonPath; },): JsonEdit {
  return {
    kind: 'dropLast',
    path,
  };
}

/**
 What sits at a path.

 @param root - the record

 @param path - where to look

 @returns The value there

 @throws {@link Error} when the path leaves the record

 @example
 ```ts
 const usable = valueAt({ root, path: ['groups', 0, 'verdict', 'usable',], },);
 ```
 */
export function valueAt({
  root,
  path,
}: {
  readonly root: unknown;
  readonly path: JsonPath
},): unknown {
  return path.reduce(
    function stepInto(
      held: unknown,
      step,
    ): unknown {
      if (Array.isArray(held,) && ((typeof step) === 'number'))
        return held[step];
      if (isJsonRecord(held,) && ((typeof step) === 'string'))
        return held[step];
      throw new Error(`the path leaves the record at ${String(step,)}`,);
    },
    root,
  );
}

/**
 A copy of a record with the value at a path replaced by a function of it.

 @param root - the record

 @param path - where the value sits, empty for the record itself

 @param change - makes the new value from the old

 @returns The changed copy; the original is untouched

 @throws {@link Error} when the path leaves the record

 @example
 ```ts
 const copy = changedAt({ root, path: ['groups',], change: function none() { return []; }, },);
 ```
 */
function changedAt(
  {
    root,
    path,
    change,
  }: {
    readonly root: unknown;
    readonly path: JsonPath;
    readonly change: (held: unknown,) => unknown;
  },
): unknown {
  /**
   First step, and the rest after it.
   */
  const [step, ...rest] = path;
  if (step === undefined)
    return change(root,);
  if (Array.isArray(root,) && ((typeof step) === 'number')) {
    return root.map(function copyOne(
      item: unknown,
      at: number,
    ): unknown {
      return (at === step)
        ? changedAt({
          root: item,
          path: rest,
          change,
        },)
        : item;
    },);
  }
  if (isJsonRecord(root,) && ((typeof step) === 'string')) {
    return Object.fromEntries(Object.entries(root,)
      .map(function copyOne([key, item,],): readonly [
        string,
        unknown
      ] {
        return [
          key,
          (key === step)
            ? changedAt({
              root: item,
              path: rest,
              change,
            },)
            : item,
        ];
      },),);
  }
  throw new Error(`the path leaves the record at ${String(step,)}`,);
}

/**
 Applies edits to a record, one after another.

 @param root - the record

 @param edits - the changes, in order

 @returns The changed copy; the original is untouched

 @throws {@link Error} when an edit's path leaves the record

 @example
 ```ts
 const broken = editedBy({ root, edits: [setAt({ path: ['findings',], value: 3, },),], },);
 ```
 */
export function editedBy({
  root,
  edits,
}: {
  readonly root: unknown;
  readonly edits: readonly JsonEdit[]
},): unknown {
  return edits.reduce(
    function applyOne(
      held: unknown,
      edit,
    ): unknown {
      /**
       The key or position the edit ends on, and the path to the record holding it.
       */
      const last = edit.path
        .at(-1,);
      /**
       Path to the record holding that key or position.
       */
      const parent = edit.path
        .slice(
        0,
        -1,
        );
      if ((edit.kind === 'append') || (edit.kind === 'dropLast')) {
        return changedAt({
          root: held,
          path: edit.path,
          change: function change(array: unknown,): unknown {
            if (!Array.isArray(array,))
              throw new Error('the path does not end on an array',);
            /**
             The array's items, typed as the unknown values they are.
             */
            const items: readonly unknown[] = array;
            return (edit.kind === 'append') ? [
              ...items,
              edit.value,
            ] : items.slice(
              0,
              -1,
            );
          },
        },);
      }
      if (last === undefined)
        throw new Error('an edit needs a path to a key',);
      return changedAt({
        root: held,
        path: parent,
        change: function change(container: unknown,): unknown {
          if (Array.isArray(container,) && ((typeof last) === 'number')) {
            if (edit.kind === 'remove')
              throw new Error('a position is not removed from an array, drop it instead',);
            return container.map(function copyOne(
              item: unknown,
              at: number,
            ): unknown {
              return (at === last) ? edit.value : item;
            },);
          }
          if (isJsonRecord(container,) && ((typeof last) === 'string')) {
            if (edit.kind === 'remove') {
              return Object.fromEntries(Object.entries(container,)
                .filter(function kept([key,],): boolean {
                  return key !== last;
                },),);
            }
            return {
              ...container,
              [last]: edit.value,
            };
          }
          throw new Error(`the path leaves the record at ${String(last,)}`,);
        },
      },);
    },
    root,
  );
}

//endregion JSON edits
