#!/usr/bin/env node
/** Build instrumented targets with a read-only compiler, then fuzz without host mounts. */
import { spawnSync } from 'node:child_process';
import { constants } from 'node:fs';
import {
  access,
  copyFile,
  cp,
  mkdir,
  mkdtemp,
  rm,
  stat,
  writeFile
} from 'node:fs/promises';
import {
  delimiter,
  join,
  resolve
} from 'node:path';

// The generator controls run in the build image with the runner the git-policy/cli gate pins by version and checksum.
import {
  copyNextest,
  nextestCopyLines
} from '../../../git-policy/cli/bin/native-nextest-snapshot.mjs';
import { mkdirSync as makeScratchBase } from 'node:fs';

// Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
// whose backing device is unknowable.
const scratchBase = join(import.meta.dirname, '..', 'node_modules', '.monochromatic', 'monochromatic-lint-fuzz');
makeScratchBase(scratchBase, { recursive: true });


/** Concurrent worktrees and agents set MONOCHROMATIC_LINT_IMAGE_TAG so one campaign never runs another's image. */
const imageTag = process.env
  .MONOCHROMATIC_LINT_IMAGE_TAG
  ?? 'development';
/** Instrumented build image: package inputs, vendored dependencies and cargo-fuzz. */
const buildImage = `localhost/monochromatic-lint-fuzz-build:${imageTag}`;
/** Mount-free run image: instrumented binaries, seeds and dictionary only. */
const runImage = `localhost/monochromatic-lint-fuzz-run:${imageTag}`;

/** Preserve the failed verification boundary rather than masking it with a later copy error. */
class FuzzVerificationError extends Error {}

/** Run one shell-free command, capturing metadata only when requested. */
function execute({
  command,
  args,
  cwd = process.cwd(),
  capture = false,
  allowFailure = false
}) {
  const result = spawnSync(
    command,
    args,
    {
      cwd,
      encoding: 'utf8',
      stdio: capture ? [
        'ignore',
        'pipe',
        'pipe'
      ] : 'inherit',
      maxBuffer: 16 * 1_024
        * 1_024,
    }
  );
  if (result.error)
    throw new FuzzVerificationError(
      `${command} could not start.`,
      { cause: result.error }
    );
  if ((!allowFailure) && (result.status !== 0))
    throw new FuzzVerificationError(`${command} ${args[0]} exited ${result.status}: ${result.stderr ?? ''}`);
  return result;
}

/** Resolve the actual PATH-selected tool rather than an older unrelated Cargo-home installation. */
async function executable(name) {
  for (const directory of (process.env
    .PATH
    ?? '').split(delimiter)) {
    const candidate = resolve(
      directory,
      name
    );
    try {
      await access(
        candidate,
        constants.X_OK
      );
      if ((await stat(candidate)).isFile())
        return candidate;
    } catch (error) {
      if ((error.code !== 'ENOENT') && (error.code !== 'ENOTDIR')
        && (error.code !== 'EACCES'))
        throw error;
    }
  }
  throw new FuzzVerificationError(`Executable ${name} was not found on PATH.`);
}

/** Copy only package inputs, never its caches, dependencies, or private machine-local files. */
async function copyPackage({
  source,
  destination,
  fuzz
}) {
  for (const name of [
    'Cargo.toml',
    'Cargo.lock',
    'src'
  ])
    await cp(
      join(
        source,
        name
      ),
      join(
        destination,
        name
      ),
      { recursive: true }
    );
  if (fuzz)
    await cp(
      join(
        source,
        'fuzz_targets'
      ),
      join(
        destination,
        'fuzz_targets'
      ),
      { recursive: true }
    );
}

/** Build and run within explicit container limits and retain discoveries before cleanup. */
async function main() {
  if (process.argv
    .slice(2)
    .some(argument => argument !== '--build-only'))
    throw new FuzzVerificationError('Only --build-only is accepted.');
  const buildOnly = process.argv
    .includes('--build-only');
  const source = process.cwd();
  const compiler = execute({
    command: 'rustc',
    args: [
      '--print',
      'sysroot'
    ],
    capture: true
  })
    .stdout
    .trim();
  const compilerVersion = execute({
    command: 'rustc',
    args: ['--version'],
    capture: true
  })
    .stdout
    .trim();
  if (!compilerVersion.includes('nightly'))
    throw new FuzzVerificationError('Fuzz instrumentation requires the repository-managed nightly Rust toolchain.');
  const context = await mkdtemp(join(
    scratchBase,
    'monochromatic-lint-fuzz-'
  ));
  const outputRoot = join(
    source,
    'target',
    'verification'
  );
  await mkdir(
    outputRoot,
    { recursive: true }
  );
  const evidence = await mkdtemp(join(
    outputRoot,
    'campaign-'
  ));
  const containers = [];
  const limits = [
    '--init',
    '--network=none',
    '--memory=2g',
    '--cpus=2',
    '--pids-limit=128'
  ];
  try {
    const packages = [
      {
        source,
        destination: 'linter/monochromatic-lint.fuzz',
        fuzz: true
      },
      {
        source: resolve(
          source,
          '../monochromatic-lint'
        ),
        destination: 'linter/monochromatic-lint',
        fuzz: false
      },
      {
        source: resolve(
          source,
          '../../jsonc-edit/core-rust'
        ),
        destination: 'jsonc-edit/core-rust',
        fuzz: false
      },
      {
        source: resolve(
          source,
          '../../jsonc-edit/core-rust.fuzz'
        ),
        destination: 'jsonc-edit/core-rust.fuzz',
        fuzz: true
      },
    ];
    for (const item of packages)
      await copyPackage({
        source: item.source,
        destination: join(
          context,
          'package',
          item.destination
        ),
        fuzz: item.fuzz
      });
    await cp(
      resolve(
        source,
        '../monochromatic-lint/fixtures'
      ),
      join(
        context,
        'package/linter/monochromatic-lint/fixtures'
      ),
      { recursive: true }
    );
    await cp(
      resolve(
        source,
        '../../jsonc-edit/core-rust/fixtures'
      ),
      join(
        context,
        'package/jsonc-edit/core-rust/fixtures'
      ),
      { recursive: true }
    );
    const vendor = execute({
      command: 'cargo',
      args: [
        'vendor',
        '--locked',
        '--offline',
        '--versioned-dirs',
        join(
          context,
          'vendor'
        )
      ],
      capture: true
    });
    const config = vendor.stdout
      .replaceAll(
      JSON.stringify(join(
        context,
        'vendor'
      )),
      '"/work/vendor"'
      );
    if (!config.includes('directory = "/work/vendor"'))
      throw new FuzzVerificationError('Cargo vendor did not emit the expected source directory mapping.');
    await mkdir(
      join(
        context,
        '.cargo'
      ),
      { recursive: true }
    );
    await writeFile(
      join(
        context,
        '.cargo/config.toml'
      ),
      config
    );
    const cargoFuzz = await executable('cargo-fuzz');
    await copyFile(
      cargoFuzz,
      join(
        context,
        'cargo-fuzz'
      )
    );
    await copyNextest({ context });
    await writeFile(
      join(
        context,
        'Containerfile'
      ),
      [
        '# Same installed Rust runtime image as the subject container tests; compiler is a separate read-only input.',
        'FROM 62ba2f7ce22ba9bc501110d3452c7ae814fba367c46f7eea3629a79286353884',
        'COPY package /work/package',
        'COPY vendor /work/vendor',
        'COPY .cargo /work/.cargo',
        'COPY cargo-fuzz /usr/local/cargo/bin/cargo-fuzz',
        'RUN ["cargo-fuzz", "--version"]',
        ...nextestCopyLines(),
        'WORKDIR /work/package/linter/monochromatic-lint.fuzz',
      // The optimized HIR ASAN build was SIGKILLed under the 2 GiB cap with two compiler jobs.
        'ENV CARGO_BUILD_JOBS=1 CARGO_NET_OFFLINE=true CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0',
        'ENV PATH="/toolchain/bin:/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"',
        'ENV RUSTC="/toolchain/bin/rustc" RUSTDOC="/toolchain/bin/rustdoc"',
        '',
      ].join('\n')
    );
    execute({
      command: 'podman',
      args: [
        'build',
        '--network=none',
        '--http-proxy=false',
        '--pull=never',
        '--memory=2g',
        '--cpu-period=100000',
        '--cpu-quota=200000',
        '--tag',
        buildImage,
        context
      ]
    });
    // Compiler files retain their host labels. Do not relabel a shared Rust installation with :Z.
    // Only build containers disable label isolation; fuzz execution has no host mounts and keeps it enabled.
    const compilerMount = [
      '--security-opt',
      'label=disable',
      '--volume',
      `${compiler}:/toolchain:ro`
    ];
    execute({
      command: 'podman',
      args: [
        'run',
        '--rm',
        ...limits,
        ...compilerMount,
        buildImage,
        'cargo',
        'nextest',
        'run',
        '--lib',
        '--offline',
        '--locked'
      ]
    });
    // Type/lint the property helpers in the same bounded compiler context as their generator controls.
    execute({
      command: 'podman',
      args: [
        'run',
        '--rm',
        ...limits,
        ...compilerMount,
        buildImage,
        'cargo',
        'clippy',
        '--lib',
        '--offline',
        '--locked',
        '--',
        '-D',
        'warnings'
      ]
    });
    const buildContainer = execute({
      command: 'podman',
      args: [
        'create',
        ...limits,
        ...compilerMount,
        buildImage,
        'cargo',
        'fuzz',
        'build',
        '--fuzz-dir',
        '.',
        '--sanitizer',
        'address',
        '--codegen-units',
        '16',
        '--target',
        'x86_64-unknown-linux-gnu',
        '--target-dir',
        '/work/build',
      ],
      capture: true
    })
      .stdout
      .trim();
    containers.push(buildContainer);
    const built = execute({
      command: 'podman',
      args: [
        'start',
        '--attach',
        buildContainer
      ],
      allowFailure: true
    });
    const buildState = execute({
      command: 'podman',
      args: [
        'inspect',
        '--format',
        '{{json .State}}',
        buildContainer
      ],
      capture: true
    });
    await writeFile(
      join(
        evidence,
        'build-state.json'
      ),
      buildState.stdout
    );
    if (built.status !== 0)
      throw new FuzzVerificationError(`Instrumented build exited ${built.status}; container state retained in ${evidence}.`);
    const targets = [
      'merge_values',
      'configuration',
      'rust_style',
      'rust_explicit_types',
      'markdown',
      'orchestration'
    ];
    await mkdir(
      join(
        context,
        'bin'
      ),
      { recursive: true }
    );
    for (const target of targets)
      execute({
        command: 'podman',
        args: [
          'cp',
          `${buildContainer}:/work/build/x86_64-unknown-linux-gnu/release/${target}`,
          join(
            context,
            'bin',
            target
          )
        ]
      });
    await writeFile(
      join(
        evidence,
        'manifest.json'
      ),
      `${JSON.stringify(
        {
          compilerVersion,
          targets,
          sanitizer: 'address',
          buildJobs: 1,
          codegenUnits: 16,
          maxInputBytes: 8_192,
          secondsPerTarget: 30,
          memory: '2g',
          cpus: 2,
          compilerMount: 'read-only during compilation with container label isolation disabled; absent during fuzzing'
        },
        null,
        2
      )  }\n`
    );
    if (buildOnly) {
      await cp(
        join(
          context,
          'bin'
        ),
        join(
          evidence,
          'bin'
        ),
        { recursive: true }
      );
      console.log(`Instrumented binaries: ${evidence}`);
      return;
    }
    await cp(
      join(
        source,
        'seed'
      ),
      join(
        context,
        'corpus'
      ),
      { recursive: true }
    );
    await cp(
      join(
        source,
        'dictionary'
      ),
      join(
        context,
        'dictionary'
      ),
      { recursive: true }
    );
    await writeFile(
      join(
        context,
        'Run.Containerfile'
      ),
      [
        '# Fuzz execution has no compiler, registry, repository, home, or credential mounts.',
        'FROM 62ba2f7ce22ba9bc501110d3452c7ae814fba367c46f7eea3629a79286353884',
        'COPY bin /fuzz/bin',
        'COPY corpus /fuzz/corpus',
        'COPY dictionary /fuzz/dictionary',
        `RUN ${JSON.stringify([
          'mkdir',
          '--parents',
          ...targets.map(target => `/fuzz/artifacts/${target}`)
        ])}`,
        'WORKDIR /fuzz',
        '',
      ].join('\n')
    );
    execute({
      command: 'podman',
      args: [
        'build',
        '--network=none',
        '--http-proxy=false',
        '--pull=never',
        '--memory=2g',
        '--cpu-period=100000',
        '--cpu-quota=200000',
        '--tag',
        runImage,
        '--file',
        join(
          context,
          'Run.Containerfile'
        ),
        context
      ]
    });
    console.log(`Fuzz evidence: ${evidence}`);
    for (const target of targets) {
      const dictionary = target === 'configuration' ? ['-dict=/fuzz/dictionary/configuration.dict'] : [];
      const container = execute({
        command: 'podman',
        args: [
          'create',
          ...limits,
          runImage,
          `/fuzz/bin/${target}`,
          `/fuzz/corpus/${target}`,
          '-max_total_time=30',
          '-max_len=8192',
          '-rss_limit_mb=1536',
          '-timeout=5',
          '-print_final_stats=1',
          `-artifact_prefix=/fuzz/artifacts/${target}/`,
          ...dictionary,
        ],
        capture: true
      })
        .stdout
        .trim();
      containers.push(container);
      const result = execute({
        command: 'podman',
        args: [
          'start',
          '--attach',
          container
        ],
        capture: true,
        allowFailure: true
      });
      await mkdir(
        join(
          evidence,
          target
        ),
        { recursive: true }
      );
      await writeFile(
        join(
          evidence,
          target,
          'stdout.log'
        ),
        result.stdout
      );
      await writeFile(
        join(
          evidence,
          target,
          'stderr.log'
        ),
        result.stderr
      );
      execute({
        command: 'podman',
        args: [
          'cp',
          `${container}:/fuzz/corpus/${target}`,
          join(
            evidence,
            target,
            'corpus'
          )
        ]
      });
      execute({
        command: 'podman',
        args: [
          'cp',
          `${container}:/fuzz/artifacts/${target}`,
          join(
            evidence,
            target,
            'artifacts'
          )
        ]
      });
      const statistics = /stat::number_of_executed_units:\s+(\d+)/u.exec((`${result.stdout}\n${result.stderr}`));
      const executedUnits = statistics ? Number(statistics[1]) : 0;
      await writeFile(
        join(
          evidence,
          target,
          'exit.json'
        ),
        `${JSON.stringify({
          status: result.status,
          signal: result.signal,
          executedUnits
        })  }\n`
      );
      if ((result.status !== 0) || (executedUnits < 1))
        throw new FuzzVerificationError(`${target} exited ${result.status} after ${executedUnits} verified executions; retained inputs and logs are in ${evidence}.`);
      console.log(`${target}: ${executedUnits} executions, exit 0; evidence ${join(
        evidence,
        target
      )}`);
    }
  } finally {
    for (const container of containers)
      execute({
        command: 'podman',
        args: [
          'rm',
          '--force',
          container
        ],
        capture: true
      });
    await rm(
      context,
      {
        recursive: true,
        force: true
      }
    );
  }
}

await main();
