/**
 The Rust toolchain the gate's Clippy step mounts, taken from the repository's lock rather than from the caller's
 environment.

 Why: the gate used to mount whichever sysroot `rustc --print sysroot` named in the environment it was started from.
 With `RUSTUP_TOOLCHAIN` unset, rustup answered the default `nightly`, whose sysroot holds no Clippy component, so
 `cargo clippy` found the image's own `cargo-clippy` later on `PATH` and that Clippy could not read the crate metadata
 the newer rustc wrote (`error[E0514]`). Taking the pinned toolchain, and refusing when its `bin` directory holds no
 Clippy, keeps one compiler for the whole Clippy step
 (`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`).

 In TS you'd write (pseudocode):
 ```ts
 const toolchain = await readLockedRustToolchain(repository);
 const sysroot = dirname(dirname(await rustupWhich(toolchain, 'rustc')));
 requireClippyIn(sysroot);
 ```
 */
import { stat, } from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import {
  LockedRustToolchainError,
  readLockedRustToolchain,
} from '@monochromatic-dev/test-harness-core-matrix/ts/locked-rust-toolchain.ts';

import { isMissingPathError, } from './native-gate-paths.ts';

import {
  type CommandOutcome,
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';

/**
 Programs the mounted sysroot must hold beside its `rustc` for the Clippy step to run from it.

 `cargo` is started by its path in the mounted sysroot, `cargo-clippy` is the program `cargo clippy` finds on `PATH`,
 `clippy-driver` is the compiler wrapper that program sets, and `rustc` and `rustdoc` are the compiler and the
 documentation path the Clippy container is given.
 */
export const clippyToolchainPrograms = [
  'cargo',
  'cargo-clippy',
  'clippy-driver',
  'rustc',
  'rustdoc',
] as const;

/**
 The permission bits any of those programs needs: `PATH` runs a program only when one execute bit is set.
 */
const executablePermissionBits = 0o111;

/**
 The two programs a toolchain's `clippy` component installs, so a refusal can name the component that is missing.
 */
const clippyComponentPrograms = [
  'cargo-clippy',
  'clippy-driver',
] as const;

/**
 The toolchain the Clippy step runs, and the directory the container mounts for it.
 */
export type ClippyToolchain = {
  /**
   The dated toolchain name `toolchain.lock.json` records, such as `nightly-2026-11-16`.
   */
  readonly toolchain: string;
  /**
   The sysroot of that installed toolchain, which holds `cargo`, `rustc`, `rustdoc` and the Clippy programs.
   */
  readonly sysroot: string;
};

/**
 One program of a sysroot, and whether it is there.
 */
type ProgramPresence = {
  /**
   The program's name, as {@link clippyToolchainPrograms} spells it.
   */
  readonly program: string;
  /**
   Whether the sysroot's `bin` directory holds it.
   */
  readonly present: boolean;
};

/**
 Whether one program is in a sysroot's `bin` directory as a file that can be run.

 Why the file type and the execute bits and not only that the name resolves: a refusal that a directory or a
 non-executable file satisfies would let `cargo clippy` fall through to another `cargo-clippy` on `PATH`, which is
 the failure this refuses.

 @param sysroot - directory of an installed toolchain

 @param program - program name inside its `bin` directory

 @returns true when the name reaches a regular file with an execute bit set

 @throws Whatever the file system answers with for a path that is not a missing file, so a refusal is never mistaken
 for a missing program
 */
async function isProgramIn({
  sysroot,
  program,
}: {
  readonly sysroot: string;
  readonly program: string;
},): Promise<boolean> {
  try {
    /**
     What the `bin` directory holds under that name.
     */
    const metadata = await stat(join(
      sysroot,
      'bin',
      program,
    ),);
    return metadata.isFile() && ((metadata.mode & executablePermissionBits) !== 0);
  }
  catch (error) {
    if (isMissingPathError({ error, }))
      return false;
    throw error;
  }
}

/**
 Programs `sysroot`'s `bin` directory lacks.

 @param sysroot - directory of an installed toolchain, whose programs live in its `bin` directory

 @returns the missing program names in {@link clippyToolchainPrograms} order, empty when the toolchain is whole

 @example
 ```ts
 await missingClippyToolchainPrograms({ sysroot: '/home/a/.rustup/toolchains/nightly-2026-11-16-x86_64-unknown-linux-gnu' });
 ```
 */
export async function missingClippyToolchainPrograms({ sysroot }: { readonly sysroot: string; }): Promise<readonly string[]> {
  /**
   Every program with the answer for this sysroot.
   */
  const presence: readonly ProgramPresence[] = await Promise.all(clippyToolchainPrograms.map(async function checked(program) {
    return {
      program,
      present: await isProgramIn({
        sysroot,
        program,
      },),
    };
  },),);
  return presence.filter(function absent(entry) {
    return !entry.present;
  },)
    .map(function named(entry) {
      return entry.program;
    },);
}

/**
 What asking rustup for a toolchain's `rustc` answered.
 */
type RustupAnswer =
  | {
    readonly kind: 'answered';
    readonly status: number;
    readonly rustc: string;
    readonly stderr: string;
  }
    | {
      readonly kind: 'couldNotAsk';
      readonly reason: string;
    };

/**
 Ask rustup which `rustc` a toolchain holds, without throwing: a refusal quotes what rustup said.

 @param toolchain - the dated toolchain name the lock records

 @param environment - environment `rustup` runs in, which decides where rustup keeps its toolchains

 @returns the answer's own fields, or why rustup could not be asked
 */
async function rustupWhich({
  toolchain,
  environment,
}: {
  readonly toolchain: string;
  readonly environment: NodeJS.ProcessEnv;
},): Promise<RustupAnswer> {
  try {
    /**
     What the command answered, flattened below so no refusal walks a chain of fields.
     */
    const outcome = await runCommand({
      command: 'rustup',
      args: [
        'which',
        '--toolchain',
        toolchain,
        'rustc',
      ],
      capture: true,
      allowFailure: true,
      env: environment,
    },);
    return {
      kind: 'answered',
      status: outcome.status ?? 1,
      rustc: outcome.stdout
        .trim(),
      stderr: outcome.stderr
        .trim(),
    };
  }
  catch (error) {
    return {
      kind: 'couldNotAsk',
      reason: String(error,),
    };
  }
}

/**
 The sysroot of one installed rustup toolchain, from `rustup which --toolchain <name> rustc`.

 Why `rustup which` and not `rustc --print sysroot`: `rustup which` answers about the named toolchain whatever the
 caller's `RUSTUP_TOOLCHAIN` or rustup default says, and it prints the toolchain's own `rustc` rather than any proxy.

 @param toolchain - the dated toolchain name the lock records

 @param environment - environment `rustup` runs in, which decides where rustup keeps its toolchains

 @returns the sysroot directory, which is the parent of the `bin` directory holding that `rustc`

 @throws NativeVerificationError when `rustup` cannot run, has no such toolchain, or names no path

 @example
 ```ts
 await rustupToolchainSysroot({ toolchain: 'nightly-2026-11-16' });
 ```
 */
export async function rustupToolchainSysroot({
  toolchain,
  environment = process.env,
}: {
  readonly toolchain: string;
  readonly environment?: NodeJS.ProcessEnv;
},): Promise<string> {
  /**
   What `rustup which` answered, which a refusal quotes.
   */
  const answer = await rustupWhich({
    toolchain,
    environment,
  },);
  if (answer.kind === 'couldNotAsk')
    throw new NativeVerificationError(`The gate's Clippy step asks the repository's locked Rust toolchain for its sysroot with \`rustup which --toolchain ${toolchain} rustc\`, which could not run: ${answer.reason}. Install rustup and the lock's tools with \`mise run toolchain:install\`, then run the gate again.`,);
  if (answer.status !== 0)
    throw new NativeVerificationError(`The gate's Clippy step needs the repository's locked Rust toolchain ${toolchain}, and rustup could not name a \`rustc\` for it (\`rustup which --toolchain ${toolchain} rustc\` exited ${String(answer.status)}): ${answer.stderr}. Install the toolchain with \`mise run toolchain:install\`, which installs every tool and component toolchain.lock.json records, then run the gate again.`,);
  if (answer.rustc === '')
    throw new NativeVerificationError(`The gate's Clippy step needs the sysroot of the repository's locked Rust toolchain ${toolchain}, and \`rustup which --toolchain ${toolchain} rustc\` printed no path. Install the toolchain with \`mise run toolchain:install\`, then run the gate again.`,);
  // The sysroot is the directory above `bin`, which holds the `rustc` rustup printed.
  return dirname(dirname(answer.rustc,),);
}

/**
 The toolchain the repository's nearest lock records.

 @param startDirectory - directory the search for `toolchain.lock.json` starts in, usually the package being gated

 @returns the dated toolchain name

 @throws NativeVerificationError when no lock is above the directory or the lock carries no Rust toolchain
 */
async function lockedToolchain({ startDirectory }: { readonly startDirectory: string; }): Promise<string> {
  try {
    return await readLockedRustToolchain(startDirectory,);
  }
  catch (error) {
    if (!(error instanceof LockedRustToolchainError))
      throw error;
    throw new NativeVerificationError(`The gate's Clippy step takes the Rust toolchain from the repository's toolchain.lock.json rather than from the environment it was started in, and no lock was usable: ${error.message}. Run the gate from inside the repository that holds the lock.`,);
  }
}

/**
 The repository's locked Rust toolchain, its sysroot, and the refusal when that sysroot holds no Clippy.

 @param startDirectory - directory the search for `toolchain.lock.json` starts in, usually the package being gated

 @param environment - environment `rustup` runs in, which decides where rustup keeps its toolchains

 @returns the locked toolchain and the sysroot the Clippy container mounts read-only

 @throws NativeVerificationError when no lock is above the directory, when rustup has no such toolchain, or when the
 sysroot holds no Clippy component, each naming what is missing and how to install it

 @example
 ```ts
 const { toolchain, sysroot } = await gateClippyToolchain({ startDirectory: packageDirectory });
 ```
 */
export async function gateClippyToolchain({
  startDirectory,
  environment = process.env,
}: {
  readonly startDirectory: string;
  readonly environment?: NodeJS.ProcessEnv;
},): Promise<ClippyToolchain> {
  /**
   The dated toolchain name the nearest lock records.
   */
  const toolchain = await lockedToolchain({ startDirectory, },);
  /**
   The sysroot of that toolchain as rustup installed it.
   */
  const sysroot = await rustupToolchainSysroot({
    toolchain,
    environment,
  },);
  /**
   The programs that sysroot's `bin` directory lacks, in the order the gate needs them.
   */
  const missing = await missingClippyToolchainPrograms({ sysroot, },);
  if (missing.length > 0) {
    /**
     Whether one missing program is one the toolchain's Clippy component installs.
     */
    const clippyMissing = missing.some(function isClippy(program) {
      return clippyComponentPrograms.some(function sameComponent(component) {
        return component === program;
      },);
    });
    /**
     What a person does about it, which names the Clippy component only when that component is what is missing.
     */
    const remedy = clippyMissing
      ? `Install the Clippy component with \`rustup component add clippy --toolchain ${toolchain}\`, or every tool and component toolchain.lock.json records with \`mise run toolchain:install\`, then run the gate again.`
      : `Install the toolchain with \`mise run toolchain:install\`, which installs every tool and component toolchain.lock.json records, then run the gate again.`;
    throw new NativeVerificationError(
      `The gate's Clippy step mounts the repository's locked Rust toolchain ${toolchain} at ${sysroot}, but that sysroot's bin directory holds no runnable ${missing.join(', ')}${clippyMissing ? ': the toolchain\'s Clippy component is not installed' : ''}. ${remedy}`,
    );
  }
  return {
    toolchain,
    sysroot,
  };
}
