#!/usr/bin/env node
// Time external writes until they reach the tree rows and the displayed source, in a disposable package copy.
// Optional `package` measures another crate directory instead, with this checkout's latency test injected,
// so a polling build and a watching build run the identical fixture, trial gaps, and container bounds.
import { spawnSync } from 'node:child_process';
import {
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  statSync,
  writeFileSync
} from 'node:fs';
import { homedir } from 'node:os';
import {
  join,
  relative,
  sep
} from 'node:path';
import { containerBounds } from './container-bounds.ts';
import { gpuArguments } from './gpu-arguments.ts';

// The bound of every container this script starts: the shared computed share of the host, or the caller's IDE_CONTAINER_* values.
const containerLimits = containerBounds();

const privateRoot = join(
  homedir(),
  'temp',
  'agent'
);
if ((statSync(privateRoot)
  .mode
  & 0o077) !== 0) throw new Error(`Scratch root must exclude group and other permissions: ${  privateRoot}`);
if (!process.env
  .usage_cache) throw new Error('Provide the disposable Cargo target-cache directory');
const cache = realpathSync(process.env
  .usage_cache);
if ((!cache.startsWith(realpathSync(privateRoot) + sep)) || (!statSync(cache)
  .isDirectory())) throw new Error(`Target cache must be a disposable directory below ${  privateRoot}`);
let cargoHome = 'ide-cargo';
if (process.env
  .usage_cargo) {
  cargoHome = realpathSync(process.env
    .usage_cargo);
  if ((!cargoHome.startsWith(realpathSync(privateRoot) + sep)) || (!statSync(cargoHome)
    .isDirectory())) throw new Error(`Cargo home copy must be a disposable directory below ${  privateRoot}`);
}
const runs = Number(process.env
  .usage_runs
  ?? '3');
if ((!Number.isInteger(runs)) || (runs < 1)) throw new Error('runs must be a positive integer');
// Another package directory, for example `git archive` of an older commit, measured with this checkout's test.
const other = process.env
  .usage_package ? realpathSync(process.env
    .usage_package) : undefined;
if (other && ((!other.startsWith(realpathSync(privateRoot) + sep)) || (!existsSync(join(
  other,
  'Cargo.toml'
))))) throw new Error(`Package copy must be a crate directory below ${  privateRoot}`);
// Name part of the ignored native measurement tests to run; other measurements print their own JSON lines.
const filter = process.env
  .usage_filter
  ?? 'refresh_latency';
if (!/^[a-z_]+$/.test(filter)) throw new Error('filter must be a test name part of lowercase letters and underscores');
const artifact = mkdtempSync(join(
  privateRoot,
  'ide-refresh-latency-'
));
const source = join(
  artifact,
  'package'
);
const origin = process.cwd();
const testFile = 'src/native/refresh_latency_tests.rs';
if (other) {
  cpSync(
    other,
    source,
    {
      recursive: true,
      filter: path => ![
        'target',
        '.git',
        'node_modules'
      ].includes(relative(
        other,
        path
      )
        .split(sep)[0] ?? '')
    }
  );
  cpSync(
    join(
      origin,
      testFile
    ),
    join(
      source,
      testFile
    )
  );
  const nativePath = join(
    source,
    'src/native.rs'
  );
  const native = readFileSync(
    nativePath,
    'utf8'
  );
  if (!native.includes('mod refresh_latency_tests;')) {
    const anchor = '#[cfg(test)]\nmod pointer_tests;\n';
    if (!native.includes(anchor)) throw new Error(`Cannot find the test-module anchor in ${  nativePath}`);
    writeFileSync(
      nativePath,
      native.replace(
        anchor,
        `${anchor  }#[cfg(test)]\nmod refresh_latency_tests;\n`
      )
    );
  }
} else {
  cpSync(
    origin,
    source,
    {
      recursive: true,
      filter: path => ![
        'target',
        '.git',
        'node_modules'
      ].includes(relative(
        origin,
        path
      )
        .split(sep)[0] ?? '')
    }
  );
}
mkdirSync(
  join(
    source,
    'target'
  ),
  { recursive: true }
);
console.log(`REFRESH_LATENCY_ARTIFACT=${  artifact
   }${other ? ` (package ${  other  })` : ' (working tree)'}`);

const results = [];
for (let run = 1; run <= runs; run++) {
  const result = spawnSync(
    'podman',
    [
      'run',
      '--rm',
      '--network=none',
      ...containerLimits.limitArguments,
      '--security-opt',
      'label=disable',
      '--volume',
      `${source  }:/work`,
      '--volume',
      `${cache  }:/work/target`,
      '--volume',
      `${cargoHome  }:/cargo`,
      '--workdir',
      '/work',
      ...containerLimits.environmentArguments,
      '--env',
      'SLINT_EMIT_DEBUG_INFO=1',
      ...gpuArguments,
      '--env',
      'HELIX_RUNTIME=/work/target/debug/runtime',
      'localhost/monochromatic/ide',
      'cargo',
      'nextest',
      'run',
      '--offline',
      '--bin',
      'monochromatic-ide',
      '--run-ignored',
      'only',
      '--no-capture',
      '--filter-expr',
      `test(${  filter  })`,
    ],
    {
      encoding: 'utf8',
      maxBuffer: 64 * 1_024
        * 1_024
    }
  );
  const output = (result.stdout ?? '') + (result.stderr ?? '');
  writeFileSync(
    join(
      artifact,
      `run-${  run  }.log`
    ),
    output
  );
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`Latency run ${  run  } failed; inspect ${  join(
    artifact,
    `run-${  run  }.log`
  )}`);
  for (const line of output.split('\n')) {
    if (line.startsWith('{"case":')) results.push({
      run,
      ...JSON.parse(line)
    });
  }
  writeFileSync(
    join(
      artifact,
      'results.json'
    ),
    JSON.stringify(
      {
        package: other ?? origin,
        filter,
        results
      },
      null,
      2
    )
  );
  for (const item of results.filter(entry => entry.run === run)) {
    if (item.median_ms === undefined) console.log(JSON.stringify(item));
    else console.log([
      run,
      item.case,
      `expanded=${  item.expanded}`,
      `median=${  item.median_ms}`,
      `p90=${  item.p90_ms}`,
      `max=${  item.max_ms}`
    ].join(' '));
  }
}
console.log(`Refresh latency results: ${  join(
  artifact,
  'results.json'
)}`);
