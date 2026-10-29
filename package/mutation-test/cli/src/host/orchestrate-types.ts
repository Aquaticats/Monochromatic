/**
 Types shared by the orchestrator, report writer, and CLI.
 
 @example
 ```ts
 import type { RunOutcome } from './orchestrate-types.ts';
 ```
 */

import type { IgnoredMutant, } from '../engine/enumerate.ts';
import type {
  Mutant,
  MutantStatus,
} from '../engine/types.ts';
import type { EnumerationInputs, } from './enumerate-package.ts';
import type { ShardResources, } from './podman.ts';

/**
 Final per-mutant outcome with provenance.
 */
export type FinalMutantResult = {
  readonly mutant: Mutant;
  readonly status: MutantStatus;
  readonly position: number;
  readonly rerunCount: number;
  readonly confirmed: boolean;
  readonly detail: string;
};

/**
 Full run outcome consumed by the report writer.
 */
export type RunOutcome = {
  readonly results: readonly FinalMutantResult[];
  readonly ignored: readonly IgnoredMutant[];
  readonly infraErrors: readonly string[];
  readonly shardCount: number;
};

/**
 Options controlling one orchestrated run.
 */
export type OrchestrateOptions = EnumerationInputs & {
  shardSize: number;
  containers: number;
  resources: ShardResources;
  selinuxRelabel: boolean;
  skipImageBuild: boolean;
  timeoutFloorMs: number;
  timeoutFactor: number;
};

/**
 Mutant lookup entry pairing a mutant with its selected tests.
 */
export type MutantEntry = {
  readonly mutant: Mutant;
  readonly tests: readonly string[];
};
