import {
  forwardersIn,
  type FunctionAt,
} from './claude-schema-blind-forwarders.test-fixture.ts';
import {
  buildersIn,
  familyOf,
  type SourceFile,
} from './claude-schema-sources.test-fixture.ts';
import { usagesIn, } from './claude-schema-usage.test-fixture.ts';

//region The forwarders of a set of sources
// What `claude-schemas.unit.test.ts` reads to keep the list of forwarders honest.

/**
 The functions that hand a sender a builder call made from their own parameters, which no builder or wrapper is. Their
 callers write the arguments of the schema, so the search lists them.

 @param sources - files searched

 @param notStageSchemas - paths of files that write the format without building a stage's answer schema

 @returns One entry per function, in the order of the sources

 @throws {@link Error} when a file does not parse

 @example
 ```ts
 const forwarders = forwardersOf({ sources: await familySources(), notStageSchemas: new Set(), },);
 ```
 */
export function forwardersOf(
  {
    sources,
    notStageSchemas,
  }: {
    readonly sources: readonly SourceFile[];
    readonly notStageSchemas: ReadonlySet<string>;
  },
): readonly FunctionAt[] {
  /**
   Each file read once.
   */
  const family = familyOf({ sources, },);
  return forwardersIn({
    family,
    builders: buildersIn({
      family,
      notStageSchemas,
    },),
    usages: usagesIn({ family, },),
  },);
}

//endregion The forwarders of a set of sources
