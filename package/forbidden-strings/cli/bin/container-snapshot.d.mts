/**
 Types of `container-snapshot.mjs`, which the scanner's TypeScript scripts import.
 */
import type { SpawnSyncReturns } from 'node:child_process';

/**
 Failed setup or incomplete evidence never counts as a verification pass; an `Error` subclass at runtime.
 */
export declare const ScannerVerificationError: ErrorConstructor;

/**
 Options of one shell-free management command.
 */
export type RunOptions = {
  readonly command: string;
  readonly args: readonly string[];
  readonly capture?: boolean;
  readonly allowFailure?: boolean;
  readonly transcript?: string;
};

/**
 Run a shell-free management command; allow failures only when their evidence is retained.

 @param options - the command, its arguments and how to treat its output and failure
 @returns the finished process
 */
export function run(options: RunOptions): SpawnSyncReturns<string>;

/**
 Options of one source snapshot.
 */
export type SnapshotOptions = {
  readonly command: readonly string[];
  readonly name: string;
  readonly toolchain?: boolean;
  readonly tool?: string;
  readonly fuzzing?: boolean;
  readonly transform?: (context: {
    readonly context: string;
    readonly scanner: string
  }) => Promise<void>;
};

/**
 What a snapshot left for the caller to run.
 */
export type Snapshot = {
  readonly context: string;
  readonly evidence: string;
  readonly image: string;
  readonly limits: readonly string[];
};

/**
 Bake scanner, engine and optional fuzz sidecar inputs into a content-addressed container.

 @param options - the command the image runs and what goes into it
 @returns the build context, the evidence directory, the image and the container limits
 */
export function snapshot(options: SnapshotOptions): Promise<Snapshot>;
