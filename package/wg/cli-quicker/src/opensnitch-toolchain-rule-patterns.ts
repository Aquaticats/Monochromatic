/**
 Name prefix of every daemon rule owned by wg-quicker's toolchain allowance.

 OpenSnitch 1.8 sorts rules by name with a bytewise string sort and, while iterating, returns at once on the
 first matching reject, deny, or precedence rule
 (`daemon/rule/loader.go` `sortRules` and `FindFirstMatch`).
 A leading `000-` therefore sorts before the daemon's auto-saved `reject-always-simple-*` and
 `deny-always-simple-*` names,
 and `precedence: true` makes the allowance return before any non-precedence rule is consulted.
 */
export const TOOLCHAIN_RULE_NAME_PREFIX = '000-allow-monochromatic-toolchain-';

/**
 Description prefix proving a rule file belongs to this allowance,
 so removal never touches a rule the user wrote.
 */
export const TOOLCHAIN_RULE_DESCRIPTION_PREFIX = 'wg-quicker managed toolchain update allowance';

/**
 Home spellings that name one directory on this host family.

 Fedora Atomic keeps homes in `/var/home` and links `/home` to it,
 and OpenSnitch records whichever spelling the process path used.
 */
export type PathAliasPair = readonly [string, string,];

/**
 One generated daemon rule before it is rendered as JSON.
 */
export type ToolchainRuleSpec = {
  /**
   Rule name and file stem,
   starting with {@link TOOLCHAIN_RULE_NAME_PREFIX}.
   */
  readonly name: string;

  /**
   Human-readable scope shown in the OpenSnitch UI after the ownership prefix.
   */
  readonly summary: string;

  /**
   Anchored RE2 expression compared against `process.path`.
   */
  readonly pattern: string;
};

/**
 Every spelling of each directory the allowance must cover.
 */
export type ToolchainRuleRoots = {
  /**
   Spellings of the XDG data home holding `monochromatic-dev/toolchain` and `monochromatic-dev/toolchain-global`.
   */
  readonly dataHomes: readonly string[];

  /**
   Spellings of the Cargo home whose `bin` directory holds rustup's proxies and cargo-binstall.
   */
  readonly cargoHomes: readonly string[];

  /**
   Spellings of the rustup home whose `toolchains` directory holds each installed compiler.
   */
  readonly rustupHomes: readonly string[];

  /**
   Spellings of the main checkout whose `.cache` and `.claude/worktrees/<id>/.cache` hold exposure links.
   */
  readonly repositoryRoots: readonly string[];
};

/**
 Characters with meaning in RE2 syntax,
 each of which a literal path must escape.
 */
const RE2_SPECIAL_CHARACTERS = new Set(['\\', '.', '+', '*', '?', '(', ')', '|', '[', ']', '{', '}', '^', '$',],);

/**
 Escapes text so an RE2 expression matches it literally.

 @param text - Literal path or segment.

 @returns Expression matching only `text`.

 @example
 ```ts
 escapeRe2Literal({ text: '/home/me/.cargo' }); // '/home/me/\\.cargo'
 ```
 */
export function escapeRe2Literal({ text, }: { readonly text: string; },): string {
  return Array
    .from(text,)
    .map(function escapeCharacter(character,): string {
      return RE2_SPECIAL_CHARACTERS.has(character,) ? `\\${character}` : character;
    },)
    .join('',);
}

/**
 Removes trailing slashes so spellings compare and concatenate predictably.

 @param path - Absolute directory.

 @returns Directory without a trailing slash, keeping a lone root slash empty.

 @example
 ```ts
 trimTrailingSlashes({ path: '/home/me/' }); // '/home/me'
 ```
 */
function trimTrailingSlashes({ path, }: { readonly path: string; },): string {
  /**
   Index one past the last character that is not a slash.
   */
  let end = path.length;
  while ((end > 0) && (path[end - 1] === '/'))
    end -= 1;
  return path.slice(
    0,
    end,
  );
}

/**
 Expands one absolute directory into every spelling a process path may use.

 Each alias pair maps a prefix to its twin in both directions,
 so `/home/me` yields `/var/home/me` and the reverse.
 The input spelling comes first;
 duplicates collapse.

 @param path - Absolute directory as the caller or `realpath` spelled it.

 @param aliasPairs - Prefix pairs naming one directory.

 @returns Distinct spellings,
 input first.

 @example
 ```ts
 pathSpellings({ path: '/home/me', aliasPairs: [['/home', '/var/home',],], }); // ['/home/me', '/var/home/me']
 ```
 */
export function pathSpellings(
  {
    path,
    aliasPairs,
  }: {
    readonly path: string;
    readonly aliasPairs: readonly PathAliasPair[];
  },
): readonly string[] {
  /**
   Input without a trailing slash.
   */
  const trimmed = trimTrailingSlashes({ path, },);
  /**
   Spellings in discovery order.
   */
  const spellings: string[] = [trimmed,];
  for (const [first, second,] of aliasPairs) {
    for (const [from, to,] of [[first, second,], [second, first,],] as const) {
      if (trimmed.startsWith(`${from}/`,))
        spellings.push(`${to}${trimmed.slice(from.length,)}`,);
    }
  }
  return [...new Set(spellings,),];
}

/**
 Joins escaped directory spellings into one non-capturing alternation.

 @param directories - Directory spellings.

 @returns Group matching any one directory literally.

 @example
 ```ts
 directoryAlternation({ directories: ['/a', '/b',] }); // '(?:/a|/b)'
 ```
 */
function directoryAlternation({ directories, }: { readonly directories: readonly string[]; },): string {
  return `(?:${
    directories
      .map(function escapeDirectory(directory,): string {
        return escapeRe2Literal({ text: directory, },);
      },)
      .join('|',)
  })`;
}

/**
 Builds the daemon rules covering the toolchain's install roots,
 exposure links,
 and Rust tools.

 The first rule matches everything below `monochromatic-dev/toolchain/` and `monochromatic-dev/toolchain-global/`,
 where `mpm-shadow` and the TypeScript sync install every tool version,
 so a new version needs no new rule.
 The second matches direct children of `.cache/toolchain-bin/` and `.cache/toolchain-launchers/`
 for the main checkout and each `.claude/worktrees/<id>`.
 The third matches rustup's proxies and cargo-binstall in the Cargo home and every binary of each rustup toolchain.
 Every expression is anchored at both ends of the directory it names,
 so a sibling such as `toolchain-evil/` or a root appearing mid-path never matches.

 @param roots - Spellings of each covered directory.

 @returns Rule specs in stable name order,
 omitting a rule whose roots are empty.

 @example
 ```ts
 toolchainRuleSpecs({ roots: { dataHomes: ['/home/me/.local/share',], cargoHomes: [], rustupHomes: [], repositoryRoots: [], }, });
 ```
 */
export function toolchainRuleSpecs(
  { roots, }: { readonly roots: ToolchainRuleRoots; },
): readonly ToolchainRuleSpec[] {
  /**
   Install-root expression, or absent when no data home exists.
   */
  const installRoots: readonly ToolchainRuleSpec[] = roots.dataHomes.length === 0
    ? []
    : [
      {
        name: `${TOOLCHAIN_RULE_NAME_PREFIX}install-roots`,
        summary: 'binaries below monochromatic-dev/toolchain and monochromatic-dev/toolchain-global',
        pattern: `^${
          directoryAlternation({
            directories: roots.dataHomes.map(function dataHomeRoot(dataHome,): string {
              return `${dataHome}/monochromatic-dev`;
            },),
          },)
        }/toolchain(?:-global)?/`,
      },
    ];
  /**
   Exposure-link expression, or absent when no checkout root exists.
   */
  const exposureLinks: readonly ToolchainRuleSpec[] = roots.repositoryRoots.length === 0
    ? []
    : [
      {
        name: `${TOOLCHAIN_RULE_NAME_PREFIX}exposure-links`,
        summary: 'files directly in .cache/toolchain-bin and .cache/toolchain-launchers of the checkout and its worktrees',
        pattern: `^${
          directoryAlternation({ directories: roots.repositoryRoots, },)
        }/(?:\\.claude/worktrees/[^/]+/)?\\.cache/(?:toolchain-bin|toolchain-launchers)/[^/]+$`,
      },
    ];
  /**
   Cargo-home alternative, or absent.
   */
  const cargoBranch = roots.cargoHomes.length === 0
    ? []
    : [`${directoryAlternation({ directories: roots.cargoHomes, },)}/bin/(?:cargo|cargo-binstall|rustup)`,];
  /**
   Rustup-home alternative, or absent.
   */
  const rustupBranch = roots.rustupHomes.length === 0
    ? []
    : [`${directoryAlternation({ directories: roots.rustupHomes, },)}/toolchains/[^/]+/bin/[^/]+`,];
  /**
   Both Rust alternatives joined into one expression.
   */
  const rustBranches = [...cargoBranch, ...rustupBranch,];
  /**
   Rust-tool expression, or absent when neither home exists.
   */
  const rustTools: readonly ToolchainRuleSpec[] = rustBranches.length === 0
    ? []
    : [
      {
        name: `${TOOLCHAIN_RULE_NAME_PREFIX}rust-tools`,
        summary: 'cargo, cargo-binstall and rustup in the Cargo home bin, and every binary of each rustup toolchain',
        pattern: `^(?:${rustBranches.join('|',)})$`,
      },
    ];
  return [...installRoots, ...exposureLinks, ...rustTools,];
}

/**
 Formats a moment as the RFC 3339 text OpenSnitch stores in `created` and `updated`.

 @param moment - Instant to render.

 @returns UTC timestamp with a `Z` offset.

 @example
 ```ts
 rfc3339({ moment: new Date(0) }); // '1970-01-01T00:00:00Z'
 ```
 */
export function rfc3339({ moment, }: { readonly moment: Date; },): string {
  // `toISOString` always renders `YYYY-MM-DDTHH:mm:ss.sssZ`, so the first 19 characters end at the seconds.
  return `${
    moment
      .toISOString()
      .slice(
        0,
        19,
      )
  }Z`;
}

/**
 JSON object the daemon loads from one rule file.
 */
export type ToolchainRuleFile = {
  readonly created: string;
  readonly updated: string;
  readonly name: string;
  readonly description: string;
  readonly action: 'allow';
  readonly duration: 'always';
  readonly operator: {
    readonly operand: 'process.path';
    readonly data: string;
    readonly type: 'regexp';
    readonly list: readonly [];
    readonly sensitive: true;
  };
  readonly enabled: true;
  readonly precedence: true;
  readonly nolog: false;
};

/**
 Renders one spec as the daemon's rule JSON.

 `sensitive` is true because the daemon lowercases a case-insensitive pattern before compiling it,
 which would turn `[A-Z]` or `\S` into a different expression and miss `Monochromatic`.

 @param spec - Rule to render.

 @param created - Creation timestamp,
 kept from an existing file on rewrite.

 @param updated - Modification timestamp.

 @returns Rule object whose key order matches the daemon's own saves.

 @example
 ```ts
 toolchainRuleFile({ spec, created: '2026-10-31T00:00:00Z', updated: '2026-10-31T00:00:00Z' });
 ```
 */
export function toolchainRuleFile(
  {
    spec,
    created,
    updated,
  }: {
    readonly spec: ToolchainRuleSpec;
    readonly created: string;
    readonly updated: string;
  },
): ToolchainRuleFile {
  return {
    created,
    updated,
    name: spec.name,
    description: `${TOOLCHAIN_RULE_DESCRIPTION_PREFIX}: ${spec.summary}`,
    action: 'allow',
    duration: 'always',
    operator: {
      operand: 'process.path',
      data: spec.pattern,
      type: 'regexp',
      list: [],
      sensitive: true,
    },
    enabled: true,
    precedence: true,
    nolog: false,
  };
}
