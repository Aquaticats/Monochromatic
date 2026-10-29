/**
 Snapshot inputs the native wrapper gained by linking the repository's scanner:
 the scanner crate, its matching engine, and the locked registry dependencies both pull in.

 The container has no network, so registry dependencies are vendored on the host from the Cargo cache
 and baked into the image beside the copied path crates.
 */
import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';

/**
 Image path of the vendored registry sources; the Containerfile copies `vendor` there.
 */
const imageVendorDirectory = '/work/vendor';

/**
 One file or directory to copy from the repository into the build context.
 */
export type SnapshotEntry = {
  from: string;
  to: string
};

/**
 Copy entries for one path crate, resolved from the wrapper package directory the gate runs in.

 @param request -
   build context, the crate's directory under `package`, and the entries a library build of it reads
 
 @returns one entry per name, keeping the repository layout so relative path dependencies resolve
 */
function crateEntries({
  context,
  crate,
  names,
}: {
  context: string;
  crate: string;
  names: readonly string[]
}): SnapshotEntry[] {
  return names.map(function crateEntry(name) {
    return {
      from: resolve(
        '../..',
        crate,
        name,
      ),
      to: join(
        context,
        'package',
        crate,
        name,
      ),
    };
  });
}

/**
 Copy entries for the linked scanner and its matching engine.

 @param request - build context directory receiving the repository-shaped snapshot
 
 @returns entries for both path crates;
   the scanner's `build.rs` and `data` hold the embedded baseline its library compiles in
 */
export function scannerSnapshotEntries({ context }: { context: string }): SnapshotEntry[] {
  return [
    ...crateEntries({
      context,
      crate: 'forbidden-strings/cli',
      names: [
        'Cargo.toml',
        'Cargo.lock',
        'build.rs',
        'src',
        'data',
      ],
    }),
    ...crateEntries({
      context,
      crate: 'forbidden-strings/core',
      names: [
        'Cargo.toml',
        'Cargo.lock',
        'src',
      ],
    }),
  ];
}

/**
 Vendor the copied wrapper's locked registry dependencies from the host's Cargo cache into the build context,
 and write the Cargo source mapping the image reads them through.
 Every archive the lockfile names must already be in that cache; Cargo fails the gate otherwise.
 Another crate of the snapshot, such as the linter the Markdown policy runs, passes its own manifest and output
 names, so each image stage reads only the sources its own lockfile names.

 @param request -
   build context holding the copied crates, the manifest under it whose lockfile is vendored
   (the wrapper's by default), and the directory names receiving the sources and the mapping
   (`vendor` and `cargo-config` by default)
 */
export async function vendorLockedDependencies({
  context,
  manifest = 'package/git-policy/cli/Cargo.toml',
  vendorName = 'vendor',
  configName = 'cargo-config',
}: {
  context: string;
  manifest?: string;
  vendorName?: string;
  configName?: string
}): Promise<void> {
  const vendor = join(
    context,
    vendorName,
  );
  const vendored = await runCommand({
    command: 'cargo',
    args: [
      'vendor',
      '--manifest-path',
      join(
        context,
        manifest,
      ),
      '--offline',
      '--locked',
      '--versioned-dirs',
      vendor,
    ],
    capture: true,
  });
  // Cargo prints the mapping for the host path; the image holds the same directory elsewhere.
  const mapping = vendored.stdout
    .replaceAll(
    JSON.stringify(vendor),
    JSON.stringify(imageVendorDirectory),
    );
  if (!mapping.includes(`directory = ${JSON.stringify(imageVendorDirectory)}`))
    throw new NativeVerificationError('Cargo vendor did not provide the expected offline source mapping.');
  const configuration = join(
    context,
    configName,
  );
  await mkdir(
    configuration,
    { recursive: true },
  );
  await writeFile(
    join(
      configuration,
      'config.toml',
    ),
    mapping,
  );
}
