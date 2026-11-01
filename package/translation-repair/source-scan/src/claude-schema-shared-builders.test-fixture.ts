//region Shared builders
// Functions that build an answer schema when called, so one file holds the schema text and several files each make a
// schema of their own. A search for the files that hold the schema text finds the file that holds the builder and no
// caller. The source search in `claude-schema-source-search.unit.test.ts` holds this list to the family's source, and
// the corpus-pass package holds its named callers to the producer list.

/**
 One shared builder, the file that declares it and the files that call it.
 */
type SharedBuilder = {
  /**
   The function's name.
   */
  readonly builder: string;

  /**
   Path of the file that declares it, which is a producer in the list.
   */
  readonly declaredIn: string;

  /**
   Each file that calls it, with the name of the schema it makes where the list holds that schema by name.
   */
  readonly callers: readonly {
    readonly file: string;
    readonly schemaName?: string;
  }[];
};

/**
 Every builder of an answer schema that more than one file can call, and every file that calls it. A stage that adds a
 call, or a builder of its own, fails the case that reads these until it is listed here, and a caller that names its
 schema fails the case on named callers until the producer list holds a schema of that name. The consistency
 builders make their schemas per ask from the labels shown, so the producer list holds one sample of each, and their
 callers carry no schema name here.
 */
export const SHARED_BUILDERS: readonly SharedBuilder[] = [
  {
    builder: 'consistencyChoiceResponseFormat',
    declaredIn: 'consistency/src/consistency-choice.ts',
    callers: [
      { file: 'corpus-pass/src/corpus-run/pass-consistency-carry-ask.ts', },
      { file: 'corpus-pass/src/corpus-run/pass-consistency-run-off.ts', },
    ],
  },
  {
    builder: 'consistencyResponseFormat',
    declaredIn: 'consistency/src/consistency-wire.ts',
    callers: [
      { file: 'corpus-pass/src/corpus-run/pass-consistency-ask.ts', },
    ],
  },
  {
    builder: 'contestResponseFormat',
    declaredIn: 'lane-contest/src/contest-ballot-wire.ts',
    callers: [
      {
        file: 'consolidate/src/consolidate-gate-stage.ts',
        schemaName: 'consolidate_gate',
      },
      {
        file: 'consolidate/src/consolidation-polish-gate-stage.ts',
        schemaName: 'consolidation_polish_gate',
      },
      {
        file: 'lane-contest/src/lane-contest-stage.ts',
        schemaName: 'lane_contest',
      },
    ],
  },
];

//endregion Shared builders
