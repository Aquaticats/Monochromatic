/**
 Command-line and environment options of the native mutation campaign.

 Every value is checked with a linear scan over its characters, so a malformed option fails before any container starts.
 */
// The bin scripts sit outside the package tsconfig's include list, so type-aware lint checks them in an inferred
// program that loads no ambient Node types; this reference loads them, so `process` stays typed in this file.
/// <reference types="node" />
import { NativeVerificationError } from './native-verification-process.mjs';

/** Directory every mutation scope must name. */
const nativeSourcePrefix = 'src/native/';
/** Extension every mutation scope must name. */
const rustSourceSuffix = '.rs';
/** Characters a mutation scope's file-name glob may contain. */
const scopeNameCharacters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_*?.-';
/** Characters of a `--examine` pattern: names, alternation, grouping and the wildcards of the mutant names. */
const examineCharacters = String.raw`ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_|:.*?()\ -`;
/** Decimal digits. */
const decimalDigits = '0123456789';
/** Lowercase letters and the underscore: the characters a Rust test-path segment may start with. */
const segmentStartCharacters = 'abcdefghijklmnopqrstuvwxyz_';
/** Test timeout, in seconds, that cargo-mutants applies to each mutant when no `--timeout` option is given. */
// Raised from 90 so a loaded host does not turn slow mutants into timeouts; a `--timeout` option overrides it.
const defaultTimeoutSeconds = 300;
/** Largest test timeout, in seconds, a `--timeout` option may request. */
const maximumTimeoutSeconds = 3_600;

/**
 Check that every character of a text is one of the allowed characters.

 @param {{ text: string, allowed: string }} request - the value under test and the characters it may use
 @returns {boolean} true when the text is non-empty and uses only allowed characters
 */
function usesOnly({
  text,
  allowed
}) {
  if (text.length === 0)
    return false;
  for (const character of text) {
    if (!allowed.includes(character))
      return false;
  }
  return true;
}

/**
 Check that a mutation scope is a `src/native/*.rs` file glob with one linear scan.

 @param {string} scope - `--file` value from the command line
 @returns {boolean} true when the name between directory and extension is non-empty and uses only glob-safe characters
 */
function isNativeSourceGlob(scope) {
  if ((!scope.startsWith(nativeSourcePrefix)) || (!scope.endsWith(rustSourceSuffix)))
    return false;
  return usesOnly({
    text: scope.slice(
      nativeSourcePrefix.length,
      scope.length - rustSourceSuffix.length,
    ),
    allowed: scopeNameCharacters,
  });
}

/**
 Check for a whole number of seconds written without a sign, a leading zero or a fraction.

 @param {string} text - `--timeout` value
 @returns {boolean} true for `1`, `30`, `3600`; false for `0`, `007`, `1.5` and the empty text
 */
function isPositiveWholeNumber(text) {
  return usesOnly({
    text,
    allowed: decimalDigits,
  }) && (!text.startsWith('0'));
}

/**
 Check for a Rust test path: two or more `::`-separated segments, each starting with a lowercase letter or underscore.

 @param {string} text - `NATIVE_MUTATION_BASELINE_SKIP` value
 @returns {boolean} true for `module::tests::name`
 */
function isTestPath(text) {
  const segments = text.split('::');
  return (segments.length >= 2) && segments.every(function isSegment(segment) {
    return usesOnly({
      text: segment,
      allowed: segmentStartCharacters + decimalDigits,
    }) && segmentStartCharacters.includes(segment[0] ?? '');
  });
}

/**
 Read one `--timeout` value.

 @param {string | undefined} value - the text after the flag
 @returns {number} the whole number of seconds it names
 */
function timeoutOf(value) {
  if ((value === undefined) || (!isPositiveWholeNumber(value))
    || (Number(value) > maximumTimeoutSeconds))
    throw new NativeVerificationError(`--timeout must be a whole number of seconds from 1 to ${maximumTimeoutSeconds}, got ${value}.`);
  return Number(value);
}

/**
 Split `--timeout <seconds>` and the repeated `--examine <regex>` from the repeated `--file` scopes; the timeout
 defaults to 300 seconds and the last `--timeout` wins.

 @param {readonly string[]} options - command-line arguments after the script path
 @returns {{ timeoutSeconds: number, examine: string[], scopeOptions: string[] }} timeout, the mutant-name patterns of
   the repeated `--examine <regex>` options (only matching mutants are tried), and the arguments left for `parseScopes`
 */
export function parseTimeout(options) {
  /** @type {number[]} */
  const timeouts = [];
  /** @type {string[]} */
  const scopeOptions = [];
  /** @type {string[]} */
  const examine = [];
  for (let index = 0; index < options.length; index += 2) {
    const flag = options[index];
    const value = options[index + 1];
    if (flag === '--examine') {
      if ((value === undefined) || (!usesOnly({
        text: value,
        allowed: examineCharacters,
      })))
        throw new NativeVerificationError(`--examine must be a regular expression of plain characters, got ${value}.`);
      examine.push(value);
    } else if (flag === '--timeout')
      timeouts.push(timeoutOf(value));
    else
      scopeOptions.push(
        flag ?? '',
        value ?? ''
      );
  }
  return {
    timeoutSeconds: timeouts.at(-1) ?? defaultTimeoutSeconds,
    examine,
    scopeOptions
  };
}

/**
 Accept only repeated `--file <glob>` scopes; anything else is a usage error.

 @param {readonly string[]} options - command-line arguments after the script path
 @returns {string[]} scopes in command-line order, forwarded to cargo-mutants unchanged
 */
export function parseScopes(options) {
  /** @type {string[]} */
  const scopes = [];
  for (let index = 0; index < options.length; index += 2) {
    const flag = options[index];
    const scope = options[index + 1];
    if ((flag !== '--file') || (scope === undefined))
      throw new NativeVerificationError('Only repeated --file <glob> options are accepted.');
    if (!isNativeSourceGlob(scope))
      throw new NativeVerificationError(`--file must be a src/native/*.rs glob, got ${scope}.`);
    scopes.push(scope);
  }
  return scopes;
}

/**
 Test-path arguments that skip one named test in the baseline and every mutant run.

 Opt-in through `NATIVE_MUTATION_BASELINE_SKIP`, which names one test path such as `module::tests::name`.
 Unset, the campaign runs every test.

 @returns {string[]} `['--', '--', '--skip', name]` or no arguments
 */
export function baselineSkipArguments() {
  const name = process.env
    .NATIVE_MUTATION_BASELINE_SKIP;
  if ((name === undefined) || (name === ''))
    return [];
  if (!isTestPath(name))
    throw new NativeVerificationError(`NATIVE_MUTATION_BASELINE_SKIP must be a test path, got ${name}.`);
  return [
    '--',
    '--',
    '--skip',
    name
  ];
}
