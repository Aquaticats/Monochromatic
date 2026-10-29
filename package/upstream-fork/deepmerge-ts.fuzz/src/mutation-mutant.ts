/**
 Pure helpers over a StrykerJS JSON report for the `mutation:*` tasks:
 parse the report, pick mutants by status or id, splice one mutant into its
 file's source, and classify which sidecar files detected it.

 Stryker locations are 1-based lines and 1-based columns
 (mutation-testing-report-schema).

 @module
 */

/**
 Whether a parsed JSON value is an object with string keys.

 @param value - Parsed JSON value.

 @returns True for non-array objects.

 @example
 ```ts
 isRecord(JSON.parse('{}',),); // true
 ```
 */
function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 One position in a source file, as Stryker records it.
 */
export type MutantPosition = {
  readonly line: number;
  readonly column: number;
};

/**
 One mutant of the report.
 */
export type Mutant = {
  readonly id: string;
  readonly mutatorName: string;
  readonly replacement: string;
  readonly status: string;
  readonly location: {
    readonly start: MutantPosition;
    readonly end: MutantPosition;
  };
};

/**
 Mutated files keyed by path relative to the checkout, with their original
 source.
 */
export type MutationReport = {
  readonly files: Readonly<Record<string, {
    readonly source: string;
    readonly mutants: readonly Mutant[];
  }>>;
};

/**
 A mutant with the file it belongs to.
 */
export type SelectedMutant = {
  readonly file: string;
  readonly source: string;
  readonly mutant: Mutant;
};

/**
 How the sidecar treated one mutant: detected by a file that specifies
 behaviour, failed only files that pin current behaviour (`known-defect*` or
 machine-local), or passed everywhere.
 */
export type SweepVerdict = 'detected' | 'pinningOnly' | 'survived';

/**
 Error for a report or location that does not match the expected shape.
 */
export class MutationReportError extends Error {
  /**
   @param message - What was malformed.
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'MutationReportError';
  }
}

/**
 Read one location position.

 @param value - Raw `start` or `end` object.

 @returns Validated position.

 @throws {@link MutationReportError} When line or column is not a positive integer.

 @example
 ```ts
 parsePosition({ line: 1, column: 1, });
 ```
 */
function parsePosition(value: unknown,): MutantPosition {
  /**
   Line field.
   */
  const line: unknown = isRecord(value,) ? value.line : undefined;
  /**
   Column field.
   */
  const column: unknown = isRecord(value,) ? value.column : undefined;
  if (((typeof line) !== 'number') || ((typeof column) !== 'number')
    || (!Number.isInteger(line,))
    || (!Number.isInteger(column,))
    || (line < 1)
    || (column < 1))
    throw new MutationReportError(`malformed mutant position ${JSON.stringify(value,)}`,);
  return {
    column,
    line,
  };
}

/**
 Read one mutant entry.

 @param value - Raw mutant object.

 @returns Validated mutant; a missing replacement means deletion.

 @throws {@link MutationReportError} When a required field is missing.

 @example
 ```ts
 parseMutant(entry);
 ```
 */
function parseMutant(value: unknown,): Mutant {
  if ((!isRecord(value,)) || (!isRecord(value.location,)))
    throw new MutationReportError(`malformed mutant ${JSON.stringify(value,)}`,);
  /**
   Fields read once so the checks narrow them.
   */
  const {
    id,
    mutatorName,
    replacement,
    status,
  } = value;
  if (((typeof id) !== 'string') || ((typeof mutatorName) !== 'string')
    || ((typeof status) !== 'string'))
    throw new MutationReportError(`mutant without id, mutatorName, or status: ${JSON.stringify(value,)}`,);
  return {
    id,
    location: {
      end: parsePosition(value.location
        .end,),
      start: parsePosition(value.location
        .start,),
    },
    mutatorName,
    replacement: ((typeof replacement) === 'string') ? replacement : '',
    status,
  };
}

/**
 Parse Stryker's `mutation.json`.

 @param text - Report file contents.

 @returns Files with their source and mutants.

 @throws {@link MutationReportError} When the report does not have the schema's shape.

 @example
 ```ts
 const report = parseMutationReport(await readFile('/out/mutation.json', 'utf8',),);
 ```
 */
export function parseMutationReport(text: string,): MutationReport {
  /**
   Parsed JSON.
   */
  const raw: unknown = JSON.parse(text,);
  if ((!isRecord(raw,)) || (!isRecord(raw.files,)))
    throw new MutationReportError('report has no files object',);
  return {
    files: Object.fromEntries(Object.entries(raw.files,)
      .map(function parseFile([file, entry,],) {
        if (!isRecord(entry,))
          throw new MutationReportError(`report entry ${file} is not an object`,);
        /**
         Fields read once so the checks narrow them.
         */
        const {
          mutants,
          source,
        } = entry;
        if (((typeof source) !== 'string') || (!Array.isArray(mutants,)))
          throw new MutationReportError(`report entry ${file} has no source or mutants`,);
        return [
          file,
          {
            mutants: mutants.map(function parseEntry(mutant: unknown,) {
              return parseMutant(mutant,);
            },),
            source,
          },
        ] as const;
      },),),
  };
}

/**
 Offset of a Stryker position in `source`.

 @param source - Original file text.

 @param position - 1-based line and column.

 @returns UTF-16 offset of that position.

 @throws {@link MutationReportError} When the line does not exist.

 @example
 ```ts
 offsetOf({ source: 'a\nbc', position: { line: 2, column: 2, }, }); // 3
 ```
 */
export function offsetOf(
  {
    source,
    position,
  }: {
    readonly source: string;
    readonly position: MutantPosition;
  },
): number {
  /**
   Source lines; the position's line must be one of them.
   */
  const lines = source.split('\n',);
  if (position.line > lines.length)
    throw new MutationReportError(`line ${String(position.line,)} is past the end of the source`,);
  /**
   Offset where the position's line starts: every earlier line plus its newline.
   */
  const lineStart = lines
    .slice(
    0,
    position.line - 1,
    )
    .reduce(
    function addLine(
      sum,
      line,
    ) {
      return sum + line.length
        + 1;
    },
    0,
    );
  return (lineStart + position.column) - 1;
}

/**
 Source text with one mutant spliced in.

 @param selected - Mutant and its file's original source.

 @returns Mutated source.

 @example
 ```ts
 const text = applyMutant(selected);
 ```
 */
export function applyMutant(selected: SelectedMutant,): string {
  /**
   Start of the replaced span.
   */
  const start = offsetOf({
    position: selected.mutant
      .location
      .start,
    source: selected.source,
  },);
  /**
   End of the replaced span.
   */
  const end = offsetOf({
    position: selected.mutant
      .location
      .end,
    source: selected.source,
  },);
  return `${selected.source
    .slice(
      0,
      start,
    )}${selected.mutant
      .replacement}${selected.source
        .slice(end,)}`;
}

/**
 Mutants with one of `statuses`, optionally narrowed to `ids`.

 @param report - Parsed report.

 @param statuses - Stryker statuses to keep, such as `Survived` and `NoCoverage`.

 @param ids - Mutant ids to keep; empty keeps every id.

 @returns Matching mutants in report order.

 @example
 ```ts
 selectMutants({ report, statuses: ['Survived',], ids: [], });
 ```
 */
export function selectMutants(
  {
    report,
    statuses,
    ids,
  }: {
    readonly report: MutationReport;
    readonly statuses: readonly string[];
    readonly ids: readonly string[];
  },
): readonly SelectedMutant[] {
  return Object.entries(report.files,)
    .flatMap(function mutantsOf([file, entry,],) {
      return entry.mutants
        .filter(function wanted(mutant,) {
          return statuses.includes(mutant.status,) && ((ids.length === 0) || ids.includes(mutant.id,));
        },)
        .map(function withFile(mutant,) {
          return {
            file,
            mutant,
            source: entry.source,
          };
        },);
    },);
}

/**
 Whether a sidecar file pins current behaviour instead of specifying it.

 @param file - Sidecar test file path.

 @returns True for `known-defect*` and machine-local files.

 @example
 ```ts
 isPinningFile('src/known-defect.unit.test.ts'); // true
 ```
 */
export function isPinningFile(file: string,): boolean {
  return file.includes('known-defect',) || file.includes('.local.',);
}

/**
 Classify a mutant by the sidecar files that failed against it.

 @param failed - Files that failed or timed out.

 @returns Verdict separating specifying files from pinning ones.

 @example
 ```ts
 sweepVerdict(['src/known-defect.unit.test.ts',]); // 'pinningOnly'
 ```
 */
export function sweepVerdict(failed: readonly string[],): SweepVerdict {
  if (failed.some(function specifies(file,) {
    return !isPinningFile(file,);
  },))
    return 'detected';
  return (failed.length > 0) ? 'pinningOnly' : 'survived';
}
