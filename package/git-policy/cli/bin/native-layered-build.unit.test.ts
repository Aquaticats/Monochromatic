import {
  chmod,
  mkdir,
  mkdtempDisposable,
  readFile,
  stat,
  utimes,
  writeFile,
} from 'node:fs/promises';
import {
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  imageRootDirectory,
  touchedCargoRun,
  writeDependencyStubs,
} from './native-layered-build.ts';
import {
  runArguments,
  testContainerfile,
} from './native-test-containerfile.ts';
import { runCommand, } from './native-verification-process.ts';
import {
  pathCrates,
  subjectCrate,
  wrapperCrates,
} from './native-wrapper-snapshot.ts';


/**
 Image the generated Containerfile is built from; any text works because the tests read lines, never build.
 */
const baseImage = 'base-image-for-line-checks';

/**
 Instruction word of a Containerfile line.
 */
function instructionOf(line: string): string {
  return line.split(' ')[0] ?? '';
}

/**
 Lines of the last stage of a Containerfile, which is the test image.

 @param text - Containerfile text

 @returns the lines from the final `FROM`
 */
function lastStage(text: string): readonly string[] {
  /**
   Every line of the Containerfile.
   */
  const lines = text.split('\n');
  /**
   Position where the last stage begins.
   */
  const start = lines.findLastIndex(function isFrom(line) {
    return instructionOf(line) === 'FROM';
  });
  return lines.slice(start);
}

await describe({
  name: touchedCargoRun.name,
  children: [
    it({
      name: 'touches every source file, then passes each Cargo argument through unchanged',
      fn: async () => {
        /**
         Disposable directory holding the fake sources and a fake `cargo`.
         */
        await using root = await mkdtempDisposable(join(
          scratchRoot(),
          'layered-build-',
        ));
        /**
         Directory whose name carries a quote, which the shell step must not misread.
         */
        const sources = join(
          root.path,
          'it\'s sources',
        );
        await mkdir(sources);
        /**
         File the step must touch.
         */
        const source = join(
          sources,
          'lib.rs',
        );
        await writeFile(
          source,
          '',
        );
        /**
         A modification time far before any build.
         */
        const longAgo = new Date(0);
        await utimes(
          source,
          longAgo,
          longAgo,
        );
        /**
         Where the fake `cargo` records the arguments it received.
         */
        const record = join(
          root.path,
          'received.json',
        );
        /**
         Directory placed first on `PATH`, holding the fake `cargo`.
         */
        const fakeBin = join(
          root.path,
          'bin',
        );
        await mkdir(fakeBin);
        await writeFile(
          join(
            fakeBin,
            'cargo',
          ),
          `#!/usr/bin/env node\nprocess.getBuiltinModule('node:fs').writeFileSync(${JSON.stringify(record)}, JSON.stringify(process.argv.slice(2)));\n`,
        );
        await chmod(
          join(
            fakeBin,
            'cargo',
          ),
          0o755,
        );
        /**
         Arguments with spaces, a quote and a shell expansion, to prove they reach Cargo as given.
         */
        const cargoArguments = [
          'nextest',
          'two words',
          'it\'s',
          '$(not-run)',
        ];
        /**
         The generated instruction, which holds an exec-form JSON array after `RUN `.
         */
        const instruction = touchedCargoRun({
          touchedPaths: [sources],
          cargoArguments,
        });
        expect(instructionOf(instruction)).toBe('RUN');
        /**
         The program, flag and command the instruction starts.
         */
        const [
          program,
          flag,
          command,
        ] = JSON.parse(instruction.slice('RUN '.length)) as [string, string, string];
        await runCommand({
          command: program,
          args: [
            flag,
            command,
          ],
          capture: true,
          env: {
            ...process.env,
            PATH: `${fakeBin}:${process.env.PATH ?? ''}`,
          },
        });
        expect(
          JSON.parse(await readFile(
            record,
            'utf8',
          )),
        ).toEqual(cargoArguments);
        expect((await stat(source)).mtimeMs).toBeGreaterThan(longAgo.getTime());
      },
    }),
  ],
});

await describe({
  name: writeDependencyStubs.name,
  children: [
    it({
      name: 'writes stand-ins at every declared target path that let Cargo check the crate without its sources',
      fn: async () => {
        /**
         Disposable tree root holding a one-crate fixture.
         */
        await using root = await mkdtempDisposable(join(
          scratchRoot(),
          'layered-stubs-',
        ));
        /**
         Crate directory below `package`.
         */
        const crate = 'fixture/crate';
        /**
         Crate directory in the real tree.
         */
        const real = join(
          root.path,
          'package',
          crate,
        );
        /**
         Target paths the fixture manifest declares, relative to the crate.
         */
        const targetPaths = {
          lib: 'src/root.rs',
          bin: 'src/entry.rs',
          test: 'tests/probe.rs',
          build: 'build.rs',
        } as const;
        /**
         Text only the real library file holds, which a stand-in must not carry.
         */
        const realContent = 'pub fn real_item() {}\n';
        await mkdir(
          join(
            real,
            'src',
          ),
          { recursive: true },
        );
        await mkdir(join(
          real,
          'tests',
        ));
        await writeFile(
          join(
            real,
            'Cargo.toml',
          ),
          [
            '[package]',
            'name = "layered-fixture"',
            'version = "0.1.0"',
            'edition = "2021"',
            '[lib]',
            `path = "${targetPaths.lib}"`,
            '[[bin]]',
            'name = "entry"',
            `path = "${targetPaths.bin}"`,
            '[[test]]',
            'name = "probe"',
            `path = "${targetPaths.test}"`,
            '[workspace]',
            '',
          ].join('\n'),
        );
        await writeFile(
          join(
            real,
            targetPaths.lib,
          ),
          realContent,
        );
        await writeFile(
          join(
            real,
            targetPaths.bin,
          ),
          'fn main() { layered_fixture::real_item(); }\n',
        );
        await writeFile(
          join(
            real,
            targetPaths.test,
          ),
          '#[test]\nfn uses_real() { layered_fixture::real_item(); }\n',
        );
        await writeFile(
          join(
            real,
            targetPaths.build,
          ),
          'fn main() { println!("cargo:rerun-if-changed=build.rs"); }\n',
        );
        await runCommand({
          command: 'cargo',
          args: [
            'generate-lockfile',
            '--offline',
            '--manifest-path',
            join(
              real,
              'Cargo.toml',
            ),
          ],
          capture: true,
        });
        await writeDependencyStubs({
          treeRoot: root.path,
          crates: [crate],
        });
        /**
         Crate directory in the stand-in tree.
         */
        const stub = join(
          root.path,
          imageRootDirectory,
          'package',
          crate,
        );
        /**
         Manifest and lock file, with their contents in the stand-in tree and in the real tree.
         */
        const copies = await Promise.all([
          'Cargo.toml',
          'Cargo.lock',
        ].map(async function readBoth(name) {
          return {
            stub: await readFile(
              join(
                stub,
                name,
              ),
              'utf8',
            ),
            real: await readFile(
              join(
                real,
                name,
              ),
              'utf8',
            ),
          };
        }));
        for (const copy of copies)
          expect(copy.stub).toBe(copy.real);
        /**
         Stand-in file of every declared target.
         */
        const standIns = await Promise.all(Object.values(targetPaths).map(function statStandIn(path) {
          return stat(join(
            stub,
            path,
          ));
        }));
        for (const standIn of standIns)
          expect(standIn.isFile()).toBe(true);
        expect(await readFile(
          join(
            stub,
            targetPaths.lib,
          ),
          'utf8',
        )).not.toContain(realContent);
        // The stand-in tree alone must be enough for Cargo to resolve and check every target.
        await runCommand({
          command: 'cargo',
          args: [
            'check',
            '--offline',
            '--locked',
            '--lib',
            '--bins',
            '--manifest-path',
            join(
              stub,
              'Cargo.toml',
            ),
          ],
          capture: true,
        });
      },
    }),
  ],
});

await describe({
  name: testContainerfile.name,
  children: [
    it({
      name: 'compiles the registry dependencies before any real source is copied',
      fn: async () => {
        /**
         Lines of the test stage.
         */
        const lines = lastStage(testContainerfile({
          base: baseImage,
          jobs: 1,
        }));
        /**
         Position of the dependency compile: the first `RUN` that runs nextest.
         */
        const dependencyRun = lines.findIndex(function compilesDependencies(line) {
          return line.startsWith('RUN ') && line.includes('nextest') && line.includes('--no-run')
            && (!line.includes('touch'));
        });
        expect(dependencyRun).toBeGreaterThan(0);
        /**
         Context directory the real sources are copied from.
         */
        const realSources = 'package/';
        for (const line of lines.slice(
          0,
          dependencyRun,
        )) {
          if (instructionOf(line) === 'COPY')
            expect(line).not.toContain(realSources);
        }
        expect(lines.slice(dependencyRun).some(function copiesRealSources(line) {
          return line.startsWith('COPY') && line.includes(realSources);
        })).toBe(true);
      },
    }),
    it({
      name: 'copies the path crates before the tested crate, and compiles after each copy',
      fn: async () => {
        /**
         Lines of the test stage.
         */
        const lines = lastStage(testContainerfile({
          base: baseImage,
          jobs: 1,
        }));
        /**
         Position of the copy of the tested crate.
         */
        const subjectCopy = lines.findIndex(function copiesSubject(line) {
          return line.startsWith('COPY') && line.includes(`package/${subjectCrate} `);
        });
        expect(subjectCopy).toBeGreaterThan(0);
        for (const crate of pathCrates) {
          /**
           Top-level directory that holds the path crate.
           */
          const root = crate.split('/')[0] ?? crate;
          /**
           Position of the copy that brings the path crate.
           */
          const copy = lines.findIndex(function copiesRoot(line) {
            return line.startsWith('COPY') && line.includes(`package/${root} `);
          });
          expect(copy).toBeGreaterThan(0);
          expect(copy).toBeLessThan(subjectCopy);
        }
        // Every source copy is followed by a compile before the next copy, and the last line is a compile.
        // The last line is the empty line that ends the file, so the compile is the one before it.
        const lastCompile = lines.at(-2);
        expect(lastCompile).toContain('touch');
        expect(lastCompile).toContain('--no-run');
        for (const crate of wrapperCrates)
          expect(lines.join('\n')).toContain(crate);
      },
    }),
    it({
      name: 'runs every test after a failure by default, while the compile steps keep the build-only options',
      fn: async () => {
        /**
         Lines of the test stage.
         */
        const lines = lastStage(testContainerfile({
          base: baseImage,
          jobs: 1,
        }));
        /**
         The image's default command.
         */
        const command = lines.find(function isCommand(line) {
          return instructionOf(line) === 'CMD';
        });
        expect(command).toBe(`CMD ${JSON.stringify([...runArguments])}`);
        expect(runArguments).toContain('--no-fail-fast');
        for (const line of lines.filter(function isCompile(candidate) {
          return candidate.includes('--no-run');
        }))
          expect(line).not.toContain('--no-fail-fast');
      },
    }),
  ],
});
