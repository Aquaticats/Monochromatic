/**
 Pure tsc output classification and filtering for the `task-tsc` wrapper.
 
 Kept apart from the `tsc-filter.ts` command-line entry so importers such as
 `testing.ts` never pull the entry into a shared bundle chunk,
 where its `import.meta.main` guard would be false and `tsc` would never run.
 
 @module
 */

//region Diagnostic line detection

/**
 Literal text that opens the error code on every tsc diagnostic line.
 */
const ERROR_CODE_TOKEN = '): error TS';

/**
 Walks the run of ASCII digits in `s` starting at `from`.
 
 @param s - input string
 
 @param from - cursor into `s`
 
 @returns exclusive end of the digit run
 */
function endOfDigitRun({
  s,
  from,
}: {
  readonly s: string;
  readonly from: number;
},): number {
  return (function walk(): number {
    /**
     Cursor advanced across the ASCII digit run; stops at the first non-digit or the end of `s`.
     */
    let idx = from;
    while (idx < s
      .length) {
      /**
       Char at the cursor; only ASCII digits advance the run.
       */
      const c = s.charAt(idx,);
      if ((c < '0') || (c > '9'))
        break;
      idx += 1;
    }
    return idx;
  })();
}

/**
 Tests whether a line is a tsc diagnostic line.
 
 Mirrors `/\(\d+,\d+\): error TS\d+:/` with a linear `indexOf` walk:
 locate `): error TS`, then require digit runs (via {@link endOfDigitRun}
 for the trailing run) flanking the surrounding `(<digits>,<digits>)`
 prefix and a trailing `<digits>:`.
 
 @param line - single line of tsc output
 
 @returns true when the line matches the diagnostic format
 
 @example
 ```ts
 isDiagnosticLine('src/index.ts(1,1): error TS2304: Cannot find name.');
 // true
 isDiagnosticLine('  Type "string" is not assignable to type "number".');
 // false
 ```
 
 @internal
 */
export function isDiagnosticLine(line: string,): boolean {
  /**
   Walks digits backwards from `pos - 1` to locate the inclusive start of
   a run, in a single linear pass.
   
   @param pos - cursor (one past the last digit so far)
   
   @returns inclusive start of the digit run (clamped at 0)
   */
  function startOfDigitsBackwards(pos: number,): number {
    return (function walk(): number {
      if (pos <= 0)
        return 0;
      /**
       Cursor walked left across the ASCII digit run; the loop guard keeps it at or above 0.
       */
      let p = pos;
      while (p > 0) {
        /**
         Char just left of the cursor; a non-digit ends the back-walk.
         */
        const c = line.charAt(p - 1,);
        if ((c < '0') || (c > '9'))
          break;
        p -= 1;
      }
      return p;
    })();
  }

  // Single linear walk over each `ERROR_CODE_TOKEN` occurrence; `from` advances
  // monotonically past every rejected candidate, so no prefix is ever rescanned.
  for (let from = 0;;) {
    /**
     Position of the literal error-code token; `-1` ends the search.
     */
    const codeIdx = line.indexOf(
      ERROR_CODE_TOKEN,
      from,
    );
    if (codeIdx === (-1))
      return false;
    /**
     Exclusive end of the trailing digit run; must be followed by `:` to match.
     */
    const codeEnd = endOfDigitRun({
      s: line,
      from: codeIdx + ERROR_CODE_TOKEN
        .length,
    },);
    if ((codeEnd === (codeIdx + ERROR_CODE_TOKEN
      .length))
      || (line.charAt(codeEnd,)
        !== ':'))
      {
      from = codeIdx + 1;
      continue;
    }
    /**
     Exclusive end of the digits in `<col>` (between the `,` and `): error TS`).
     */
    const colEnd = codeIdx;
    /**
     Inclusive start of the column digit run; comma boundary must sit just before.
     */
    const colStart = startOfDigitsBackwards(colEnd,);
    if ((colStart === colEnd) || (line.charAt(colStart - 1,)
      !== ',')) {
      from = codeIdx + 1;
      continue;
    }
    /**
     Inclusive start of the line digit run; opening `(` must sit just before.
     */
    const lineStart = startOfDigitsBackwards(colStart - 1,);
    if ((lineStart === (colStart - 1)) || (line.charAt(lineStart - 1,)
      !== '(')) {
      from = codeIdx + 1;
      continue;
    }
    return true;
  }
}

/**
 Tests whether a diagnostic line originates from a `node_modules` path.
 
 @param line - single diagnostic line of tsc output
 
 @returns true when the file path portion contains `/node_modules/`
 
 @example
 ```ts
 isNodeModulesDiagnostic('node_modules/.bun/\@jsr+zod__zod\@4.3.6/src/index.ts(1,1): error TS2532: Object is possibly undefined.');
 // true
 isNodeModulesDiagnostic('src/index.ts(1,1): error TS2304: Cannot find name.');
 // false
 ```
 
 @internal
 */
export function isNodeModulesDiagnostic(line: string,): boolean {
  return line.includes('node_modules/',)
    || line
      .includes('node_modules\\',);
}

/**
 Tests whether a diagnostic line originates from auto-generated i18n files.
 
 typesafe-i18n generates `i18n-types.ts`, `i18n-util.ts`, and `i18n-util.async.ts`
 with patterns that violate `--isolatedDeclarations`. These files carry
 "Any manual changes will be overwritten" headers, so fixing them is futile.
 
 @param line - single diagnostic line of tsc output
 
 @returns true when the file path matches an auto-generated i18n file
 
 @example
 ```ts
 isI18nGeneratedDiagnostic('src/i18n/i18n-types.ts(4,7): error TS9010: ...');
 // true
 isI18nGeneratedDiagnostic('src/i18n/en/index.ts(4,7): error TS9010: ...');
 // false
 ```
 */
export function isI18nGeneratedDiagnostic(line: string,): boolean {
  // oxlint-disable eslint-plugin-unicorn/prefer-string-raw -- String.raw template literals cannot end with `\` (the trailing backtick is consumed as an escape target); plain '\\' string escapes are the only option for these path separators.
  return line.includes('/i18n/',)
    || line
      .includes('/i18n\\',)
    || line
      .includes('\\i18n/',)
    || line
      .includes('\\i18n\\',);
  // oxlint-enable eslint-plugin-unicorn/prefer-string-raw
}

/**
 Tests whether a diagnostic line should be suppressed.
 
 Suppresses diagnostics from `node_modules` (per {@link isNodeModulesDiagnostic};
 JSR `.ts` source leaking through `skipLibCheck`) and auto-generated typesafe-i18n
 files (per {@link isI18nGeneratedDiagnostic}; these violate `--isolatedDeclarations`
 and cannot be manually fixed).
 
 @param line - single diagnostic line of tsc output
 
 @returns true when the diagnostic should be filtered out
 
 @example
 ```ts
 isSuppressedDiagnostic('node_modules/.bun/zod/src/index.ts(1,1): error TS2532: ...');
 // true
 isSuppressedDiagnostic('src/i18n/i18n-util.ts(24,14): error TS9010: ...');
 // true
 isSuppressedDiagnostic('src/app.ts(5,3): error TS2304: ...');
 // false
 ```
 */
export function isSuppressedDiagnostic(line: string,): boolean {
  return isNodeModulesDiagnostic(line,)
    || isI18nGeneratedDiagnostic(line,);
}

/**
 Tests whether a line is a continuation of a previous diagnostic.
 
 Continuation lines start with whitespace and carry indented context
 for the preceding diagnostic (e.g. type mismatch details).
 
 @param line - single line of tsc output
 
 @returns true when the line starts with whitespace
 
 @example
 ```ts
 isContinuationLine('  Type "string" is not assignable to type "number".');
 // true
 isContinuationLine('src/index.ts(1,1): error TS2304: Cannot find name.');
 // false
 ```
 
 @internal
 */
export function isContinuationLine(line: string,): boolean {
  return (line.length
    > 0) && (line.startsWith(' ',)
      || line
        .startsWith('\t',));
}

//endregion Diagnostic line detection

//region Output filtering

/**
 Filters tsc output to remove suppressed diagnostics.
 
 Suppressed sources: `node_modules` (JSR `.ts` leaking through `skipLibCheck`)
 and auto-generated typesafe-i18n files (`i18n-types.ts`, `i18n-util.ts`, etc.).
 Classifies each line with {@link isDiagnosticLine} and {@link isSuppressedDiagnostic},
 and removes both the diagnostic line itself and any {@link isContinuationLine}
 continuation lines that follow it (indented lines providing additional type
 error context).
 
 @param output - raw tsc stdout or stderr content
 
 @returns object with filtered output and whether any non-suppressed errors remain
 
 @example
 ```ts
 const result = filterTscOutput([
   'node_modules/.bun/zod/src/index.ts(1,1): error TS2532: Object is possibly undefined.',
   '  Type "string" is not assignable.',
   'src/i18n/i18n-util.ts(24,14): error TS9010: Variable must have an explicit type annotation.',
   'src/app.ts(5,3): error TS2304: Cannot find name "foo".',
 ].join('\n'));
 // result.filtered === 'src/app.ts(5,3): error TS2304: Cannot find name "foo".'
 // result.hasRemainingErrors === true
 ```
 
 @internal
 */
export function filterTscOutput(output: string,): {
  readonly filtered: string;
  readonly hasRemainingErrors: boolean;
} {
  if (output.length
    === 0) {
    return {
      filtered: '',
      hasRemainingErrors: false,
    };
  }

  /**
   Source output split per line so each diagnostic header and continuation can be classified independently.
   */
  const lines = output.split('\n',);
  /**
   Lines retained after filtering; rejoined with `\n` to reconstruct the output stream.
   */
  const kept: string[] = [];
  /* oxlint-disable no-restricted-syntax/no-function-root-let -- multi-statement state machine: droppingContinuation and hasRemainingErrors are mutated by four branches across loop iterations, with side effects on `kept`. */
  /**
   True while the loop is inside a suppressed diagnostic block, so its continuation lines are also dropped.
   */
  let droppingContinuation = false;
  /**
   True once any non-suppressed diagnostic is retained; the caller uses it to decide the wrapper's exit code.
   */
  let hasRemainingErrors = false;
  /* oxlint-enable no-restricted-syntax/no-function-root-let */

  for (const line of lines) {
    if (isDiagnosticLine(line,)) {
      if (isSuppressedDiagnostic(line,)) {
        // Drop this diagnostic and mark that following continuation lines should be dropped
        droppingContinuation = true;
      }
      else {
        // Keep this diagnostic
        droppingContinuation = false;
        hasRemainingErrors = true;
        kept.push(line,);
      }
    }
    else if (isContinuationLine(line,)
      && droppingContinuation) {
      // Drop continuation of a node_modules diagnostic
    }
    else {
      // Non-diagnostic, non-continuation line (e.g. summary, blank line)
      droppingContinuation = false;
      kept.push(line,);
    }
  }

  return {
    filtered: kept.join('\n',),
    hasRemainingErrors,
  };
}

//endregion Output filtering
