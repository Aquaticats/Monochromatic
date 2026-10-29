/**
 Compiler crashes and unexplained failures must not become successful type checks.

 @module
 */
import { describe, expect, it } from '@monochromatic-dev/test-harness-core/ts';
import { runCompilerFailureCase, type CompilerFailureCase } from './tsc-failure-fixture.ts';

/** Ordinary successful output proves the disposable executable is actually used. */
const success = 'fixture compiler version 1\n';
/** Known generated dependency diagnostics remain eligible for the existing suppression policy. */
const suppressed = 'node_modules/example/index.ts(1,1): error TS2322: dependency fixture\n';
/** This is the observed panic header, emitted without exhausting any actual host resource. */
const panic = 'runtime: failed to create new OS thread (have 140 already; errno=11)\nfatal error: newosproc\n';
/** Non-TypeScript stderr also must preserve its failing process status. */
const unknownFailure = 'fixture compiler cannot read its configuration\n';
/** The configured compiler reports emitted build-info files even when diagnostics were suppressed. */
const emission = 'TSFILE: /fixture/build-info.tsbuildinfo\n';
/** Quoted syntax remains literal compiler output when embedded in the disposable executable. */
const syntaxBoundary = `quote " slash \\ newline\n*/; process.exit(99); \${value}`;

/** Each case selects process output independently of the wrapper implementation. */
const cases: readonly CompilerFailureCase[] = [
  { name: 'preserves syntax delimiters as literal fixture output', stdout: syntaxBoundary, stderr: '', exitCode: 0,
    expectedCode: 0, expectedStdout: syntaxBoundary, expectedStderr: '' },
  { name: 'forwards successful compiler output', stdout: success, stderr: '', exitCode: 0,
    expectedCode: 0, expectedStdout: 'fixture compiler version 1', expectedStderr: '' },
  { name: 'permits only known suppressed dependency errors', stdout: suppressed, stderr: '', exitCode: 2,
    expectedCode: 0, expectedStdout: '', expectedStderr: '' },
  { name: 'permits known suppressed diagnostics on stderr', stdout: '', stderr: suppressed, exitCode: 2,
    expectedCode: 0, expectedStdout: '', expectedStderr: '' },
  { name: 'preserves a Go panic failure', stdout: '', stderr: panic, exitCode: 2,
    expectedCode: 2, expectedStdout: '', expectedStderr: panic },
  { name: 'preserves a silent compiler failure', stdout: '', stderr: '', exitCode: 2,
    expectedCode: 2, expectedStdout: '', expectedStderr: '' },
  { name: 'preserves non-TypeScript diagnostic failures', stdout: '', stderr: unknownFailure, exitCode: 7,
    expectedCode: 7, expectedStdout: '', expectedStderr: unknownFailure },
  { name: 'does not hide a panic beside suppressed diagnostics', stdout: suppressed, stderr: panic, exitCode: 2,
    expectedCode: 2, expectedStdout: '', expectedStderr: panic },
  { name: 'retains emission metadata alongside fully suppressed errors', stdout: suppressed + emission, stderr: '', exitCode: 2,
    expectedCode: 0, expectedStdout: emission, expectedStderr: '' },
  { name: 'requires actual suppressed diagnostics before permitting emission-only failure', stdout: emission, stderr: '', exitCode: 2,
    expectedCode: 2, expectedStdout: emission, expectedStderr: '' },
  { name: 'preserves unexplained stdout beside suppressed errors', stdout: suppressed + unknownFailure, stderr: '', exitCode: 2,
    expectedCode: 2, expectedStdout: unknownFailure, expectedStderr: '' },
  { name: 'rejects emission metadata without a path', stdout: `${suppressed}TSFILE: \n`, stderr: '', exitCode: 2,
    expectedCode: 2, expectedStdout: 'TSFILE: \n', expectedStderr: '' },
];

//region The emitted CLI preserves unsuccessful processes unless every diagnostic was knowingly suppressed

await describe({ name: '', concurrency: 1, children: cases.map(function compilerFailureCase(input) {
  return it({ name: input.name, fn: async function compilerExitBoundary() {
    /** The result belongs to the built wrapper and its actual disposable compiler child. */
    const result = await runCompilerFailureCase(input);
    expect(result.status).toBe(input.expectedCode);
    expect(result.stdout).toBe(input.expectedStdout);
    expect(result.stderr).toBe(input.expectedStderr);
  } });
}) });

//endregion The emitted CLI preserves unsuccessful processes unless every diagnostic was knowingly suppressed
