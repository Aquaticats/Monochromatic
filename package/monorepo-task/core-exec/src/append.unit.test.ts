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
  unlinkSync,
  writeFileSync,
} from 'node:fs';
import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

/**
 Runs a shell command and resolves with its captured output.
 
 Wraps `nano-spawn` with `shell: true` to mirror the previous `promisify(exec)`
 contract: resolves `{ stdout, stderr }` on success and rejects with a
 `SubprocessError` on non-zero exit.
 
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

//region Fixture Setup: per-test fixtures

function setup() {
  const testFileDir = import.meta.dirname;
  const cliPath = join(testFileDir, 'append.ts',);

  const packageDir = join(testFileDir, '..',);
  const timestamp = Date.now();
  const randomId = Math.random().toString(36,).slice(2, 8,);
  const testDir = join(packageDir, 'dist', 'temp', 'test',
    `cli-append-${timestamp}-${randomId}`,);

  if (!existsSync(testDir,))
    mkdirSync(testDir, { recursive: true, },);

  const testFile = join(testDir, 'test.txt',);
  writeFileSync(testFile, 'Initial content\n',);

  return { cliPath, testDir, testFile, };
}

function teardown({ testFile, testDir, }: { testFile: string; testDir: string; },) {
  if (existsSync(testFile,))
    unlinkSync(testFile,);

  if (existsSync(testDir,))
    rmSync(testDir, { recursive: true, },);
}

//endregion Fixture Setup

await describe({
  name: 'task-append',
  children: [
    it({
      name: 'appends single line to existing file',
      fn: async () => {
        const fixtures = setup();
        const { cliPath, testFile, } = fixtures;

        const { stdout, stderr, } = await execAsync(
          `node ${cliPath} "new line" --to ${testFile}`,
        );

        expect(stderr,).toBe('',);

        const content = await readFile(testFile, 'utf8',);
        expect(content,).toBe('Initial content\nnew line\n',);

        teardown(fixtures,);
      },
    },),
    it({
      name: 'appends multiple lines as separate arguments',
      fn: async () => {
        const fixtures = setup();
        const { cliPath, testFile, } = fixtures;

        const { stdout, stderr, } = await execAsync(
          `node ${cliPath} "line 1" "line 2" "line 3" --to ${testFile}`,
        );

        expect(stderr,).toBe('',);

        const content = await readFile(testFile, 'utf8',);
        expect(content,).toBe('Initial content\nline 1\nline 2\nline 3\n',);

        teardown(fixtures,);
      },
    },),
    it({
      name: 'appends multiline text with newline characters',
      fn: async () => {
        const fixtures = setup();
        const { cliPath, testFile, } = fixtures;

        const { stdout, stderr, } = await execAsync(
          `node ${cliPath} "line 1\\nline 2" --to ${testFile}`,
        );

        expect(stderr,).toBe('',);

        const content = await readFile(testFile, 'utf8',);
        expect(content,).toBe('Initial content\nline 1\\nline 2\n',);

        teardown(fixtures,);
      },
    },),
    it({
      name: 'uses short flag -t for target file',
      fn: async () => {
        const fixtures = setup();
        const { cliPath, testFile, } = fixtures;

        const { stdout, stderr, } = await execAsync(
          `node ${cliPath} "short flag test" -t ${testFile}`,
        );

        expect(stderr,).toBe('',);

        const content = await readFile(testFile, 'utf8',);
        expect(content,).toBe('Initial content\nshort flag test\n',);

        teardown(fixtures,);
      },
    },),
    it({
      name: 'fails when no text is provided',
      fn: async () => {
        const fixtures = setup();
        const { cliPath, testFile, } = fixtures;

        await expect(execAsync(`node ${cliPath} --to ${testFile}`,),).rejects.toThrow();

        teardown(fixtures,);
      },
    },),
    it({
      name: 'fails when no target file is specified',
      fn: async () => {
        const fixtures = setup();
        const { cliPath, } = fixtures;

        await expect(execAsync(`node ${cliPath} "some text"`,),).rejects.toThrow();

        teardown(fixtures,);
      },
    },),
    it({
      name: 'fails when target file does not exist',
      fn: async () => {
        const fixtures = setup();
        const { cliPath, testDir, } = fixtures;

        const nonExistentFile = join(testDir, 'non-existent.txt',);
        await expect(execAsync(`node ${cliPath} "text" --to ${nonExistentFile}`,),)
          .rejects
          .toThrow();

        teardown(fixtures,);
      },
    },),
    it({
      name: 'fails when file has no write permissions',
      skip: process.platform === 'win32',
      fn: async () => {
        const fixtures = setup();
        const { cliPath, testFile, } = fixtures;

        // Make file read-only
        writeFileSync(testFile, 'read only content\n', { mode: 0o444, },);

        // Bun's appendFile may bypass POSIX permissions in some environments; skip when that happens
        try {
          const { appendFile: nodeAppendFile, } = await import('node:fs/promises');
          await nodeAppendFile(testFile, 'probe',);
          // If we get here, permissions aren't enforced; skip assertion
          teardown(fixtures,);
          return;
        }
        catch (error) {
          expect(error,).toBeInstanceOf(Error,);
          // Permissions enforced, proceed with test
        }

        await expect(execAsync(`node ${cliPath} "text" --to ${testFile}`,),)
          .rejects
          .toThrow();

        teardown(fixtures,);
      },
    },),
    it({
      name: 'preserves existing file content',
      fn: async () => {
        const fixtures = setup();
        const { cliPath, testFile, } = fixtures;

        // Add some initial content
        writeFileSync(testFile, 'Line 1\nLine 2\n',);

        await execAsync(`node ${cliPath} "Line 3" --to ${testFile}`,);

        const content = await readFile(testFile, 'utf8',);
        expect(content,).toBe('Line 1\nLine 2\nLine 3\n',);

        teardown(fixtures,);
      },
    },),
    it({
      name: 'handles empty string as valid input',
      fn: async () => {
        const fixtures = setup();
        const { cliPath, testFile, } = fixtures;

        await execAsync(`node ${cliPath} "" --to ${testFile}`,);

        const content = await readFile(testFile, 'utf8',);
        expect(content,).toBe('Initial content\n\n',);

        teardown(fixtures,);
      },
    },),
    it({
      name: 'handles special characters in text',
      fn: async () => {
        const fixtures = setup();
        const { cliPath, testFile, } = fixtures;

        // Use single quotes to prevent shell expansion of $USER and other special chars
        const specialText = 'Hello && echo test | cat';
        await execAsync(`node ${cliPath} '${specialText}' --to ${testFile}`,);

        const content = await readFile(testFile, 'utf8',);
        expect(content,).toBe(`Initial content\n${specialText}\n`,);

        teardown(fixtures,);
      },
    },),
    it({
      name: 'appends multiple times to the same file',
      fn: async () => {
        const fixtures = setup();
        const { cliPath, testFile, } = fixtures;

        await execAsync(`node ${cliPath} "First append" --to ${testFile}`,);
        await execAsync(`node ${cliPath} "Second append" --to ${testFile}`,);
        await execAsync(`node ${cliPath} "Third append" --to ${testFile}`,);

        const content = await readFile(testFile, 'utf8',);
        expect(content,).toBe(
          'Initial content\nFirst append\nSecond append\nThird append\n',
        );

        teardown(fixtures,);
      },
    },),
  ],
},);
