#!/usr/bin/env node
// Check that cargo-nextest reads this package's .config/nextest.toml inside the build container.
// What: three observations in the container with the shared bound.
// First, the shipped file holds a default profile with a termination timeout and the `config-probe` profile.
// Second, `--profile config-probe` is accepted with the file and refused when an empty directory hides `.config`;
// the package is its own workspace root, so this is the lookup the real tasks make.
// Third, a disposable crate with a copy of the file and a test that sleeps for a minute is killed by the probe
// profile within seconds, which is what the default profile's timeout does to a hung test.
// Why: a nextest.toml in a directory the runner does not read changes nothing and fails silently.
import { spawnSync } from 'node:child_process';
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  statSync,
  writeFileSync
} from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { containerBounds } from './container-bounds.ts';

const containerLimits = containerBounds();
console.log(containerLimits.summary);
const privateRoot = join(
  homedir(),
  'temp',
  'agent'
);
if ((statSync(privateRoot)
  .mode
  & 0o077) !== 0) throw new Error(`Scratch root must exclude group and other permissions: ${  privateRoot}`);
const config = readFileSync(
  '.config/nextest.toml',
  'utf8'
);
if (!/^\[profile\.default\]\nslow-timeout = \{ period = "[0-9]+s", terminate-after = [0-9]+ \}$/m.test(config)) {
  throw new Error('.config/nextest.toml has no default profile with a slow-timeout that terminates');
}
if (!/^\[profile\.config-probe\]\nslow-timeout = \{ period = "2s", terminate-after = 1 \}$/m.test(config)) {
  throw new Error('.config/nextest.toml has no config-probe profile with a two second timeout');
}

const scratch = mkdtempSync(join(
  privateRoot,
  'ide-nextest-config-'
));

/**
 Runs one command in the bounded build container and times it.

 @param volumes - `--volume` values to mount

 @param command - program and arguments that run in the container's `/work` directory

 @returns the exit status, the combined output, and the seconds the run took
 */
function run({
  volumes,
  command
}: {
  volumes: string[];
  command: string[];
}): { status: number | null; output: string; seconds: number } {
  const started = Date.now();
  const result = spawnSync(
    'podman',
    [
      'run',
      '--rm',
      '--network=none',
      ...containerLimits.limitArguments,
      '--security-opt',
      'label=disable',
      ...volumes.flatMap(volume => [
        '--volume',
        volume
      ]),
      '--volume',
      'ide-cargo:/cargo',
      '--workdir',
      '/work',
      ...containerLimits.environmentArguments,
      '--env',
      'SLINT_EMIT_DEBUG_INFO=1',
      '--env',
      'HELIX_RUNTIME=/work/target/debug/runtime',
      'localhost/monochromatic/ide',
      ...command,
    ],
    {
      encoding: 'utf8',
      maxBuffer: 64 * 1_024
        * 1_024
    }
  );
  if (result.error) throw result.error;
  return {
    status: result.status,
    output: (result.stdout ?? '') + (result.stderr ?? ''),
    seconds: (Date.now() - started) / 1_000
  };
}
try {
  const packageDirectory = realpathSync(process.cwd());
  // No test is named like this, so a run that finds the profile selects nothing and passes.
  const selectNothing = [
    'cargo',
    'nextest',
    'run',
    '--no-default-features',
    '--lib',
    '--profile',
    'config-probe',
    '--no-tests=pass',
    '--filter-expr',
    'test(no_such_test_name_exists)'
  ];
  const withFile = run({
    volumes: [`${packageDirectory  }:/work`],
    command: selectNothing
  });
  console.log(JSON.stringify({
    case: 'package with the file',
    status: withFile.status
  }));
  if (withFile.status !== 0) throw new Error(`nextest refused the config-probe profile although the file is present:\n${  withFile.output}`);
  const hidden = join(
    scratch,
    'empty-config'
  );
  mkdirSync(hidden);
  const withoutFile = run({
    volumes: [
      `${packageDirectory  }:/work`,
      `${hidden  }:/work/.config:ro`
    ],
    command: selectNothing
  });
  console.log(JSON.stringify({
    case: 'package with .config hidden',
    status: withoutFile.status
  }));
  if ((withoutFile.status === 0) || (!withoutFile.output
    .includes('config-probe'))) {
    throw new Error(`nextest accepted the config-probe profile without the file, so the file is not what supplies it:\n${  withoutFile.output}`);
  }
  // A disposable crate, its own workspace root like the package, with the shipped file and a test that would run a minute.
  const crate = join(
    scratch,
    'crate'
  );
  mkdirSync(
    join(
      crate,
      'src'
    ),
    { recursive: true }
  );
  mkdirSync(join(
    crate,
    '.config'
  ));
  writeFileSync(
    join(
      crate,
      'Cargo.toml'
    ),
    '[package]\nname = "hang"\nversion = "0.0.0"\nedition = "2024"\n\n[workspace]\n'
  );
  writeFileSync(
    join(
      crate,
      'src',
      'lib.rs'
    ),
    '#[test]\nfn hangs() {\n    std::thread::sleep(std::time::Duration::from_secs(60));\n}\n'
  );
  copyFileSync(
    '.config/nextest.toml',
    join(
      crate,
      '.config',
      'nextest.toml'
    )
  );
  const killed = run({
    volumes: [`${crate  }:/work`],
    command: [
      'cargo',
      'nextest',
      'run',
      '--offline',
      '--profile',
      'config-probe',
      '--color',
      'never'
    ]
  });
  console.log(JSON.stringify({
    case: 'hung test under the probe profile',
    status: killed.status,
    seconds: killed.seconds
  }));
  if ((killed.status === 0) || (!killed.output
    .includes('TIMEOUT'))
    || (killed.seconds > 45)) {
    throw new Error(`The hung test was not terminated by the probe profile within seconds:\n${  killed.output}`);
  }
  console.log('nextest reads .config/nextest.toml of the package and terminates a hung test');
} finally {
  rmSync(
    scratch,
    {
      recursive: true,
      force: true
    }
  );
}
