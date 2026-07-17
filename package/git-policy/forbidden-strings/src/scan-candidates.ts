/**
 * Forbidden-strings scanner process adapter.
 *
 * @module
 */
import {
  isAbsolute,
  relative,
  resolve,
  sep,
} from 'node:path';
import type {
  CandidateFile,
  PolicyFinding,
} from '@monochromatic-dev/git-policy-api/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import nanoSpawn, { SubprocessError, } from 'nano-spawn';
import { ForbiddenStringsPluginError, } from './errors.ts';
import { materializeCandidates, } from './materialize-candidates.ts';
import { parseScannerOutput, } from './scanner-output.ts';

/**
 * Repository paths skipped by scanner `--all` to prevent canonical source self-matches.
 */
const SCANNER_SELF_MATCH_PATHS: ReadonlySet<string> = new Set([
  'package/cli/forbidden-strings/data/betterleaks-default-config.toml',
  'package/cli/forbidden-strings/data/builtin-rules.txt',
  'package/cli/forbidden-strings/src/port-betterleaks-relaxations.ts',
  'package/cli/forbidden-strings/src/rule/algebra_tests.rs',
],);
/**
 * Domain sentinel for configured rules path outside candidate repository.
 */
const RULES_PATH_OUTSIDE_REPOSITORY: unique symbol = Symbol('configured forbidden-strings rules path is outside repository',);

/**
 * Resolves configured rules file to repository-relative Git path when it is inside repository.
 *
 * @param repositoryRoot - scanner cwd controlling rules-file precedence
 *
 * @param environment - process environment carrying the rules-path override
 *
 * @returns repository-relative configured rules path or domain sentinel for external path
 *
 * @example
 * ```ts
 * configuredRulesCandidatePath({ repositoryRoot: '/repo', environment: process.env });
 * ```
 */
function configuredRulesCandidatePath({
  repositoryRoot,
  environment,
}: Readonly<{
  repositoryRoot: string;
  environment: ForeignBorrowed<NodeJS.ProcessEnv>;
}>,): string | typeof RULES_PATH_OUTSIDE_REPOSITORY {
  /**
   * Scanner rules source selected through existing environment precedence.
   */
  const configuredPath = environment
    .FORBIDDEN_STRINGS_RULES
    ?? 'forbidden-strings.local.txt';
  /**
   * Lexical path from repository root to selected rules file.
   */
  const candidatePath = relative(
    repositoryRoot,
    resolve(
      repositoryRoot,
      configuredPath,
    ),
  );
  if ((candidatePath.length === 0) || isAbsolute(candidatePath,)
    || (candidatePath === '..')
    || candidatePath.startsWith(`..${sep}`,))
    return RULES_PATH_OUTSIDE_REPOSITORY;
  return candidatePath.split(sep,)
    .join('/',);
}

/**
 * Applies scanner `--all` path semantics before explicit temporary-file scanning.
 *
 * @param repositoryRoot - scanner cwd controlling path-anchored exclusions
 *
 * @param environment - process environment carrying the rules-path override
 *
 * @param candidates - exact policy candidates
 *
 * @returns candidates scanner walker would retain
 *
 * @example
 * ```ts
 * scannerEligibleCandidates({ repositoryRoot: '/repo', environment: process.env, candidates: [] });
 * ```
 */
function scannerEligibleCandidates({
  repositoryRoot,
  environment,
  candidates,
}: Readonly<{
  repositoryRoot: string;
  environment: ForeignBorrowed<NodeJS.ProcessEnv>;
  candidates: readonly CandidateFile[];
}>,): readonly CandidateFile[] {
  /**
   * Configured rules path excluded alongside canonical generated sources.
   */
  const rulesPath = configuredRulesCandidatePath({
    repositoryRoot,
    environment,
  },);
  return candidates.filter(function scannerWouldVisit(candidate,): boolean {
    return (candidate.path !== rulesPath)
      && (!SCANNER_SELF_MATCH_PATHS.has(candidate.path,));
  },);
}

/**
 * Owned cancellation signal and listener cleanup for scanner subprocess.
 */
type ScannerAbortRelay = Readonly<{
  /**
   * Owned signal safe to pass into external subprocess adapter.
   */
  signal: AbortSignal;
  /**
   * Removes forwarding listener from borrowed engine signal.
   */
  [Symbol.dispose]: () => void;
}>;

/**
 * Relays borrowed engine cancellation into owned scanner signal.
 *
 * @param signal - borrowed engine cancellation signal
 *
 * @mutates signal - installs and removes one abort listener during scanner lifetime
 *
 * @returns owned scanner signal with deterministic listener cleanup
 *
 * @example
 * ```ts
 * using relay = createScannerAbortRelay(new AbortController().signal);
 * ```
 */
function createScannerAbortRelay(
  signal: ForeignBorrowed<AbortSignal>,
): ScannerAbortRelay {
  /**
   * Scanner-owned cancellation controller.
   */
  const controller = new AbortController();
  /**
   * One-way cancellation callback retaining no borrowed abort reason.
   */
  const forwardAbort = (): void => {
    controller.abort();
  };
  if (signal.aborted)
    controller.abort();
  else
    signal.addEventListener(
      'abort',
      forwardAbort,
      { once: true, },
    );
  return {
    signal: controller.signal,
    [Symbol.dispose](): void {
      signal.removeEventListener(
        'abort',
        forwardAbort,
      );
    },
  };
}

/**
 * Runs external scanner over exact candidate bytes.
 *
 * @param executable - PATH-resolved command or configured executable path
 *
 * @param builtinRules - whether scans pass the scanner's opt-in `--builtin-rules` flag
 *
 * @param repositoryRoot - scanner cwd controlling rules-file precedence
 *
 * @param environment - process environment carrying the rules-path override; injectable so
 *   tests stay hermetic under a parallel runner without mutating shared global state
 *
 * @param candidates - exact Git candidates
 *
 * @param signal - engine cancellation signal
 *
 * @mutates signal - abort relay installs and removes one listener while scanner runs
 *
 * @returns redacted policy findings
 *
 * @example
 * ```ts
 * await scanCandidates({ executable: 'forbidden-strings', builtinRules: false, repositoryRoot: '/repo', candidates: [], signal: new AbortController().signal });
 * ```
 */
export async function scanCandidates({
  executable,
  builtinRules,
  repositoryRoot,
  environment = process.env,
  candidates,
  signal,
}: Readonly<{
  executable: string;
  builtinRules: boolean;
  repositoryRoot: string;
  environment?: NodeJS.ProcessEnv;
  candidates: readonly CandidateFile[];
  signal: ForeignBorrowed<AbortSignal>;
}>): Promise<readonly PolicyFinding[]> {
  /**
   * Scanner-owned cancellation signal detached from borrowed engine signal.
   */
  using abortRelay = createScannerAbortRelay(signal,);
  /**
   * Disposable exact scanner inputs.
   */
  await using materialized = await materializeCandidates(scannerEligibleCandidates({
    repositoryRoot,
    environment,
    candidates,
  },),);
  if (materialized.paths
    .length
    === 0)
    return [];
  /**
   * Scanner argv: the opt-in embedded-baseline flag (when configured) before
   * explicit temporary-file positionals.
   */
  const scannerArguments = builtinRules
    ? [
      '--builtin-rules',
      ...materialized.paths,
    ]
    : materialized.paths;
  try {
    await nanoSpawn(
      executable,
      [...scannerArguments,],
      {
        cwd: repositoryRoot,
        signal: abortRelay.signal,
      },
    );
    return [];
  }
  catch (error: unknown) {
    if (!(error instanceof SubprocessError))
      throw error;
    if (error.exitCode === 1)
      return parseScannerOutput({
        stderr: error.stderr,
        candidateForPath: function candidateForPath(path,): CandidateFile {
          /**
           * Exact mapped candidate.
           */
          const candidate = materialized.candidatesByPath
            .get(path,);
          if (candidate === undefined)
            throw new ForbiddenStringsPluginError(`Forbidden-strings scanner reported unknown candidate: ${path}`,);
          return candidate;
        },
      },);
    if (error.signalName !== undefined)
      throw new ForbiddenStringsPluginError(
        `Forbidden-strings scanner was interrupted by ${error.signalName}.`,
        { cause: error, },
      );
    if (error.exitCode === undefined)
      throw new ForbiddenStringsPluginError(
        'Forbidden-strings scanner executable could not be started.',
        { cause: error, },
      );
    throw new ForbiddenStringsPluginError(
      `Forbidden-strings scanner exited with infrastructure status ${String(error.exitCode,)}.`,
      { cause: error, },
    );
  }
}
