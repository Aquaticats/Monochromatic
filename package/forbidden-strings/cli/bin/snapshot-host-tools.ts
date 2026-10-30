/**
 Host-built inputs of a snapshot: vendored dependencies, the Rust toolchain and installed Cargo tools.
 */
import { createHash } from 'node:crypto';
import {
  cp,
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import { mapInSequence } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import {
  run,
  ScannerVerificationError,
} from './scanner-run.ts';
import { scanner } from './snapshot-constants.ts';

/**
 A copied host tool: its digest or version, and the Containerfile lines that install it.
 */
export type CopiedTool = {
  readonly record: string;
  readonly lines: readonly string[];
};

/**
 Vendor the locked dependencies offline and write the Cargo configuration that maps them into the image.

 @param context - the build context

 @param fuzzing - true to vendor the fuzz sidecar's dependencies too

 @throws ScannerVerificationError when Cargo does not print the expected source mapping

 @example
 ```ts
 await vendorDependencies({ context, fuzzing: false });
 ```
 */
export async function vendorDependencies({
  context,
  fuzzing,
}: {
  readonly context: string;
  readonly fuzzing: boolean;
}): Promise<void> {
  /**
   Where Cargo places the vendored sources inside the context.
   */
  const vendor = join(
    context,
    'vendor',
  );
  /**
   The Cargo configuration Cargo printed for the vendored sources.
   */
  const configuration = (await run({
    command: 'cargo',
    args: [
      'vendor',
      '--manifest-path',
      join(
        context,
        scanner,
        'Cargo.toml',
      ),
      '--offline',
      '--locked',
      '--versioned-dirs',
      ...(fuzzing ? [
        '--sync',
        join(
          context,
          `${scanner}.fuzz`,
          'Cargo.toml',
        ),
      ] : []),
      vendor,
    ],
    capture: true,
  })).stdout;
  /**
   The configuration with the vendor path as the image sees it.
   */
  const mapped = configuration.replaceAll(
    JSON.stringify(vendor),
    '"/work/vendor"',
  );
  if (!mapped.includes('directory = "/work/vendor"'))
    throw new ScannerVerificationError('Cargo vendor did not provide the expected offline source mapping.');
  await mkdir(
    join(
      context,
      'cargo-config',
    ),
    { recursive: true },
  );
  await writeFile(
    join(
      context,
      'cargo-config/config.toml',
    ),
    mapped,
  );
}

/**
 Copy one directory of the host toolchain into the context.

 @param context - the build context

 @param root - the host compiler's sysroot

 @param directory - the sysroot subdirectory to copy

 @example
 ```ts
 await copyToolchainDirectory({ context, root, directory: 'bin' });
 ```
 */
async function copyToolchainDirectory({
  context,
  root,
  directory,
}: {
  readonly context: string;
  readonly root: string;
  readonly directory: string;
}): Promise<void> {
  await cp(
    join(
      root,
      directory,
    ),
    join(
      context,
      'toolchain',
      directory,
    ),
    { recursive: true },
  );
}

/**
 Copy the host's Rust toolchain into the context.

 @param context - the build context

 @returns the compiler version text, and the Containerfile lines that install the toolchain

 @example
 ```ts
 const { record, lines } = await copyToolchain(context);
 ```
 */
export async function copyToolchain(context: string): Promise<CopiedTool> {
  /**
   The host compiler's sysroot.
   */
  const root = (await run({
    command: 'rustc',
    args: [
      '--print',
      'sysroot',
    ],
    capture: true,
  })).stdout
    .trim();
  await mapInSequence({
    items: [
      'bin',
      'lib',
    ],
    run: function copyOf(directory) {
      return copyToolchainDirectory({
        context,
        root,
        directory,
      });
    },
  });
  return {
    record: (await run({
      command: 'rustc',
      args: [
        '--version',
        '--verbose',
      ],
      capture: true,
    })).stdout
      .trim(),
    lines: [
      'COPY toolchain /toolchain',
      'ENV PATH=/toolchain/bin:/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin RUSTC=/toolchain/bin/rustc RUSTDOC=/toolchain/bin/rustdoc',
    ],
  };
}

/**
 Copy one installed Cargo tool into the context.

 @param context - the build context

 @param tool - executable name in the Cargo home's `bin`

 @returns the SHA-256 of the executable, and the Containerfile line that installs it

 @example
 ```ts
 const { record, lines } = await copyTool({ context, tool: 'cargo-mutants' });
 ```
 */
export async function copyTool({
  context,
  tool,
}: {
  readonly context: string;
  readonly tool: string;
}): Promise<CopiedTool> {
  /**
   The installed executable.
   */
  const executable = join(
    process.env
      .CARGO_HOME
      ?? join(
        homedir(),
        '.cargo',
      ),
    'bin',
    tool,
  );
  /**
   The bytes the image will hold.
   */
  const bytes = await readFile(executable);
  await cp(
    executable,
    join(
      context,
      tool,
    ),
  );
  return {
    record: createHash('sha256')
      .update(bytes)
      .digest('hex'),
    lines: [`COPY ${tool} /usr/local/cargo/bin/${tool}`],
  };
}
