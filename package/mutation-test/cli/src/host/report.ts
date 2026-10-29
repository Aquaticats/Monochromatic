/**
 Final run report: native JSON schema plus terminal findings.
 
 No mutation score anywhere by design; the report carries raw totals,
 per-mutant records with provenance, and infra errors.
 
 @example
 ```ts
 const report = buildRunReport({ outcome, packagePath: 'package/module/fs-path' });
 ```
 */

import type {
  IgnoredMutant,
} from '../engine/enumerate.ts';
import type { MutantStatus, } from '../engine/types.ts';
import type { RunOutcome, } from './orchestrate.ts';

/**
 Current run-report schema version.
 */
export const RUN_REPORT_SCHEMA_VERSION = 1;

/**
 Raw totals per final status plus suppressions.
 */
export type RunTotals = {
  readonly killed: number;
  readonly survived: number;
  readonly timeout: number;
  readonly compileError: number;
  readonly runtimeError: number;
  readonly ignored: number;
};

/**
 One mutant record in the final report.
 */
export type RunMutantRecord = {
  readonly id: string;
  readonly file: string;
  readonly line: number;
  readonly column: number;
  readonly operator: string;
  readonly original: string;
  readonly replacement: string;
  readonly description: string;
  readonly status: MutantStatus;
  readonly position: number;
  readonly rerunCount: number;
  readonly confirmed: boolean;
};

/**
 Complete native run report.
 */
export type RunReport = {
  readonly schemaVersion: typeof RUN_REPORT_SCHEMA_VERSION;
  readonly packagePath: string;
  readonly totals: RunTotals;
  readonly mutants: readonly RunMutantRecord[];
  readonly ignored: readonly IgnoredMutant[];
  readonly infraErrors: readonly string[];
  readonly shardCount: number;
};

/**
 Builds the native JSON run report from an orchestration outcome.
 
 @param options - Outcome and package identity.
 
 @returns Serialisable run report.
 
 @example
 ```ts
 const report = buildRunReport({ outcome, packagePath });
 ```
 */
export function buildRunReport(options: {
  readonly outcome: RunOutcome;
  readonly packagePath: string;
},): RunReport {
  /**
   Totals accumulated across final results.
   */
  const totals = options.outcome
    .results
    .reduce(
    function addResult(
      accumulated,
      result,
    ): RunTotals {
      return {
        killed: accumulated.killed + (result.status === 'killed' ? 1 : 0),
        survived: accumulated.survived + (result.status === 'survived' ? 1 : 0),
        timeout: accumulated.timeout + (result.status === 'timeout' ? 1 : 0),
        compileError: accumulated.compileError
          + (result.status === 'compileError' ? 1 : 0),
        runtimeError: accumulated.runtimeError
          + (result.status === 'runtimeError' ? 1 : 0),
        ignored: accumulated.ignored,
      };
    },
    {
      killed: 0,
      survived: 0,
      timeout: 0,
      compileError: 0,
      runtimeError: 0,
      ignored: options.outcome
        .ignored
        .length,
    },
    );

  return {
    schemaVersion: RUN_REPORT_SCHEMA_VERSION,
    packagePath: options.packagePath,
    totals,
    mutants: options.outcome
      .results
      .map(function toRecord(result,): RunMutantRecord {
        return {
          id: result.mutant
            .id,
          file: result.mutant
            .file,
          line: result.mutant
            .line,
          column: result.mutant
            .column,
          operator: result.mutant
            .operator,
          original: result.mutant
            .original,
          replacement: result.mutant
            .replacement,
          description: result.mutant
            .description,
          status: result.status,
          position: result.position,
          rerunCount: result.rerunCount,
          confirmed: result.confirmed,
        };
      },),
    ignored: options.outcome
      .ignored,
    infraErrors: options.outcome
      .infraErrors,
    shardCount: options.outcome
      .shardCount,
  };
}

/**
 Formats terminal output: totals plus survivor and timeout findings.
 
 @param report - Completed run report.
 
 @returns Human-readable multi-line summary.
 
 @example
 ```ts
 console.log(formatTerminalSummary(report));
 ```
 */
export function formatTerminalSummary(report: RunReport,): string {
  /**
   Actionable findings: survivors and timeouts.
   */
  const findings = report.mutants
    .filter(function actionable(record,): boolean {
      return (record.status === 'survived') || (record.status === 'timeout');
    },)
    .map(function toLine(record,): string {
      return `${record.status === 'survived' ? 'Survived' : 'Timeout'}: ${record.file}:${
        String(record.line,)
      }:${String(record.column,)} ${record.operator} ${record.description}${
        record.confirmed ? '' : ' (unconfirmed)'
      }`;
    },);
  /**
   Infra error lines, when any.
   */
  const infra = report.infraErrors
    .map(function toLine(message,): string {
      return `Infra: ${message}`;
    },);

  return [
    ...findings,
    ...infra,
    `Killed: ${String(report.totals
      .killed,)}`,
    `Survived: ${String(report.totals
      .survived,)}`,
    `Timeout: ${String(report.totals
      .timeout,)}`,
    `CompileError: ${String(report.totals
      .compileError,)}`,
    `RuntimeError: ${String(report.totals
      .runtimeError,)}`,
    `Ignored: ${String(report.totals
      .ignored,)}`,
    `Shards: ${String(report.shardCount,)}`,
  ]
    .join('\n',);
}
