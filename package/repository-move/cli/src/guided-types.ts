/**
 Types of the guided runner: the configuration of one move, the stages of each command, and the engine's context.

 @module
 */

/**
 Every path and bound of one move. Paths come from `doc/planning/repository-on-the-860-drive.md`; each can be
 overridden so a fixture run never touches the real disks. A bound marked author's choice is a number this runner's
 author chose so that nothing waits forever.
 */
export type GuidedConfig = {
  readonly homeDir: string;
  readonly userName: string;
  readonly groupName: string;
  readonly repo: string;
  readonly externalWorktrees: string;
  readonly drive: string;
  readonly destRoot: string;
  readonly received: string;
  readonly finalPath: string;
  readonly externalFinal: string;
  readonly snapshots: string;
  readonly moveState: string;
  readonly wiz: string;
  readonly toolsFolder: string;
  readonly copyTool: string;
  readonly scanTool: string;
  readonly measureTool: string;
  readonly inventory: string;
  readonly inventoryMain: string;
  readonly fixtureParent: string;
  readonly cliGitState: string;
  readonly nodePath: string;
  readonly linkBinFolder: string;
  readonly unitPath: string;
  readonly transferDeadlineSeconds: number;
  readonly firstPassTotalSeconds: number;
  readonly externalDeadlineSeconds: number;
  readonly minFreeGiB: number;
  readonly nvmeMinFreeGiB: number;
  readonly marginPercent: number;
  readonly sharingLimitPercent: number;
  readonly scanPerWorktreeSeconds: number;
  readonly fixtureStepTimeoutSeconds: number;
  readonly measureTimeoutSeconds: number;
  readonly sizeTableTimeoutSeconds: number;
  readonly scanTimeoutSeconds: number;
  readonly killGraceSeconds: number;
  readonly windowMaxOverrideSeconds: number;
  readonly rootScope: string;
  readonly userScope: string;
};

/**
 Whether a stage runs a privileged command: `yes` for any, `no` for none.
 */
export type RootUse = 'no' | 'yes';

/**
 Output an internal stage step produces, and what it may read.
 */
export type InternalContext = {
  readonly config: GuidedConfig;
};

/**
 One stage: what it says, the command it runs, how it is checked, and how it is undone.
 */
export type Stage = {
  readonly id: string;
  readonly title: string;
  readonly says: string;

  /**
   Shell text run with `bash --noprofile --norc -e -o pipefail -c`, or a function that builds it when the stage runs (for text that
   depends on files an earlier stage wrote). `@DEADLINE@` becomes the stage's per-step deadline in seconds, the smaller
   of the transfer deadline and the window's remainder; `@REMAINING@` becomes the window's remainder alone.
   */
  readonly command: string | ((context: InternalContext,) => Promise<string>);

  /**
   Shell text that must exit 0 after the command; `$OUTPUT_FILE` names the command's output.
   */
  readonly check?: string;
  readonly checkText?: string;
  readonly rollback?: string;

  /**
   Exit statuses of the command that count as success; default `[0]`.
   */
  readonly okCodes?: readonly number[];
  readonly timeoutSeconds?: number;

  /**
   The stage runs inside the cut-over window and is refused once the window's maximum has passed.
   */
  readonly window?: boolean;

  /**
   The stage starts the window clock.
   */
  readonly windowStart?: boolean;

  /**
   The caller must pass `--assert=<id>` to run the stage, because it states a fact only a person or the coordinator can
   know; the text says which fact.
   */
  readonly assertion?: string;
  readonly timer?: {
    readonly file: string;
    readonly mode: 'add' | 'set';
  };

  /**
   The stage is repeated on every run and never recorded as done.
   */
  readonly noMark?: boolean;

  /**
   Whether any command of the stage uses `sudo`.
   */
  readonly root: RootUse;

  /**
   The unprivileged route that exists for the root work, or why none does.
   */
  readonly unprivileged: string;

  /**
   An internal step done in TypeScript before the command; its text is written to the output file the check reads.
   */
  readonly internal?: (context: InternalContext,) => Promise<string>;
};

/**
 The commands of the runner.
 */
export type GuidedCommand = 'cutover' | 'prepare' | 'rollback' | 'status' | 'thaw';

/**
 The flags of one invocation.
 */
export type GuidedFlags = {
  readonly command: GuidedCommand;
  readonly dryRun: boolean;
  readonly redo: readonly string[];
  readonly asserted: readonly string[];
};
