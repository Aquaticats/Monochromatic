import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import spawn from 'nano-spawn';
import {
  existsSync,
  mkdirSync,
  rmSync,
  utimesSync,
  writeFileSync,
} from 'node:fs';
import { join, } from 'node:path';

/**
 Runs a shell command and resolves with its captured output.
 
 Wraps `nano-spawn` with `shell: true` to mirror the previous `promisify(exec)`
 contract: resolves `{ stdout, stderr }` on success and rejects with a
 `SubprocessError` (carrying `stdout`, `stderr`) on non-zero exit.
 
 @param command - Full shell command line to execute
 
 @returns Captured `stdout` and `stderr`
 
 @example
 ```ts
 const { stdout } = await execAsync('echo hi'); // stdout === 'hi'
 ```
 */
function execAsync(command: string,): Promise<{ readonly stdout: string; readonly stderr: string; }> {
  return spawn(command, { shell: true, },);
}

//region Fixture Setup: temp directory with controllable source and output files

function setup() {
  const testFileDir = import.meta.dirname;
  const cliPath = join(testFileDir, 'depends.ts',);

  const packageDir = join(testFileDir, '..',);
  const timestamp = Date.now();
  const randomId = Math.random().toString(36,).slice(2, 8,);
  const testDir = join(packageDir, 'dist', 'temp', 'test',
    `cli-depends-${timestamp}-${randomId}`,);
  const srcDir = join(testDir, 'src',);
  const outDir = join(testDir, 'out',);
  const markerPath = join(testDir, 'ran.marker',);

  mkdirSync(srcDir, { recursive: true, },);
  mkdirSync(outDir, { recursive: true, },);

  return { cliPath, testDir, srcDir, outDir, markerPath, };
}

function teardown({ testDir, }: { testDir: string; },) {
  if (existsSync(testDir,))
    rmSync(testDir, { recursive: true, },);
}

//endregion Fixture Setup

//region Helpers

/**
 Options for {@link touch}.
 
 @example
 ```ts
 const options: TouchOptions = {
   path: join(srcDir, 'a.ts'),
   ageMs: 5000,
 };
 ```
 */
type TouchOptions = {
  /** Absolute file path */
  readonly path: string;
  /** How many milliseconds in the past to set mtime (0 = now) */
  readonly ageMs?: number;
};

/**
 Creates a file and optionally sets its mtime to the past.
 
 @param path - Absolute file path
 
 @param ageMs - How many milliseconds in the past to set mtime (0 = now)
 
 @example
 ```ts
 touch({ path: join(srcDir, 'a.ts'), ageMs: 5000 }); // 5 seconds old
 ```
 */
function touch({
  path,
  ageMs = 0,
}: TouchOptions,): void {
  writeFileSync(path, `file: ${path}`,);
  if (ageMs > 0) {
    const past = new Date(Date.now() - ageMs,);
    utimesSync(path, past, past,);
  }
}

/**
 Builds the command string that writes a marker file to prove execution.
 
 Output is collapsed, so we verify execution via side effects (file creation)
 rather than stdout content.
 
 @param markerPath - Absolute path to the marker file
 @returns Shell command string that creates the marker file
 
 @example
 ```ts
 const cmd = writeMarkerCmd(markerPath);
 ```
 */
function writeMarkerCmd(markerPath: string,): string {
  return `node -e "require('fs').writeFileSync('${markerPath}', 'ran')"`;
}

//endregion Helpers

await describe({
  name: '',
  children: [
    describe({
      name: 'task-depends file-based staleness',
      children: [
        it({
          name: 'runs command when no output files exist',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, outDir, markerPath, } = fixtures;

            touch({ path: join(srcDir, 'a.ts',), },);

            await execAsync(
              `node ${cliPath} -s "${srcDir}/**" -o "${outDir}/**" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(true,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'runs command when sources are newer than outputs',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, outDir, markerPath, } = fixtures;

            touch({ path: join(outDir, 'a.js',), ageMs: 5_000, },);
            touch({ path: join(srcDir, 'a.ts',), },);

            await execAsync(
              `node ${cliPath} -s "${srcDir}/**" -o "${outDir}/**" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(true,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'skips command when outputs are newer than sources',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, outDir, markerPath, } = fixtures;

            touch({ path: join(srcDir, 'a.ts',), ageMs: 5_000, },);
            touch({ path: join(outDir, 'a.js',), },);

            await execAsync(
              `node ${cliPath} -s "${srcDir}/**" -o "${outDir}/**" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(false,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'verbose flag logs staleness reason',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, outDir, markerPath, } = fixtures;

            touch({ path: join(srcDir, 'a.ts',), },);

            const { stderr, } = await execAsync(
              `node ${cliPath} -v -s "${srcDir}/**" -o "${outDir}/**" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(stderr,).toContain('[task-depends]',);
            expect(stderr,).toContain('stale',);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'verbose flag logs up-to-date message when skipping',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, outDir, markerPath, } = fixtures;

            touch({ path: join(srcDir, 'a.ts',), ageMs: 5_000, },);
            touch({ path: join(outDir, 'a.js',), },);

            const { stderr, } = await execAsync(
              `node ${cliPath} -v -s "${srcDir}/**" -o "${outDir}/**" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(stderr,).toContain('fresh',);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'accepts multiple --sources globs',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, testDir, srcDir, outDir, markerPath, } = fixtures;

            const libDir = join(testDir, 'lib',);
            mkdirSync(libDir, { recursive: true, },);
            touch({ path: join(srcDir, 'a.ts',), },);
            touch({ path: join(libDir, 'b.ts',), },);

            await execAsync(
              `node ${cliPath} -s "${srcDir}/**" -s "${libDir}/**" -o "${outDir}/**" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(true,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'accepts multiple --outputs globs',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, testDir, srcDir, outDir, markerPath, } = fixtures;

            const dist2Dir = join(testDir, 'dist2',);
            mkdirSync(dist2Dir, { recursive: true, },);
            touch({ path: join(srcDir, 'a.ts',), ageMs: 5_000, },);
            touch({ path: join(outDir, 'a.js',), },);
            touch({ path: join(dist2Dir, 'b.js',), },);

            await execAsync(
              `node ${cliPath} -s "${srcDir}/**" -o "${outDir}/**" -o "${dist2Dir}/**" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(false,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'runs when source files exist but no output files match',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, outDir, markerPath, } = fixtures;

            touch({ path: join(srcDir, 'a.ts',), },);

            await execAsync(
              `node ${cliPath} -s "${srcDir}/**" -o "${outDir}/**" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(true,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'skips when source glob matches nothing and output files exist',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, outDir, markerPath, } = fixtures;

            // Empty source glob → no timestamps → sourceTime = -Infinity → fresh
            // Use srcDir (exists) with a non-matching extension to avoid ENOENT on missing dirs
            touch({ path: join(outDir, 'a.js',), },);

            await execAsync(
              `node ${cliPath} -s "${srcDir}/**/*.xyz" -o "${outDir}/**" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(false,);

            teardown(fixtures,);
          },
        },),
      ],
    },),
    describe({
      name: 'task-depends sh: output with Infinity/-Infinity',
      children: [
        it({
          name: 'runs command when sh: output returns -Infinity (missing)',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, markerPath, } = fixtures;

            await execAsync(
              `node ${cliPath} -s "sh:echo Infinity" -o "sh:echo -Infinity" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(true,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'skips command when sh: output returns Infinity (exists)',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, markerPath, } = fixtures;

            await execAsync(
              `node ${cliPath} -s "sh:echo -Infinity" -o "sh:echo Infinity" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(false,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'newest strategy hides -Infinity when mixed with Infinity',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, markerPath, } = fixtures;

            // With default newest: Math.max(Infinity, -Infinity) = Infinity → fresh
            await execAsync(
              `node ${cliPath} -s "sh:echo -Infinity" -o "sh:echo Infinity" -o "sh:echo -Infinity" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(false,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'oldest strategy catches -Infinity in mixed outputs',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, markerPath, } = fixtures;

            // With oldest: Math.min(Infinity, -Infinity) = -Infinity → stale
            await execAsync(
              `node ${cliPath} -s "sh:echo 0" --output-time-strategy oldest -o "sh:echo Infinity" -o "sh:echo -Infinity" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(true,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'skips when all sh: outputs return Infinity',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, markerPath, } = fixtures;

            await execAsync(
              `node ${cliPath} -s "sh:echo -Infinity" -o "sh:echo Infinity" -o "sh:echo Infinity" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(false,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'verbose flag logs resolved timestamp',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, markerPath, } = fixtures;

            const { stderr, } = await execAsync(
              `node ${cliPath} -v -s "sh:echo Infinity" -o "sh:echo -Infinity" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(stderr,).toContain('Infinity',);
            expect(stderr,).toContain('-Infinity',);

            teardown(fixtures,);
          },
        },),
      ],
    },),
    describe({
      name: 'task-depends sh: source with Infinity/-Infinity',
      children: [
        it({
          name: 'runs when sh: source returns Infinity (dirty)',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, outDir, markerPath, } = fixtures;

            touch({ path: join(outDir, 'a.js',), },);

            await execAsync(
              `node ${cliPath} -s "sh:echo Infinity" -o "${outDir}/**" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(true,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'skips when sh: source returns -Infinity (clean)',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, outDir, markerPath, } = fixtures;

            touch({ path: join(outDir, 'a.js',), },);

            await execAsync(
              `node ${cliPath} -s "sh:echo -Infinity" -o "${outDir}/**" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(false,);

            teardown(fixtures,);
          },
        },),
      ],
    },),
    describe({
      name: 'task-depends sh: timestamp output',
      children: [
        it({
          name: 'uses unix epoch seconds from sh: command stdout',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, outDir, markerPath, } = fixtures;

            // Source returns a timestamp far in the future (year 2040)
            // Output files are from now; source is newer → stale
            touch({ path: join(outDir, 'a.js',), },);

            await execAsync(
              `node ${cliPath} -s "sh:echo 2208988800" -o "${outDir}/**" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(true,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'uses unix epoch milliseconds from sh: command stdout',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, outDir, markerPath, } = fixtures;

            // Source returns a timestamp far in the future (year 2040, in ms)
            touch({ path: join(outDir, 'a.js',), },);

            await execAsync(
              `node ${cliPath} -s "sh:echo 2208988800000" -o "${outDir}/**" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(true,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'uses ISO 8601 date from sh: command stdout',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, outDir, markerPath, } = fixtures;

            // Source returns a date far in the future
            touch({ path: join(outDir, 'a.js',), },);

            await execAsync(
              `node ${cliPath} -s "sh:echo 2040-01-01T00:00:00Z" -o "${outDir}/**" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(true,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'skips when sh: timestamp is older than outputs',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, outDir, markerPath, } = fixtures;

            // Source returns a very old timestamp (year 2000)
            touch({ path: join(outDir, 'a.js',), },);

            await execAsync(
              `node ${cliPath} -s "sh:echo 946684800" -o "${outDir}/**" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(false,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'uses timestamp from sh: output command',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, markerPath, } = fixtures;

            // Output returns a very old timestamp → older than sources → stale
            touch({ path: join(srcDir, 'a.ts',), },);

            await execAsync(
              `node ${cliPath} -s "${srcDir}/**" -o "sh:echo 946684800" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(true,);

            teardown(fixtures,);
          },
        },),
      ],
    },),
    describe({
      name: 'task-depends sh: command errors',
      children: [
        it({
          name: 'throws when sh: command fails with non-zero exit',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, outDir, markerPath, } = fixtures;

            await expect(
              execAsync(
                `node ${cliPath} -s "sh:exit 1" -o "${outDir}/**" -- ${
                  writeMarkerCmd(markerPath,)
                }`,
              ),
            )
              .rejects
              .toThrow();

            teardown(fixtures,);
          },
        },),
        it({
          name: 'throws when sh: command returns unparseable output',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, markerPath, } = fixtures;

            await expect(
              execAsync(
                `node ${cliPath} -o "sh:echo hello" -- ${writeMarkerCmd(markerPath,)}`,
              ),
            )
              .rejects
              .toThrow();

            teardown(fixtures,);
          },
        },),
        it({
          name: 'throws when sh: command returns empty output',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, markerPath, } = fixtures;

            await expect(
              execAsync(
                `node ${cliPath} -o "sh:echo" -- ${writeMarkerCmd(markerPath,)}`,
              ),
            )
              .rejects
              .toThrow();

            teardown(fixtures,);
          },
        },),
      ],
    },),
    describe({
      name: 'task-depends time strategies',
      children: [
        it({
          name: 'source-time-strategy oldest uses minimum source timestamp',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, outDir, markerPath, } = fixtures;

            // Two sources: one very old (epoch), one very new (now)
            // With oldest: min picks the old one → old > new output → false → fresh
            touch({ path: join(outDir, 'a.js',), },);

            await execAsync(
              `node ${cliPath} --source-time-strategy oldest -s "sh:echo 946684800" -s "sh:echo 2208988800" -o "${outDir}/**" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(false,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'source-time-strategy newest uses maximum source timestamp',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, outDir, markerPath, } = fixtures;

            // Same sources as above, but with newest: max picks the new one → new > output → stale
            touch({ path: join(outDir, 'a.js',), },);

            await execAsync(
              `node ${cliPath} --source-time-strategy newest -s "sh:echo 946684800" -s "sh:echo 2208988800" -o "${outDir}/**" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(true,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'output-time-strategy oldest catches missing output in mixed list',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, outDir, markerPath, } = fixtures;

            // File output exists (fresh) + sh: output is -Infinity (missing)
            // With oldest: min picks -Infinity → source > -Infinity → stale
            touch({ path: join(srcDir, 'a.ts',), },);
            touch({ path: join(outDir, 'a.js',), },);

            await execAsync(
              `node ${cliPath} --output-time-strategy oldest -s "${srcDir}/**" -o "${outDir}/**" -o "sh:echo -Infinity" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(true,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'output-time-strategy newest hides missing output in mixed list',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, outDir, markerPath, } = fixtures;

            // Same setup but with newest: max picks the real mtime → source (old) < output → fresh
            touch({ path: join(srcDir, 'a.ts',), ageMs: 5_000, },);
            touch({ path: join(outDir, 'a.js',), },);

            await execAsync(
              `node ${cliPath} --output-time-strategy newest -s "${srcDir}/**" -o "${outDir}/**" -o "sh:echo -Infinity" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(false,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'rejects invalid strategy value',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, outDir, } = fixtures;

            await expect(
              execAsync(
                `node ${cliPath} --source-time-strategy invalid -s "${srcDir}/**" -o "${outDir}/**" -- echo test`,
              ),
            )
              .rejects
              .toThrow();

            teardown(fixtures,);
          },
        },),
        it({
          name: 'sh: strategy receives timestamps via stdin and returns aggregated value',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, outDir, markerPath, } = fixtures;

            // Custom strategy using sort -n | head -1 (minimum, like oldest)
            // Two sources: year 2000 (946684800s) and year 2040 (2208988800s)
            // Strategy receives parsed ms values, picks minimum (year 2000) → older than output → fresh
            touch({ path: join(outDir, 'a.js',), },);

            await execAsync(
              `node ${cliPath} --source-time-strategy "sh:sort -n | head -1" -s "sh:echo 946684800" -s "sh:echo 2208988800" -o "${outDir}/**" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(false,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'sh: strategy can use sort -rn for maximum (newest)',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, outDir, markerPath, } = fixtures;

            // Custom strategy using sort -rn | head -1 (maximum, like newest)
            // Max picks year 2040 → newer than output → stale
            touch({ path: join(outDir, 'a.js',), },);

            await execAsync(
              `node ${cliPath} --source-time-strategy "sh:sort -rn | head -1" -s "sh:echo 946684800" -s "sh:echo 2208988800" -o "${outDir}/**" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(true,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'accepts sh: prefix for output-time-strategy',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, markerPath, } = fixtures;

            // Custom output strategy: sort -n picks minimum → -Infinity dominates → stale
            touch({ path: join(srcDir, 'a.ts',), },);

            await execAsync(
              `node ${cliPath} --output-time-strategy "sh:sort -n | head -1" -s "${srcDir}/**" -o "sh:echo Infinity" -o "sh:echo -Infinity" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(true,);

            teardown(fixtures,);
          },
        },),
      ],
    },),
    describe({
      name: 'task-depends no sources behavior',
      children: [
        it({
          name: 'skips when no -s flags and output returns Infinity',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, markerPath, } = fixtures;

            // No sources → sourceTime = -Infinity → -Infinity > Infinity → false → fresh
            await execAsync(
              `node ${cliPath} -o "sh:echo Infinity" -- ${writeMarkerCmd(markerPath,)}`,
            );

            expect(existsSync(markerPath,),).toBe(false,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'skips when no -s flags and output returns -Infinity',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, markerPath, } = fixtures;

            // No sources → sourceTime = -Infinity → -Infinity > -Infinity → false → fresh
            await execAsync(
              `node ${cliPath} -o "sh:echo -Infinity" -- ${writeMarkerCmd(markerPath,)}`,
            );

            expect(existsSync(markerPath,),).toBe(false,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'runs with explicit Infinity source and -Infinity output',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, markerPath, } = fixtures;

            // Explicit "always dirty" source pattern for gate checks
            await execAsync(
              `node ${cliPath} -s "sh:echo Infinity" -o "sh:echo -Infinity" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(true,);

            teardown(fixtures,);
          },
        },),
      ],
    },),
    describe({
      name: 'task-depends combined file and sh: checks',
      children: [
        it({
          name: 'runs when file sources are newer and sh: output is Infinity',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, markerPath, } = fixtures;

            // Source is now, output sh: returns Infinity → source_mtime > Infinity → false → fresh
            // This is correct: Infinity means "output exists and is infinitely fresh"
            touch({ path: join(srcDir, 'a.ts',), },);

            await execAsync(
              `node ${cliPath} -s "${srcDir}/**" -o "sh:echo Infinity" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(false,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'runs when file sources are newer and sh: output returns old timestamp',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, markerPath, } = fixtures;

            touch({ path: join(srcDir, 'a.ts',), },);

            await execAsync(
              `node ${cliPath} -s "${srcDir}/**" -o "sh:echo 946684800" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(true,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'skips when both file and sh: checks are fresh',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, outDir, markerPath, } = fixtures;

            touch({ path: join(srcDir, 'a.ts',), ageMs: 5_000, },);
            touch({ path: join(outDir, 'a.js',), },);

            await execAsync(
              `node ${cliPath} -s "${srcDir}/**" -o "${outDir}/**" -o "sh:echo Infinity" -- ${
                writeMarkerCmd(markerPath,)
              }`,
            );

            expect(existsSync(markerPath,),).toBe(false,);

            teardown(fixtures,);
          },
        },),
      ],
    },),
    describe({
      name: 'task-depends output collapsing',
      children: [
        it({
          name: 'hides stdout on successful command execution',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, outDir, } = fixtures;

            touch({ path: join(srcDir, 'a.ts',), },);

            const { stdout, } = await execAsync(
              `node ${cliPath} -s "${srcDir}/**" -o "${outDir}/**" -- echo HIDDEN_OUTPUT`,
            );

            expect(stdout,).not.toContain('HIDDEN_OUTPUT',);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'shows stdout on failed command execution',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, outDir, } = fixtures;

            touch({ path: join(srcDir, 'a.ts',), },);

            try {
              await execAsync(
                `node ${cliPath} -s "${srcDir}/**" -o "${outDir}/**" -- node -e "console.log('VISIBLE_OUTPUT'); process.exit(1)"`,
              );
              expect(true,).toBe(false,);
            }
            catch (error) {
              const execError = error as { stdout: string; stderr: string; };
              expect(execError.stdout,).toContain('VISIBLE_OUTPUT',);
            }

            teardown(fixtures,);
          },
        },),
        it({
          name: 'shows stderr on failed command execution',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, outDir, } = fixtures;

            touch({ path: join(srcDir, 'a.ts',), },);

            try {
              await execAsync(
                `node ${cliPath} -s "${srcDir}/**" -o "${outDir}/**" -- node -e "console.error('ERROR_OUTPUT'); process.exit(1)"`,
              );
              expect(true,).toBe(false,);
            }
            catch (error) {
              const execError = error as { stdout: string; stderr: string; };
              expect(execError.stderr,).toContain('ERROR_OUTPUT',);
            }

            teardown(fixtures,);
          },
        },),
      ],
    },),
    describe({
      name: 'task-depends error handling',
      children: [
        it({
          name: 'propagates command failure exit code',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, outDir, } = fixtures;

            touch({ path: join(srcDir, 'a.ts',), },);

            await expect(
              execAsync(
                `node ${cliPath} -s "${srcDir}/**" -o "${outDir}/**" -- node -e "process.exit(42)"`,
              ),
            )
              .rejects
              .toThrow();

            teardown(fixtures,);
          },
        },),
        it({
          name: 'allowFailure flag suppresses command failure',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, outDir, } = fixtures;

            touch({ path: join(srcDir, 'a.ts',), },);

            const result = await execAsync(
              `node ${cliPath} -a -s "${srcDir}/**" -o "${outDir}/**" -- node -e "process.exit(1)"`,
            );

            expect(result,).toBeDefined();

            teardown(fixtures,);
          },
        },),
        it({
          name: 'fails when no -o provided',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, } = fixtures;

            await expect(
              execAsync(`node ${cliPath} -s "${srcDir}/**" -- echo test`,),
            )
              .rejects
              .toThrow();

            teardown(fixtures,);
          },
        },),
        it({
          name: 'fails when no command provided',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, srcDir, outDir, } = fixtures;

            await expect(
              execAsync(`node ${cliPath} -s "${srcDir}/**" -o "${outDir}/**"`,),
            )
              .rejects
              .toThrow();

            teardown(fixtures,);
          },
        },),
      ],
    },),
  ],
},);
