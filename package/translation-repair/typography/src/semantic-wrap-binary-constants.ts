//region Semantic wrap binary constants
// The fixed words and numbers of the program that wraps shipped text, kept
// apart so the modules that start it, check it and state its failures share
// one writing of each. `semantic-wrap-binary.ts` says how the program is used.

/**
 Name the program is started by, which is also the first word of the line it
 answers `--version` with.
 */
export const LINT_BINARY_NAME = 'monochromatic-lint';

/**
 Version of the program this package is written against; the repository's
 `toolchain.lock.json` records the same one for `"cargo:monochromatic-lint"`
 (the tool plan keeps the floating request for a tool the repository
 publishes), and `semantic-wrap-binary.unit.test.ts` holds the two together.
 */
export const PINNED_LINT_BINARY_VERSION = '0.4.0';

/**
 File name of the configuration the program reads, selecting the one rule.
 */
export const CONFIG_FILE_NAME = 'semantic-wrap.config.jsonc';

/**
 File name the program is told a passage has, which selects Markdown (not
 MDX) and the configuration block that matches it.
 */
export const PASSAGE_FILE_NAME = 'passage.md';

/**
 Bytes granted to one reply, 64 mebibytes: a passage is kilobytes, so this
 bounds a runaway reply generously and still carries a passage a thousand
 times a page.
 */
export const MAX_REPLY_BYTES = 67_108_864;

/**
 Longest version token a program's answer may hold to be repeated.
 */
export const MOST_VERSION_CHARACTERS = 32;

/**
 Lowest UTF-16 code unit of the surrogate range; a code point in the range is
 a surrogate standing alone, because a pair reads as one code point above it.
 */
export const SURROGATE_FIRST = 0xD8_00;

/**
 Highest UTF-16 code unit of the surrogate range.
 */
export const SURROGATE_LAST = 0xDF_FF;

//endregion Semantic wrap binary constants
