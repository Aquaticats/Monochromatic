/**
 Type-check a batch of declared-type cases with `tsc` and attribute each
 diagnostic to the case that produced it.

 Writes the cases and a standalone `tsconfig.json` (extending the repo's DOM
 preset, so the same strictness applies: `exactOptionalPropertyTypes`,
 `noUncheckedIndexedAccess`) into a scratch directory under this package,
 runs the workspace `tsc`, and maps each `file(line,col): error` line back to
 the case whose source lines contain it.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import {
  emitCase,
  emitHeader,
} from './declared-type-emit.ts';
import type { DrawnCase, } from './declared-type-generate.ts';

/**
 One emitted case with its runtime result.
 */
export type RunCase = {
  readonly drawn: DrawnCase;
  readonly result: unknown
};

/**
 Diagnostics attributed to one case.
 */
export type CaseFailure = {
  readonly id: number;
  readonly source: string;
  readonly diagnostics: readonly string[];
};

/**
 Emitted module plus the 1-based line span of each case.
 */
export type EmittedModule = {
  readonly source: string;
  readonly spans: readonly {
    readonly id: number;
    readonly first: number;
    readonly last: number
  }[];
};

/**
 Emitted source lines of one case.
 */
type CaseBody = {
  readonly id: number;
  readonly lines: readonly string[]
};

/**
 Emit a whole case module and record where each case landed.

 @param cases - Cases in emission order; `id` is the array position.
 
 @param exportName - Name of the exported case array.

 @returns Module source and case line spans.

 @example
 ```ts
 const emitted = emitModule({ cases, exportName: 'CASES', });
 ```
 */
export function emitModule(
  {
    cases,
    exportName,
  }: {
    readonly cases: readonly RunCase[];
    readonly exportName: string
  },
): EmittedModule {
  /**
   Header lines, which precede every case.
   */
  const header = emitHeader(exportName,);
  /**
   Case bodies with their ids.
   */
  const bodies = cases.map(function body(
    {
      drawn,
      result,
    }: RunCase,
    id: number,
  ): CaseBody {
    return {
      id,
      lines: emitCase({
        drawn,
        id,
        result,
      },),
    };
  },);
  /**
   Line spans computed from cumulative body lengths.
   */
  const {spans} = bodies.reduce<{
    readonly next: number;
    readonly spans: EmittedModule['spans']
  }>(
    function place(
      acc,
      {
        id,
        lines,
      }: CaseBody,
    ) {
      return {
        next: acc.next + lines.length,
        spans: [
          ...acc.spans,
          {
            first: acc.next,
            id,
            last: (acc.next + lines.length) - 1,
          },
        ],
      };
    },
    {
      next: header.length + 1,
      spans: [],
    },
  );
  return {
    source: [
      ...header,
      ...bodies.flatMap(function linesOf(caseBody: CaseBody,) {
        return caseBody.lines;
      },),
      '];',
      '',
    ].join('\n',),
    spans,
  };
}

/**
 Run the workspace `tsc` over one directory's `tsconfig.json`.

 @param dir - Directory holding `tsconfig.json` and the cases.

 @returns Combined compiler output.

 @example
 ```ts
 const output = await runTsc(dir);
 ```
 */
async function runTsc(dir: string,): Promise<string> {
  /**
   Workspace compiler, the same one `lint:types` resolves.
   */
  const tsc = resolve(
    import.meta.dirname,
    '../../../../node_modules/.bin/tsc',
  );
  /**
   Compiler process.
   */
  const child = spawn(
    tsc,
    [
      '--project',
      join(
        dir,
        'tsconfig.json',
      ),
      '--pretty',
      'false',
    ],
    {
      cwd: dir,
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Collected output chunks.
   */
  const chunks: string[] = [];
  child.stdout
    .on(
    'data',
    function collect(chunk: Buffer,) {
      chunks.push(chunk.toString('utf8',),);
    },
    );
  child.stderr
    .on(
    'data',
    function collect(chunk: Buffer,) {
      chunks.push(chunk.toString('utf8',),);
    },
    );
  await once(
    child,
    'close',
  );
  return chunks.join('',);
}

/**
 Type-check cases and attribute diagnostics to cases.

 @param dir - Scratch directory to write into (created if missing).
 
 @param cases - Cases to check.

 @returns Failing cases with their diagnostics, plus diagnostics outside any case.

 @example
 ```ts
 const { failures, stray } = await checkCases({ dir, cases, });
 ```
 */
export async function checkCases(
  {
    dir,
    cases,
  }: {
    readonly dir: string;
    readonly cases: readonly RunCase[]
  },
): Promise<{
  readonly failures: readonly CaseFailure[];
  readonly stray: readonly string[]
}> {
  await mkdir(
    dir,
    { recursive: true, },
  );
  /**
   Emitted module and its case spans.
   */
  const emitted = emitModule({
    cases,
    exportName: 'DECLARED_TYPE_CASES',
  },);
  await writeFile(
    join(
      dir,
      'cases.ts',
    ),
    emitted.source,
  );
  await writeFile(
    join(
      dir,
      'tsconfig.json',
    ),
    `${JSON.stringify(
      {
    compilerOptions: {
      composite: false,
      declaration: false,
      incremental: false,
      isolatedDeclarations: false,
      noEmit: true,
    },
    extends: resolve(
      import.meta.dirname,
      '../../../config/typescript/tsconfig.dom.json',
    ),
    include: ['cases.ts',],
  },
      undefined,
      2,
    )}\n`,
  );
  /**
   Diagnostic lines naming the cases file.
   */
  const diagnostics = (await runTsc(dir,))
    .split('\n',)
    .reduce<string[]>(
    function group(
      acc,
      line,
    ) {
      // Continuation lines are indented and belong to the preceding diagnostic.
      if (line.includes('error TS',))
        acc.push(line,);
      else if (line.startsWith(' ',) && (acc.length > 0))
        acc[acc.length - 1] = `${acc.at(-1,) ?? ''}\n${line}`;
      return acc;
    },
    [],
    );
  /**
   Source lines, for quoting each failing case.
   */
  const sourceLines = emitted.source
    .split('\n',);
  /**
   Diagnostics grouped by case id.
   */
  const byCase = new Map<number, string[]>();
  /**
   Diagnostics that fall outside every case (header or emitter problems).
   */
  const stray: string[] = [];
  for (const diagnostic of diagnostics) {
    /**
     1-based line number of the diagnostic.
     */
    const line = diagnostic.startsWith('cases.ts(',) ? Number(diagnostic.slice(
      'cases.ts('.length,
      diagnostic.indexOf(',',),
    ),) : 0;
    /**
     Case whose span contains the line.
     */
    const span = emitted.spans
      .find(function contains({
        first,
        last,
      },) {
        return (line >= first) && (line <= last);
      },);
    if (span === undefined) {
      stray.push(diagnostic,);
      continue;
    }
    byCase.set(
      span.id,
      [
        ...(byCase.get(span.id,) ?? []),
        diagnostic,
      ],
    );
  }
  return {
    failures: [...byCase,].map(function toFailure([id, messages,],) {
      /**
       Span of this case, for quoting its source.
       */
      const span = emitted.spans[id];
      return {
        diagnostics: messages,
        id,
        source: span === undefined ? '' : sourceLines.slice(
          span.first - 1,
          span.last,
        )
          .join('\n',),
      };
    },),
    stray,
  };
}
