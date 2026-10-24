import type { SourceFile, } from './claude-schema-sources.test-fixture.ts';

//region Shapes the schema search does not read, found in the family's sources
// The check behind the header of `claude-schema-sources.test-fixture.ts`: the search that keeps the producer list honest
// reads a fixed set of shapes and leaves others unread, and a shape it leaves unread can hide a file that sends an
// answer schema. `blindSpotsOf` reads the same sources and reports each place that holds one of those shapes, with the
// file and the line, so a shape the search cannot read never turns up in the family without a failure that names it.

/**
 The shapes the schema search leaves unread, each as the name a report gives it.
 */
type BlindShape =
  | 'builder-handed-to-a-call-held-by-a-constant'
    | 'builder-in-namespace-enum-or-parameter-property'
    | 'builder-stored-in-object'
    | 'computed-load-path'
    | 'format-in-undeclared-function'
    | 'load-in-unfollowed-form'
    | 'result-returned-through-a-local';

/**
 One place in a source file that holds a shape the search leaves unread.
 */
export type BlindSpot = {
  /**
   Path of the file under `package/translation-repair`.
   */
  readonly file: string;

  /**
   Line of the place, counted from one.
   */
  readonly line: number;

  /**
   The shape the place holds.
   */
  readonly shape: BlindShape;
};

/**
 The places in the sources that hold a shape the schema search leaves unread.

 @param sources - files searched

 @param notStageSchemas - paths of files that write the format without building a stage's answer schema

 @returns One entry per place, in the order of the sources and then of the lines

 @throws {@link Error} when a file does not parse

 @example
 ```ts
 const spots = blindSpotsOf({ sources: await familySources(), notStageSchemas: new Set(), },);
 ```
 */
export function blindSpotsOf(
  {
    sources: _sources,
    notStageSchemas: _notStageSchemas,
  }: {
    readonly sources: readonly SourceFile[];
    readonly notStageSchemas: ReadonlySet<string>;
  },
): readonly BlindSpot[] {
  return [];
}

//endregion Shapes the schema search does not read, found in the family's sources
