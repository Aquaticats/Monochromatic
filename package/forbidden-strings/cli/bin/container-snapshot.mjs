#!/usr/bin/env node
/** Scanner-owned source snapshots for offline, mount-free verification. */
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import {
  closeSync,
  openSync,
  readFileSync
} from 'node:fs';
import {
  cp,
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  rm,
  writeFile
} from 'node:fs/promises';
import { homedir } from 'node:os';
import {
  join,
  resolve
} from 'node:path';
import { mkdirSync as makeScratchBase } from 'node:fs';

// Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
// whose backing device is unknowable.
const scratchBase = join(import.meta.dirname, '..', 'node_modules', '.monochromatic', 'forbidden-strings');
makeScratchBase(scratchBase, { recursive: true });


/** Failed setup or incomplete evidence never counts as a verification pass. */
export class ScannerVerificationError extends Error {}

/** Run shell-free management commands; allow failures only when their evidence is retained. */
export function run({
  command,
  args,
  capture = false,
  allowFailure = false,
  transcript
}) {
  const descriptors = transcript ? [
    openSync(
      join(
        transcript,
        'stdout.log'
      ),
      'w'
    ),
    openSync(
      join(
        transcript,
        'stderr.log'
      ),
      'w'
    )
  ] : [];
  let result;
  try {
    result = spawnSync(
      command,
      args,
      {
        cwd: process.cwd(),
        encoding: 'utf8',
        stdio: transcript ? [
          'ignore',
          ...descriptors
        ] : capture ? [
          'ignore',
          'pipe',
          'pipe'
        ] : 'inherit',
        maxBuffer: 32 * 1_024
          * 1_024,
      }
    );
  } finally {
    for (const descriptor of descriptors) closeSync(descriptor);
  }
  if (transcript) {
    // The full bytes stay in the evidence directory; terminal output is only a bounded preview.
    for (const filename of [
      'stdout.log',
      'stderr.log'
    ])
      console.log(readFileSync(
        join(
          transcript,
          filename
        ),
        'utf8'
      )
        .slice(-16_000));
  }
  if (result.error) throw new ScannerVerificationError(
    'Verification command could not start.',
    { cause: result.error }
  );
  if ((!allowFailure) && (result.status !== 0))
    throw new ScannerVerificationError(`${command} ${args[0]} exited ${result.status}: ${result.stderr ?? ''}`);
  return result;
}

/** Hash the actual copied bytes, not a possibly dirty Git tree identifier alone. */
async function inventory({
  root,
  relative = ''
}) {
  const records = [];
  const entries = await readdir(
    join(
      root,
      relative
    ),
    { withFileTypes: true }
  );
  entries.sort((
    first,
    second
  ) => first.name
    .localeCompare(second.name));
  for (const entry of entries) {
    const name = join(
      relative,
      entry.name
    );
    if (entry.isDirectory()) records.push(...await inventory({
      root,
      relative: name
    }));
    else if (entry.isFile()) {
      const bytes = await readFile(join(
        root,
        name
      ));
      records.push({
        path: name,
        sha256: createHash('sha256')
          .update(bytes)
          .digest('hex')
      });
    } else throw new ScannerVerificationError(`Unexpected snapshot entry: ${name}`);
  }
  return records;
}

/** Bake scanner, engine and optional fuzz sidecar inputs into a content-addressed container. */
export async function snapshot({
  command,
  name,
  toolchain = false,
  tool,
  fuzzing = false,
  transform
}) {
  const context = await mkdtemp(join(
    scratchBase,
    'scanner-snapshot-'
  ));
  const evidenceRoot = resolve('target/verification');
  await mkdir(
    evidenceRoot,
    { recursive: true }
  );
  const evidence = await mkdtemp(join(
    evidenceRoot,
    `${name}-`
  ));
  try {
    const base = '6ec87f6d290a2f59bda5b3ffd4197058fe0749d02b4978c877e8edf6dc38802a';
    const scanner = 'package/forbidden-strings/cli';
    const packages = [
      {
        source: resolve(
          import.meta.dirname,
          '..'
        ),
        destination: scanner,
        files: [
          'Cargo.toml',
          'Cargo.lock',
          'build.rs',
          'src',
          'tests',
          'data',
          'bin',
          'mise.toml'
        ]
      },
      {
        source: resolve(
          import.meta.dirname,
          '../../../forbidden-strings/core'
        ),
        destination: 'package/forbidden-strings/core',
        files: [
          'Cargo.toml',
          'Cargo.lock',
          'src'
        ]
      },
    ];
    if (fuzzing) packages.push({
      source: resolve(
        import.meta.dirname,
        '../../cli.fuzz'
      ),
      destination: `${scanner}.fuzz`,
      files: [
        'Cargo.toml',
        'Cargo.lock',
        'src',
        'fuzz_targets',
        'seed',
        'dictionary',
        'bin',
        'mise.toml'
      ],
    });
    for (const item of packages) {
      for (const file of item.files)
        await cp(
          join(
            item.source,
            file
          ),
          join(
            context,
            item.destination,
            file
          ),
          { recursive: true }
        );
    }
    const repository = resolve(
      import.meta.dirname,
      '../../../..'
    );
    for (const file of [
      'forbidden-strings.append.txt',
      'clippy.toml'
    ])
      await cp(
        join(
          repository,
          file
        ),
        join(
          context,
          file
        )
      );
    if (transform) await transform({
      context,
      scanner
    });
    const sources = await inventory({ root: context });
    const sourceSha256 = createHash('sha256')
      .update(JSON.stringify(sources))
      .digest('hex');
    const vendor = join(
      context,
      'vendor'
    );
    const vendorArgs = [
      'vendor',
      '--manifest-path',
      join(
        context,
        scanner,
        'Cargo.toml'
      ),
      '--offline',
      '--locked',
      '--versioned-dirs'
    ];
    if (fuzzing) vendorArgs.push(
      '--sync',
      join(
        context,
        `${scanner}.fuzz`,
        'Cargo.toml'
      )
    );
    const configuration = run({
      command: 'cargo',
      args: [
        ...vendorArgs,
        vendor
      ],
      capture: true
    })
      .stdout;
    const mapped = configuration.replaceAll(
      JSON.stringify(vendor),
      '"/work/vendor"'
    );
    if (!mapped.includes('directory = "/work/vendor"'))
      throw new ScannerVerificationError('Cargo vendor did not provide the expected offline source mapping.');
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
      mapped
    );
    const additional = [];
    const tools = {};
    if (toolchain) {
      const root = run({
        command: 'rustc',
        args: [
          '--print',
          'sysroot'
        ],
        capture: true
      })
        .stdout
        .trim();
      for (const directory of [
        'bin',
        'lib'
      ])
        await cp(
          join(
            root,
            directory
          ),
          join(
            context,
            'toolchain',
            directory
          ),
          { recursive: true }
        );
      tools.rustc = run({
        command: 'rustc',
        args: [
          '--version',
          '--verbose'
        ],
        capture: true
      })
        .stdout
        .trim();
      additional.push(
        'COPY toolchain /toolchain',
        'ENV PATH=/toolchain/bin:/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin RUSTC=/toolchain/bin/rustc RUSTDOC=/toolchain/bin/rustdoc'
      );
    }
    if (tool) {
      const executable = join(
        process.env
          .CARGO_HOME
          ?? join(
            homedir(),
            '.cargo'
          ),
        'bin',
        tool
      );
      const bytes = await readFile(executable);
      tools[tool] = createHash('sha256')
        .update(bytes)
        .digest('hex');
      await cp(
        executable,
        join(
          context,
          tool
        )
      );
      additional.push(`COPY ${tool} /usr/local/cargo/bin/${tool}`);
    }
    const workdir = fuzzing ? `${scanner}.fuzz` : scanner;
    await writeFile(
      join(
        context,
        'Containerfile'
      ),
      [
        `FROM ${base}`,
        ...additional,
        'COPY package /work/package',
        'COPY vendor /work/vendor',
        'COPY cargo-config /work/.cargo',
        'COPY forbidden-strings.append.txt clippy.toml /work/',
        'RUN ["mkdir", "--parents", "/home/tester/.cargo"]',
        'RUN ["chown", "--recursive", "1000:1000", "/work", "/home/tester"]',
        'ENV HOME=/home/tester CARGO_HOME=/home/tester/.cargo CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RAYON_NUM_THREADS=2',
        'USER 1000:1000',
        `WORKDIR /work/${workdir}`,
        `CMD ${JSON.stringify(command)}`,
        '',
      ].join('\n')
    );
    const imageTag = `localhost/scanner-${name}:${sourceSha256}`;
    const imageFile = join(
      context,
      'image.id'
    );
    run({
      command: 'podman',
      args: [
        'build',
        '--network=none',
        '--http-proxy=false',
        '--pull=never',
        '--memory=2g',
        '--cpu-period=100000',
        '--cpu-quota=200000',
        '--iidfile',
        imageFile,
        '--tag',
        imageTag,
        context
      ]
    });
    const image = (await readFile(
      imageFile,
      'utf8'
    )).trim();
    const limits = [
      '--init',
      '--network=none',
      '--memory=2g',
      '--cpus=2',
      '--pids-limit=128',
      '--ulimit',
      'nofile=4096:4096'
    ];
    const version = run({
      command: 'podman',
      args: [
        'run',
        '--rm',
        ...limits,
        image,
        '/usr/bin/git',
        '--version'
      ],
      capture: true
    })
      .stdout
      .trim();
    if (version !== 'git version 2.56.0') throw new ScannerVerificationError(`Wrong Git in scanner fixture: ${version}`);
    const head = run({
      command: 'git',
      args: [
        '-C',
        repository,
        'rev-parse',
        'HEAD'
      ],
      capture: true
    })
      .stdout
      .trim();
    await writeFile(
      join(
        evidence,
        'manifest.json'
      ),
      `${JSON.stringify(
        {
          base,
          image,
          head,
          sourceSha256,
          sources,
          tools,
          version,
          limits,
          user: '1000:1000',
          command
        },
        null,
        2
      )  }\n`
    );
    console.log(`Scanner ${name} evidence: ${evidence}`);
    return {
      context,
      evidence,
      image,
      limits
    };
  } catch (error) {
    await writeFile(
      join(
        evidence,
        'setup-failure.txt'
      ),
      `${String(error)  }\n`
    );
    await rm(
      context,
      {
        recursive: true,
        force: true
      }
    );
    throw error;
  }
}
