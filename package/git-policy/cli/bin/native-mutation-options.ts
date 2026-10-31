/**
 Command-line and environment options of the native mutation campaign.

 The parsing and its checks live in the shared cargo-mutants package; this module supplies what is the native wrapper's
 own: its source directory, its default and largest timeout, and the environment variable that names a test to skip.
 */
import {
  baselineSkipArguments as sharedBaselineSkipArguments,
  parseFileScopes,
  parseTimeoutAndExamine,
  type TimeoutAndExamine,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

/**
 Directory every mutation scope must name.
 */
const nativeSourcePrefix = 'src/native/';

/**
 Extension every mutation scope must name.
 */
const rustSourceSuffix = '.rs';

/**
 Test timeout, in seconds, that cargo-mutants applies to each mutant when no `--timeout` option is given.
 Raised from 90 so a loaded host does not turn slow mutants into timeouts; a `--timeout` option overrides it.
 */
const defaultTimeoutSeconds = 300;

/**
 Largest test timeout, in seconds, a `--timeout` option may request.
 */
const maximumTimeoutSeconds = 3_600;

/**
 Environment variable that names one test path to skip in the baseline and every mutant run.
 */
const baselineSkipVariable = 'NATIVE_MUTATION_BASELINE_SKIP';

/**
 Split `--timeout <seconds>` and the repeated `--examine <regex>` from the repeated `--file` scopes; the timeout
 defaults to 300 seconds and the last `--timeout` wins.

 @param options - command-line arguments after the script path

 @returns timeout, the mutant-name patterns of
   the repeated `--examine <regex>` options (only matching mutants are tried), and the arguments left for `parseScopes`

 @example
 ```ts
 const { timeoutSeconds, examine, scopeOptions } = parseTimeout(['--timeout', '60']);
 ```
 */
export function parseTimeout(options: readonly string[]): TimeoutAndExamine {
  return parseTimeoutAndExamine({
    options,
    rules: {
      defaultTimeoutSeconds,
      maximumTimeoutSeconds,
    },
  });
}

/**
 Accept only repeated `--file <glob>` scopes; anything else is a usage error.

 @param options - command-line arguments after the script path

 @returns scopes in command-line order, forwarded to cargo-mutants unchanged

 @example
 ```ts
 const scopes = parseScopes(['--file', 'src/native/lock.rs']);
 ```
 */
export function parseScopes(options: readonly string[]): string[] {
  return parseFileScopes({
    options,
    rules: {
      prefix: nativeSourcePrefix,
      suffix: rustSourceSuffix,
    },
  });
}

/**
 Test-path arguments that skip one named test in the baseline and every mutant run.

 Opt-in through `NATIVE_MUTATION_BASELINE_SKIP`, which names one test path such as `module::tests::name`.
 Unset, the campaign runs every test.

 @returns `['--', '--', '--skip', name]` or no arguments

 @example
 ```ts
 const skip = baselineSkipArguments();
 ```
 */
export function baselineSkipArguments(): string[] {
  /**
   The test path the environment names, if any.
   */
  const name = process.env[baselineSkipVariable];
  return sharedBaselineSkipArguments({
    variable: baselineSkipVariable,
    ...(name === undefined ? {} : { value: name }),
  });
}
