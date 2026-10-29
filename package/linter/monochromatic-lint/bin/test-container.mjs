// Run the full suite by default, or one explicitly named slice in the same isolated source snapshot.
import {
  mkdir,
  mkdtemp,
  cp,
  rm,
  writeFile
} from 'node:fs/promises';
import {
  join,
  resolve
} from 'node:path';
import { spawnSync } from 'node:child_process';
// The gate image runs the suite with the runner the git-policy/cli gate pins by version and checksum.
import { copyNextest } from '../../../git-policy/cli/bin/native-nextest-snapshot.ts';
import { mkdirSync as makeScratchBase } from 'node:fs';

// Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
// whose backing device is unknowable.
const scratchBase = join(import.meta.dirname, '..', 'node_modules', '.monochromatic', 'monochromatic-lint');
makeScratchBase(scratchBase, { recursive: true });

// Concurrent worktrees set MONOCHROMATIC_LINT_IMAGE_TAG so one snapshot never runs another's image.
const testImage = `localhost/monochromatic-lint-test:${process.env
  .MONOCHROMATIC_LINT_IMAGE_TAG
  ?? 'development'}`;
const [filter, ...extra] = process.argv
  .slice(2);
if (extra.length > 0) throw new Error('Expected at most one nextest test-name filter');
const context = await mkdtemp(join(
  scratchBase,
  'monochromatic-lint-test-'
));
const source = process.cwd();
try {
  for (const name of [
    'Cargo.toml',
    'Cargo.lock',
    'src',
    'fixtures'
  ]) {
    await cp(
      join(
        source,
        name
      ),
      join(
        context,
        'monochromatic-lint',
        name
      ),
      { recursive: true }
    );
    await cp(
      resolve(
        source,
        '../../jsonc-edit/core-rust',
        name
      ),
      join(
        context,
        'jsonc-edit',
        name
      ),
      { recursive: true }
    );
  }
  const vendorPath = join(
    context,
    'vendor'
  );
  const vendor = spawnSync(
    'cargo',
    [
      'vendor',
      '--locked',
      '--offline',
      '--versioned-dirs',
      vendorPath
    ],
    {
      encoding: 'utf8',
      maxBuffer: 8 * 1_024
        * 1_024
    }
  );
  if (vendor.error) throw vendor.error;
  if (vendor.status !== 0) throw new Error(`Cargo vendor failed: ${vendor.stderr}`);
  const config = vendor.stdout
    .replaceAll(
    JSON.stringify(vendorPath),
    '"/work/vendor"'
    );
  if (!config.includes('directory = "/work/vendor"')) throw new Error('Cargo vendor did not emit the expected source mapping');
  await mkdir(
    join(
      context,
      'cargo-config'
    ),
    { recursive: true }
  );
  await writeFile(
    join(
      context,
      'cargo-config/config.toml'
    ),
    config
  );
  await cp(
    resolve(
      source,
      '../../../clippy.toml'
    ),
    join(
      context,
      'clippy.toml'
    )
  );
  // The registry guard test validates the repository's own configuration, so the snapshot carries it.
  await cp(
    resolve(
      source,
      '../../../monochromatic-lint.config.jsonc'
    ),
    join(
      context,
      'monochromatic-lint.config.jsonc'
    )
  );
  await cp(
    join(
      source,
      'test.Containerfile'
    ),
    join(
      context,
      'Containerfile'
    )
  );
  await copyNextest({ context });
  const build = spawnSync(
    'podman',
    [
      'build',
      '--network=none',
      '--http-proxy=false',
      '--pull=never',
      '--memory=2g',
      '--cpu-period=100000',
      '--cpu-quota=200000',
      '--tag',
      testImage,
      context
    ],
    { stdio: 'inherit' }
  );
  if (build.error) throw build.error;
  if (build.status !== 0) throw new Error(`Container build failed: ${build.status}`);
  const arguments_ = [
    'run',
    '--rm',
    '--init',
    '--network=none',
    '--memory=2g',
    '--cpus=2',
    '--pids-limit=128',
    testImage
  ];
  if (filter !== undefined) arguments_.push(
    'cargo',
    'nextest',
    'run',
    '--offline',
    '--locked',
    '--all-targets',
    filter
  );
  const run = spawnSync(
    'podman',
    arguments_,
    { stdio: 'inherit' }
  );
  if (run.error) throw run.error;
  if (run.status !== 0) throw new Error(`Container test failed: ${run.status}`);
} finally {
  await rm(
    context,
    {
      recursive: true,
      force: true
    }
  );
}
