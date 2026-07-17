import { SubprocessError, } from 'nano-spawn';
import { caughtValueText, } from '@monochromatic-dev/module-caught-value/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { autoPush, } from './auto-push.ts';
import { parseGlobalOptions, } from './parse-global-options.ts';
import { runManagementCommand, } from './management.ts';
import { parseCommitRegion, } from './parser/commit.ts';
import { printPostCommandOutput, } from './post-command-output.ts';
import { COMMIT_TRANSACTION_NOT_APPLICABLE, } from './policy-engine/commit-transaction.ts';
import { runCommitTransactionBoundary, } from './policy-engine/commit-transaction-boundary.ts';
import {
  CommitTransactionRecoveryError,
  recoverCommitTransaction,
} from './policy-engine/commit-transaction-recovery.ts';
import { runPreForwardPolicyEngine, } from './policy-engine/pre-forward-engine.ts';
import { runPostCommitLifecycle, } from './policy-engine/post-commit-lifecycle.ts';
import {
  MANUAL_PUSH_NOT_APPLICABLE,
  runManualPushGate,
} from './policy-engine/manual-push-gate.ts';
import {
  createEngineFailureEvent,
  renderPolicyEvents,
} from './policy-engine/events.ts';
import { resolveGit, } from './resolve-git.ts';
import { TrustedConfigError, } from './trust/config-loader.ts';
import {
  resolveRuntimeConfig,
  RUNTIME_CONFIG_ABSENT,
} from './trust/runtime-config.ts';
import { ForwardedGitWorktreeCopyError, } from './worktree-copy/errors.ts';
import { runGitWithWorktreeCopy, } from './worktree-copy/lifecycle.ts';

/**
 * Logger root for cli-git after removing the package log shim.
 *
 * @example
 * ```ts
 * const rl = tagged({ tag: someFunction.name, l, },);
 * ```
 */
const l = tagged({ tag: 'cli-git', },);

//region Rule pipeline: validate and transform args before forwarding to real git

/**
 * Tagged logger for the main entry point.
 */
const rl = tagged({
  tag: 'main',
  l,
},);

/**
 * Pre-subcommand argv tokens that cause git to short-circuit: it prints
 * version/help and ignores the subcommand. When any of these appears, the
 * rules pipeline is skipped entirely so wrapper injections do not slot
 * between the short-circuit flag and a later token, which git would then
 * try to parse as flags of its `version` (or `help`) subcommand and reject
 * with `unknown switch ...`.
 */
const SHORT_CIRCUIT_FLAGS: ReadonlySet<string> = new Set([
  '--version',
  '-v',
  '--help',
  '-h',
],);

//endregion Rule pipeline

//region Execution: resolve real git, apply rules, spawn

/**
 * Wrapper runtime-config resolution result.
 */
type WrapperRuntimeResolution =
  | Readonly<{ loaded: Awaited<ReturnType<typeof resolveRuntimeConfig>>; }>
  | Readonly<{ error: unknown; }>;

/**
 * Expected non-zero policy decision after buffered events were emitted.
 */
class PolicyDecisionError extends Error {
  /**
   * Settled cli-git exit code.
   */
  public readonly exitCode: 1 | 2;

  /**
   * Creates control-flow error that cannot be confused with engine exceptions.
   *
   * @param exitCode - settled policy decision
   */
  public constructor(exitCode: 1 | 2,) {
    super('policy decision blocked real Git',);
    this.name = 'PolicyDecisionError';
    this.exitCode = exitCode;
  }
}

/**
 * Runs one direct cli-git invocation.
 *
 * Package root calls this function only when its single MJS artifact is Node's
 * direct program entry.
 *
 * @example
 * ```ts
 * await runCliGit();
 * ```
 */
export async function runCliGit(): Promise<void> {
  /**
   * Raw arguments passed after the script name.
   */
  const rawArgs: readonly string[] = process.argv
    .slice(2,);

  /**
   * Resolves trusted config from process-owned wrapper arguments while retaining failures.
   *
   * @returns loaded config or captured failure.
   *
   * @example
   * ```ts
   * await resolveCurrentWrapperRuntime();
   * ```
   */
  async function resolveCurrentWrapperRuntime(): Promise<WrapperRuntimeResolution> {
    try {
      return { loaded: await resolveRuntimeConfig({ args: rawArgs, },), };
    }
    catch (error: unknown) {
      return { error, };
    }
  }

  /**
   * Layout used to intercept namespaced management command before Git forwarding.
   */
  const managementLayout = parseGlobalOptions(rawArgs,);
  /**
   * Whether wrapper owns this invocation as a management command.
   */
  const isManagementCommand = rawArgs[managementLayout.subcommandIndex] === 'cli-git';

try {
  if (isManagementCommand) {
    /**
     * Real Git required for startup transaction recovery.
     */
    const gitPath = await resolveGit();
    await recoverCommitTransaction({
      args: rawArgs,
      gitPath,
    },);
    process.exitCode = await runManagementCommand({
      args: rawArgs.slice(managementLayout.subcommandIndex + 1,),
      gitGlobalArgs: rawArgs.slice(
        0,
        managementLayout.subcommandIndex,
      ),
    },);
  }
  else {
  /**
   * Layout of `rawArgs` consulted before the rules run so short-circuit flags can be detected on the user's literal input.
   */
  const { subcommandIndex: rawSubcommandIndex, } = parseGlobalOptions(rawArgs,);
  /**
   * Pre-subcommand region of `rawArgs`; scanned for flags that make git ignore the subcommand entirely.
   */
  const rawPreSubcommand = rawArgs.slice(
    0,
    rawSubcommandIndex,
  );
  /**
   * True when git will short-circuit on a pre-subcommand `--version`/`-v`/`--help`/`-h`; rule injections between the flag and the subcommand would be parsed by the wrong git subcommand and error.
   */
  const willShortCircuit = rawPreSubcommand.some(function isShortCircuitFlag(arg,) {
    return SHORT_CIRCUIT_FLAGS.has(arg,);
  },);

  /**
   * Absolute real Git path needed before repository config executes.
   */
  const gitPath = await resolveGit();
  if (!willShortCircuit)
    await recoverCommitTransaction({
      args: rawArgs,
      gitPath,
    },);
  /**
   * Trusted runtime config resolution after transaction recovery.
   */
  const runtimeResolution: WrapperRuntimeResolution = willShortCircuit
    ? { loaded: RUNTIME_CONFIG_ABSENT, }
    : await resolveCurrentWrapperRuntime();
  if ('error' in runtimeResolution) {
    /**
     * Stable trust failure code.
     */
    const code = runtimeResolution.error instanceof TrustedConfigError
      ? runtimeResolution.error
        .code
      : 'trust-failed';
    process.stderr
      .write(renderPolicyEvents([createEngineFailureEvent({
      sequence: 0,
      code,
      message: caughtValueText(runtimeResolution.error,),
    },),],),);
    throw new PolicyDecisionError(2,);
  }

  rl.debug(`using real git at ${gitPath}`,);
  /**
   * Trusted policy options retained identically across convergence passes.
   */
  const policyOptions = runtimeResolution.loaded === RUNTIME_CONFIG_ABSENT
    ? {}
    : {
      config: { policies: runtimeResolution.loaded
        .validated
        .policySeverities, },
      registeredPolicies: runtimeResolution.loaded
        .validated
        .registeredPolicies,
      policyOptions: runtimeResolution.loaded
        .validated
        .policyOptions,
    };
  /**
   * Supported commit transaction, including final stable policy pass.
   */
  const commitTransaction = willShortCircuit
    ? COMMIT_TRANSACTION_NOT_APPLICABLE
    : await runCommitTransactionBoundary({
      args: rawArgs,
      gitPath,
      policyOptions,
    },);
  /**
   * Stable policy result before real Git forwarding.
   */
  const policyResult = willShortCircuit
    ? undefined
    : ((typeof commitTransaction) !== 'symbol'
      ? commitTransaction.policyResult
      : await runPreForwardPolicyEngine({
        options: {
          args: rawArgs,
          trigger: 'pre-forward',
          ...policyOptions,
        },
        gitPath,
      },));
  if (policyResult !== undefined) {
    /**
     * Stable wrapper JSONL emitted only after pass settles.
     */
    const renderedEvents = renderPolicyEvents(policyResult.events,);
    if (renderedEvents !== '')
      process.stderr
        .write(renderedEvents,);
    if (!policyResult.shouldForward) {
      if (policyResult.exitCode === 0)
        throw new TypeError('Non-forwarding policy result cannot use exit code 0.',);
      throw new PolicyDecisionError(policyResult.exitCode,);
    }
  }

  /**
   * Final arguments after staged policy execution and fixed transforms.
   */
  const processedArgs = policyResult?.args ?? rawArgs;

  if (willShortCircuit)
    rl.debug('pre-subcommand short-circuit flag present, skipping policy pipeline',);

  rl.debug(`final args: [${processedArgs.join(', ',)}]`,);

  /**
   * Layout of `processedArgs`: where the subcommand sits, what precedes it, and
   * the directory git will operate in after `-C` chaining.
   */
  const {
    subcommandIndex,
    effectiveCwd,
  } = parseGlobalOptions(processedArgs,);
  /**
   * Subcommand at the located index; `undefined` when args carry no subcommand (e.g. `git --version`).
   */
  const subcommand = processedArgs[subcommandIndex];
  /**
   * Post-subcommand region used by commit lifecycle classification.
   */
  const postSubcommand = processedArgs.slice(subcommandIndex + 1,);
  if (runtimeResolution.loaded !== RUNTIME_CONFIG_ABSENT) {
    /**
     * Settled manual-push gate or not-applicable sentinel.
     */
    const manualPushResult = await runManualPushGate({
      rawArgs,
      transformedArgs: processedArgs,
      gitPath,
      policySeverities: runtimeResolution.loaded
        .validated
        .policySeverities,
      registeredPolicies: runtimeResolution.loaded
        .validated
        .registeredPolicies,
      policyOptions: runtimeResolution.loaded
        .validated
        .policyOptions,
    },);
    if (manualPushResult !== MANUAL_PUSH_NOT_APPLICABLE) {
      /**
       * Manual-push events use wrapper stderr routing.
       */
      const renderedManualPushEvents = renderPolicyEvents(manualPushResult.events,);
      if (renderedManualPushEvents !== '')
        process.stderr
          .write(renderedManualPushEvents,);
      if (!manualPushResult.shouldForward) {
        if (manualPushResult.exitCode === 0)
          throw new TypeError('Non-forwarding manual-push result cannot use exit code 0.',);
        throw new PolicyDecisionError(manualPushResult.exitCode,);
      }
    }
  }
  /**
   * Whether private-index transaction already executed real Git.
   */
  const transactionCommitted = ((typeof commitTransaction) !== 'symbol')
    && commitTransaction.committed;
  if (!transactionCommitted) {
    await runGitWithWorktreeCopy({
      args: processedArgs,
      gitPath,
    },);
  }

  /**
   * True when a real commit just landed (the spawn above succeeded and this
   * was not a dry-run preview), so the new commit should be backed up.
   * Dry-run detection uses the parsed commit region, which covers
   * `--dry-run` plus the output-format flags git documents as implying it
   * (`--short`, `--porcelain`, `--long`, `-z`/`--null`) in any accepted
   * abbreviation. A failed commit throws out of the spawn above and never
   * reaches here.
   */
  const committed = transactionCommitted
    || ((subcommand === 'commit')
      && (!parseCommitRegion(postSubcommand,)
        .isDryRun));

  if (committed) {
    /**
     * Post-commit policy gate against landed commit ground truth.
     */
    const postCommitResult = await runPostCommitLifecycle({
      rawArgs,
      transformedArgs: processedArgs,
      gitPath,
      cwd: effectiveCwd,
      ...(runtimeResolution.loaded === RUNTIME_CONFIG_ABSENT
        ? {}
        : {
          policySeverities: runtimeResolution.loaded
            .validated
            .policySeverities,
          registeredPolicies: runtimeResolution.loaded
            .validated
            .registeredPolicies,
          policyOptions: runtimeResolution.loaded
            .validated
            .policyOptions,
        }),
    },);
    /**
     * Settled post-commit JSONL including explicit landed state when blocked.
     */
    const renderedPostCommitEvents = renderPolicyEvents(postCommitResult.events,);
    if (renderedPostCommitEvents !== '')
      process.stderr
        .write(renderedPostCommitEvents,);
    if (postCommitResult.blocked)
      throw new PolicyDecisionError(2,);
    await autoPush({
      gitPath,
      cwd: effectiveCwd,
    },);
  }

  printPostCommandOutput({
    rawArgs,
    processedArgs,
  },);
  }
}
catch (error) {
  if (error instanceof CommitTransactionRecoveryError) {
    process.stderr
      .write(renderPolicyEvents([createEngineFailureEvent({
      sequence: 0,
      code: 'content-unavailable',
      message: error.message,
    },),],),);
    process.exitCode = 2;
  }
  else if (error instanceof PolicyDecisionError)
    process.exitCode = error.exitCode;
  else if (error instanceof ForwardedGitWorktreeCopyError) {
    console.error(error.copyFailureMessage,);
    process.exitCode = error.gitFailed
      ? error.gitFailureExitCode
        ?? 1
      : 2;
  }
  else if (error instanceof SubprocessError)
    process.exitCode = error.exitCode
      ?? 1;
  else if (Error.isError(error,)) {
    console.error(error.message,);
    process.exitCode = 1;
  }
  else {
    throw error;
  }
}
}

//endregion Execution
