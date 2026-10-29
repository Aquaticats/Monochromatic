// TODO: deprecate Optique
/* oxlint-disable no-restricted-syntax/no-nullish-union -- mirrors @optique/core `optional(option(...))` output: optique yields each optional value as `T | undefined` with the key always present (not absent), so `ParsedArgs` must declare `T | undefined` to stay structurally assignable from `runSync(parser)`. The nullish unions are dictated by the external parser, not modelling our own optionality. */
/**
 Shape produced by {@link parseArgs}.
 
 Spelled explicitly (not inferred via `InferValue<typeof parser>`)
 because `--isolatedDeclarations` does not survive optique's deeply-
 generic combinators across the export boundary; the explicit shape
 also doubles as documentation for the test fixtures.
 */
export type ParsedArgs = {
  /**
   `-w` / `--watch`; watch roots in argv order.
   */
  readonly watch: readonly string[];
  /**
   `-i` / `--include`; include globs in argv order.
   */
  readonly include: readonly string[];
  /**
   `-e` / `--exclude`; exclude globs in argv order.
   */
  readonly exclude: readonly string[];
  /**
   `--include-regex`; raw regex source strings in argv order.
   */
  readonly includeRegex: readonly string[];
  /**
   `--exclude-regex`; raw regex source strings in argv order.
   */
  readonly excludeRegex: readonly string[];
  /**
   `--ext`; raw values pre-split (each entry may be a comma list).
   */
  readonly ext: readonly string[];
  /**
   `--type`; raw type tokens (each entry may be a comma list).
   */
  readonly type: readonly string[];
  /**
   `--events`; raw comma list, or `undefined` when not passed.
   */
  readonly events: string | undefined;
  /**
   `--hidden`; `true` when passed.
   */
  readonly hidden: boolean;
  /**
   `--no-hidden`; `true` when passed.
   */
  readonly noHidden: boolean;
  /**
   `--follow-symlinks`; `true` when passed.
   */
  readonly followSymlinks: boolean;
  /**
   `--no-follow-symlinks`; `true` when passed.
   */
  readonly noFollowSymlinks: boolean;
  /**
   `--gitignore`; `true` when passed.
   */
  readonly gitignore: boolean;
  /**
   `--no-gitignore`; `true` when passed.
   */
  readonly noGitignore: boolean;
  /**
   `--ignore-file`; extra gitignore-format files in argv order.
   */
  readonly ignoreFile: readonly string[];
  /**
   `--depth`; parsed integer or `undefined`.
   */
  readonly depth: number | undefined;
  /**
   `--poll`; parsed integer or `undefined`.
   */
  readonly poll: number | undefined;
  /**
   `--no-content-changed`; `true` when passed.
   */
  readonly noContentChanged: boolean;
  /**
   `--max-hash-size`; parsed integer or `undefined`.
   */
  readonly maxHashSize: number | undefined;
  /**
   `--debounce`; parsed integer or `undefined`.
   */
  readonly debounce: number | undefined;
  /**
   `--stop-timeout`; parsed integer or `undefined`.
   */
  readonly stopTimeout: number | undefined;
  /**
   `--no-initial`; `true` when passed.
   */
  readonly noInitial: boolean;
  /**
   `--clear`; `true` when passed.
   */
  readonly clear: boolean;
  /**
   `--no-clear`; `true` when passed.
   */
  readonly noClear: boolean;
  /**
   `--signal`; raw signal name, or `undefined` when not passed.
   */
  readonly signal: string | undefined;
  /**
   `--process-group`; `true` when passed.
   */
  readonly processGroup: boolean;
  /**
   `--no-process-group`; `true` when passed.
   */
  readonly noProcessGroup: boolean;
  /**
   Positional args after `--`; first is command, rest is its args.
   */
  readonly rest: readonly string[];
};
/* oxlint-enable no-restricted-syntax/no-nullish-union */
