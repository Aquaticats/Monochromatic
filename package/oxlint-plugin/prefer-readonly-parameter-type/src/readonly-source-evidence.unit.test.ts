import {
  mkdtempSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { resolve, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { Context, } from '@oxlint/plugins';

import {
  readonlySourceEvidence,
  readonlySourceEvidenceCacheStats,
} from '../dist/final/node/index.mjs';

/**
 Fixture source inputs used by cache-sharing test.
 */
const FIXTURE = {
  sourceRoot: resolve(
    import.meta.dirname,
    '../../../test-fixture/oxlint-no-restricted-syntax/src',
  ),
  sourceText: `export function readValue(input: { value: string }): string {\n  return input.value;\n}\n`,
} as const;

/**
 Disposable configured directory used by one evidence test.
 
 @example
 ```ts
 using directory = disposableDirectory();
 ```
 */
type DisposableDirectory = {
  readonly path: string;
  readonly [Symbol.dispose]: () => void;
};

/**
 Evidence-cache measurements across four rule contexts.
 
 @example
 ```ts
 const measurement = measureSharedEvidence({ filePath });
 ```
 */
type SharedEvidenceMeasurement = {
  readonly computationDelta: number;
  readonly missDelta: number;
  readonly hitDelta: number;
  readonly sameResult: boolean;
};

/**
 Creates disposable directory inside configured fixture source root.
 
 @returns directory removed when test scope ends.
 
 @example
 ```ts
 using directory = disposableDirectory();
 ```
 */
function disposableDirectory(): DisposableDirectory {
  /**
   Unique configured source directory.
   */
  const path = mkdtempSync(resolve(FIXTURE.sourceRoot, 'readonly-evidence-',),);
  return {
    path,
    [Symbol.dispose](): void {
      rmSync(path, {
        recursive: true,
        force: true,
      },);
    },
  };
}

/**
 Minimal rule context needed before reporting begins.
 
 @param filePath - Configured TypeScript source path.
 
 @returns distinct foreign context carrying shared source snapshot.
 
 @example
 ```ts
 evidenceContext({ filePath });
 ```
 */
function evidenceContext({
  filePath,
  sourceText,
}: {
  readonly filePath: string;
  readonly sourceText: string;
}): ForeignBorrowed<Context> {
  return {
    filename: filePath,
    sourceCode: {
      text: sourceText,
      hasBOM: false,
    },
  } as ForeignBorrowed<Context>;
}

/**
 Writes configured source consumed by evidence analysis.
 
 @param directoryPath - Disposable configured directory.
 
 @returns written TypeScript source path.
 
 @example
 ```ts
 writeEvidenceSource({ directoryPath });
 ```
 */
function writeEvidenceSource({ directoryPath, }: { readonly directoryPath: string; },): string {
  /**
   Source path admitted by fixture TypeScript project.
   */
  const filePath = resolve(directoryPath, 'input.ts',);
  writeFileSync(filePath, FIXTURE.sourceText,);
  return filePath;
}

/**
 Requests shared evidence from four distinct contexts.
 
 @param filePath - Configured source path shared by contexts.
 
 @returns repeated source evidence results.
 
 @example
 ```ts
 analyzeFromFourContexts({ filePath });
 ```
 */
function analyzeFromFourContexts({
  filePath,
}: {
  readonly filePath: string;
}): readonly ReturnType<typeof readonlySourceEvidence>[] {
  return [
    'preference',
    'mutation',
    'opaque-effect',
    'effect-contract',
  ].map(function analyzeFromDistinctContext(): ReturnType<typeof readonlySourceEvidence> {
    return readonlySourceEvidence({
      context: evidenceContext({
        filePath,
        sourceText: FIXTURE.sourceText,
      },),
    },);
  },);
}

/**
 Completes cache measurement after initial counter snapshot.
 
 @param filePath - Configured source path shared by contexts.
 
 @param before - Counters before evidence requests.
 
 @returns cache deltas and result identity.
 
 @example
 ```ts
 measureAfterBefore({ filePath, before });
 ```
 */
function measureAfterBefore({
  filePath,
  before,
}: {
  readonly filePath: string;
  readonly before: ReturnType<typeof readonlySourceEvidenceCacheStats>;
}): SharedEvidenceMeasurement {
  /**
   Results returned to four distinct rule contexts.
   */
  const results = analyzeFromFourContexts({ filePath, },);
  return measureAfterResults({
    before,
    results,
  },);
}

/**
 Calculates cache deltas after all evidence requests.
 
 @param before - Counters before evidence requests.
 
 @param results - Evidence results returned to distinct contexts.
 
 @returns cache deltas and result identity.
 
 @example
 ```ts
 measureAfterResults({ before, results });
 ```
 */
function measureAfterResults({
  before,
  results,
}: {
  readonly before: ReturnType<typeof readonlySourceEvidenceCacheStats>;
  readonly results: readonly ReturnType<typeof readonlySourceEvidence>[];
}): SharedEvidenceMeasurement {
  /**
   Cache counters after all reporter contexts requested evidence.
   */
  const after = readonlySourceEvidenceCacheStats();
  return {
    computationDelta: after.computations - before.computations,
    missDelta: after.misses - before.misses,
    hitDelta: after.hits - before.hits,
    sameResult: results.every(function sharesFirst(result,): boolean {
      return result === results[0];
    },),
  };
}

/**
 Measures shared evidence reuse for configured source.
 
 @param filePath - Configured source path shared by contexts.
 
 @returns cache deltas and result identity.
 
 @example
 ```ts
 measureSharedEvidence({ filePath });
 ```
 */
function measureSharedEvidence({
  filePath,
}: {
  readonly filePath: string;
}): SharedEvidenceMeasurement {
  /**
   Cache counters before exact source enters process.
   */
  const before = readonlySourceEvidenceCacheStats();
  return measureAfterBefore({
    filePath,
    before,
  },);
}

/**
 Completes changed-source measurement after initial cache snapshot.
 
 @param filePath - Configured source path receiving changed overlay.
 
 @param before - Counters after original source analysis.
 
 @returns cache deltas caused by changed source text.
 
 @example
 ```ts
 measureChangedAfterBefore({ filePath, before });
 ```
 */
function measureChangedAfterBefore({
  filePath,
  before,
}: {
  readonly filePath: string;
  readonly before: ReturnType<typeof readonlySourceEvidenceCacheStats>;
}): Pick<SharedEvidenceMeasurement, 'computationDelta' | 'missDelta'> {
  readonlySourceEvidence({
    context: evidenceContext({
      filePath,
      sourceText: `${FIXTURE.sourceText}\nexport const changed = true;\n`,
    },),
  },);
  /**
   Cache counters after changed overlay entered semantic bridge.
   */
  const after = readonlySourceEvidenceCacheStats();
  return {
    computationDelta: after.computations - before.computations,
    missDelta: after.misses - before.misses,
  };
}

/**
 Measures invalidation when one source path receives changed text.
 
 @param filePath - Configured source path receiving both overlays.
 
 @returns cache deltas caused by changed source text.
 
 @example
 ```ts
 measureChangedSource({ filePath });
 ```
 */
function measureChangedSource({
  filePath,
}: {
  readonly filePath: string;
}): Pick<SharedEvidenceMeasurement, 'computationDelta' | 'missDelta'> {
  readonlySourceEvidence({
    context: evidenceContext({
      filePath,
      sourceText: FIXTURE.sourceText,
    },),
  },);
  /**
   Cache counters after original source analysis.
   */
  const before = readonlySourceEvidenceCacheStats();
  return measureChangedAfterBefore({
    filePath,
    before,
  },);
}

await describe({
  name: readonlySourceEvidence.name,
  children: [
    it({
      name: 'computes once for four distinct rule contexts over one snapshot',
      fn: async () => {
        /**
         Disposable directory removed after evidence assertions.
         */
        using directory = disposableDirectory();
        /**
         Cache measurements over written configured source.
         */
        const measurement = measureSharedEvidence({
          filePath: writeEvidenceSource({ directoryPath: directory.path, },),
        },);
        expect(measurement.computationDelta,).toBe(1,);
        expect(measurement.missDelta,).toBe(1,);
        expect(measurement.hitDelta,).toBe(3,);
        expect(measurement.sameResult,).toBe(true,);
      },
    },),
    it({
      name: 'recomputes when one source path receives changed text',
      fn: async () => {
        /**
         Disposable directory removed after invalidation assertions.
         */
        using directory = disposableDirectory();
        /**
         Cache measurements across original and changed overlays.
         */
        const measurement = measureChangedSource({
          filePath: writeEvidenceSource({ directoryPath: directory.path, },),
        },);
        expect(measurement.computationDelta,).toBe(1,);
        expect(measurement.missDelta,).toBe(1,);
      },
    },),
  ],
},);
