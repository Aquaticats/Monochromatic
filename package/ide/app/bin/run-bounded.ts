// What: the one bounded podman argv every IDE task runs, built from the container bound, the read-only root clippy
// config, the work directory, and the image.
// Why: the argv used to be copied into every task body, so a mount, env, or limit change had to be made in thirty-six
// places, and a missed copy only showed up when the task ran. The gpu arguments and the clippy config path live here for
// the same reason. The pure argv builder is tested in run-bounded.test.ts with node:test.
import { execFileSync, type ExecFileSyncOptions } from 'node:child_process';
import { gpuArguments } from './gpu-arguments.ts';
import type { ContainerBounds } from './container-bounds.ts';

/**
 Limits and environment a run needs from the container bound.
 */
export type BoundedContainerLimits = Pick<ContainerBounds, 'limitArguments' | 'environmentArguments'>;

/**
 Everything one bounded argv is built from.

 @property image - image the task runs in, already selected for the release or default build
 @property containerLimits - limit and environment arguments from the container bound
 @property rootClippyConfig - host path of the repository-root clippy.toml, mounted read-only
 @property workDirectory - host directory mounted at /work
 @property network - whether the run reaches the network, off unless a task needs it
 @property gpu - whether the run takes the host GPU's device arguments
 @property command - command and arguments inside the container
 */
export type BoundedRun = {
  readonly image: string;
  readonly containerLimits: BoundedContainerLimits;
  readonly rootClippyConfig: string;
  readonly workDirectory: string;
  readonly network?: boolean;
  readonly gpu?: boolean;
  readonly command?: readonly string[];
};

/**
 Build the podman argv of one bounded container run.

 @param switches - image, bound, clippy config, work directory, switches, and command

 @returns podman arguments, ready for `execFileSync('podman', arguments)`
 */
export function boundedArguments({
  image,
  containerLimits,
  rootClippyConfig,
  workDirectory,
  network = false,
  gpu = false,
  command = [],
}: BoundedRun,): string[] {
  return [
    'run',
    '--rm',
    ...(network ? [] : ['--network=none',]),
    ...(gpu ? gpuArguments : []),
    ...containerLimits.limitArguments,
    '--security-opt',
    'label=disable',
    '--volume',
    `${workDirectory}:/work`,
    '--volume',
    'ide-cargo:/cargo',
    // The repository-root clippy.toml holds the fleet-wide disallowed-methods ban that [lints.clippy] only activates.
    // It is mounted read-only outside /work and named by CLIPPY_CONF_DIR: a mount over /work/clippy.toml would leave an
    // empty clippy.toml in the package directory on the host. Host clippy finds the file in an ancestor directory.
    '--volume',
    `${rootClippyConfig}:/clippy-conf/clippy.toml:ro`,
    '--env',
    'CLIPPY_CONF_DIR=/clippy-conf',
    '--workdir',
    '/work',
    ...containerLimits.environmentArguments,
    '--env',
    'SLINT_EMIT_DEBUG_INFO=1',
    '--env',
    'HELIX_RUNTIME=/work/target/debug/runtime',
    image,
    ...command,
  ];
}

/**
 Runs one podman argv and returns nothing; the injected form lets a test read the argv instead of running a container.
 */
export type BoundedExecutor = (
  command: string,
  args: readonly string[],
  options: ExecFileSyncOptions,
) => void;

/**
 Build the run helpers one task body uses.

 @param options - image, bound, clippy config, work directory, and an optional executor for tests

 @returns `runBounded`, which runs one podman argv, and `runCargo`, which runs cargo inside it
 */
export function createRunBounded({
  image,
  containerLimits,
  rootClippyConfig,
  workDirectory = process.cwd(),
  execute = function executeSync(command, args, options,): void {
    execFileSync(command, args, options,);
  },
}: {
  readonly image: string;
  readonly containerLimits: BoundedContainerLimits;
  readonly rootClippyConfig: string;
  readonly workDirectory?: string;
  readonly execute?: BoundedExecutor;
},): {
  readonly runBounded: (
    args: readonly string[],
    switches?: { readonly network?: boolean; readonly gpu?: boolean; } & ExecFileSyncOptions,
  ) => void;
  readonly runCargo: (
    args: readonly string[],
    switches?: { readonly network?: boolean; readonly gpu?: boolean; } & ExecFileSyncOptions,
  ) => void;
} {
  /**
   Run one command inside a bounded container.

   @param args - command and arguments inside the container

   @param switches - network and gpu switches plus the executor options the caller passes through
   */
  function runBounded(
    args: readonly string[],
    {
      network = false,
      gpu = false,
      ...options
    }: { readonly network?: boolean; readonly gpu?: boolean; } & ExecFileSyncOptions = {},
  ): void {
    execute(
      'podman',
      boundedArguments({
        image,
        containerLimits,
        rootClippyConfig,
        workDirectory,
        network,
        gpu,
        command: args,
      },),
      {
        stdio: 'inherit',
        ...options,
      },
    );
  }

  /**
   Run cargo inside a bounded container.

   @param args - cargo arguments

   @param switches - network and gpu switches plus the executor options the caller passes through
   */
  function runCargo(
    args: readonly string[],
    switches: { readonly network?: boolean; readonly gpu?: boolean; } & ExecFileSyncOptions = {},
  ): void {
    runBounded(
      ['cargo', ...args,],
      switches,
    );
  }

  return {
    runBounded,
    runCargo,
  };
}
