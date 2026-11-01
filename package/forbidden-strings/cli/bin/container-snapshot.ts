/**
 Scanner-owned source snapshots for offline, mount-free verification.

 The process helper, the copied inputs and the image build live in sibling files; this module re-exports the helper so
 the verification scripts keep one import.
 */
import { createHash } from 'node:crypto';
import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import { run } from './scanner-run.ts';
import { inventory } from './source-inventory.ts';
import {
  baseImage,
  limits,
  repository,
  scanner,
} from './snapshot-constants.ts';
import {
  copyTool,
  copyToolchain,
  vendorDependencies,
} from './snapshot-host-tools.ts';
import {
  buildImage,
  writeContainerfile,
} from './snapshot-image.ts';
import { copySources } from './snapshot-sources.ts';

export {
  run,
  type RunOptions,
  type RunResult,
  ScannerVerificationError,
} from './scanner-run.ts';

/**
 Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
 whose backing device is unknowable.
 */
const scratchBase = join(
  import.meta.dirname,
  '..',
  'node_modules',
  '.monochromatic',
  'forbidden-strings',
);

/**
 What a transform receives: the build context and the scanner's directory inside it.
 */
export type SnapshotContext = {
  readonly context: string;
  readonly scanner: string;
};

/**
 Options of one source snapshot.
 */
export type SnapshotOptions = {
  readonly command: readonly string[];
  readonly name: string;
  readonly toolchain?: boolean;
  readonly tool?: string;
  readonly fuzzing?: boolean;
  readonly transform?: (context: SnapshotContext) => Promise<void>;
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

 @param command - the command the image runs

 @param name - the label of the evidence directory and the image tag

 @param toolchain - true to carry the host's Rust toolchain

 @param tool - an installed Cargo tool to carry, or nothing

 @param fuzzing - true to carry the fuzz sidecar and start in it

 @param transform - edits the copied sources before they are hashed, or nothing

 @returns the build context, the evidence directory, the image and the container limits

 @throws ScannerVerificationError when setup fails; the failure is also written to `setup-failure.txt` in the evidence

 @example
 ```ts
 const fixture = await snapshot({ command: ['cargo', 'test'], name: 'test' });
 ```
 */
export async function snapshot({
  command,
  name,
  toolchain = false,
  tool,
  fuzzing = false,
  transform,
}: SnapshotOptions): Promise<Snapshot> {
  await mkdir(
    scratchBase,
    { recursive: true },
  );
  /**
   The build context.
   */
  const context = await mkdtemp(join(
    scratchBase,
    'scanner-snapshot-',
  ));
  /**
   Directory that holds one evidence directory per snapshot.
   */
  const evidenceRoot = resolve('target/verification');
  await mkdir(
    evidenceRoot,
    { recursive: true },
  );
  /**
   The evidence directory of this snapshot.
   */
  const evidence = await mkdtemp(join(
    evidenceRoot,
    `${name}-`,
  ));
  try {
    await copySources({
      context,
      fuzzing,
    });
    if (transform) await transform({
      context,
      scanner,
    });
    /**
     Digest and path of every copied file.
     */
    const sources = await inventory({ root: context });
    /**
     Digest of the inventory, which names the image.
     */
    const sourceSha256 = createHash('sha256')
      .update(JSON.stringify(sources))
      .digest('hex');
    await vendorDependencies({
      context,
      fuzzing,
    });
    /**
     The copied toolchain, when the snapshot carries one.
     */
    const toolchainCopy = toolchain ? await copyToolchain(context) : undefined;
    /**
     The copied Cargo tool, when the snapshot carries one.
     */
    const toolCopy = tool === undefined ? undefined : await copyTool({
      context,
      tool,
    });
    /**
     Versions and digests of the host tools the image carries.
     */
    const tools: Readonly<Record<string, string>> = {
      ...(toolchainCopy === undefined ? {} : { rustc: toolchainCopy.record }),
      ...((toolCopy === undefined) || (tool === undefined) ? {} : { [tool]: toolCopy.record }),
    };
    /**
     Containerfile lines that install the host tools.
     */
    const additional: readonly string[] = [
      ...(toolchainCopy?.lines ?? []),
      ...(toolCopy?.lines ?? []),
    ];
    await writeContainerfile({
      context,
      additional,
      command,
      fuzzing,
    });
    /**
     The built image and the Git it carries.
     */
    const built = await buildImage({
      context,
      name,
      sourceSha256,
    });
    /**
     The commit of the repository the sources came from.
     */
    const head = (await run({
      command: 'git',
      args: [
        '-C',
        repository,
        'rev-parse',
        'HEAD',
      ],
      capture: true,
    })).stdout
      .trim();
    await writeFile(
      join(
        evidence,
        'manifest.json',
      ),
      `${JSON.stringify(
        {
          base: baseImage,
          image: built.image,
          head,
          sourceSha256,
          sources,
          tools,
          version: built.version,
          limits,
          user: '1000:1000',
          command,
        },
        null,
        2,
      )}\n`,
    );
    console.log(`Scanner ${name} evidence: ${evidence}`);
    return {
      context,
      evidence,
      image: built.image,
      limits,
    };
  } catch (error) {
    await writeFile(
      join(
        evidence,
        'setup-failure.txt',
      ),
      `${String(error)}\n`,
    );
    await rm(
      context,
      {
        recursive: true,
        force: true,
      },
    );
    throw error;
  }
}
