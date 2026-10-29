import type {
  PackageEntry,
  PackageManager,
  PackageSpec,
} from './types.ts';

/**
 Default flag used for binary existence checks when no custom check is specified.
 */
export const DEFAULT_CHECK = '--version';

//region p() builder

/**
 Builds a {@link PackageEntry} from a shorthand string via {@link buildFromShorthand}
 or a detailed spec via {@link buildFromSpec}.
 
 **String form**: binary name, effname, and all per-manager package names are the same value.
 Availability is unrestricted (`available` omitted).
 
 **Object form**: `bin` defaults to `effname` when omitted.
 When `yes` is present, it encodes both availability and per-manager name overrides.
 When `yes` is absent, availability is unrestricted (`available` omitted).
 
 @param shorthandOrSpec - Name string or object with `effname`, optional `bin`, and optional `yes` array
 
 @returns Immutable package entry
 
 @example
 ```ts
 p('curl')
 // { bin: 'curl', effname: 'curl', overrides: {} }
 p({ bin: 'rg', effname: 'ripgrep' })
 p({ effname: 'acpica', yes: ['apt', ['dnf', 'acpica-tools'], ['pacman', 'acpica-utils']] })
 ```
 */
export function p(shorthandOrSpec: string | PackageSpec,): PackageEntry {
  if ((typeof shorthandOrSpec) === 'string')
    return buildFromShorthand(shorthandOrSpec,);
  return buildFromSpec(shorthandOrSpec,);
}

//endregion p() builder

//region Internal builders

/**
 Creates a {@link PackageEntry} where all names are identical and `check`
 falls back to {@link DEFAULT_CHECK}.
 
 @param name - Shared value for `bin`, `effname`, and all managers
 
 @returns Immutable package entry with unrestricted availability and empty overrides
 */
function buildFromShorthand(name: string,): PackageEntry {
  return {
    bin: name,
    check: DEFAULT_CHECK,
    effname: name,
    overrides: Object.freeze({},),
  };
}

/**
 Creates a {@link PackageEntry} from a structured spec, defaulting `check` to
 {@link DEFAULT_CHECK} and parsing the `yes` array into availability set and
 per-manager overrides via {@link parseYes}.
 
 @param spec - Full package specification
 
 @returns Immutable package entry
 */
function buildFromSpec(spec: PackageSpec,): PackageEntry {
  /**
   Destructured spec fields; pulled out so the return literal stays compact.
   */
  const {
    bin,
    check,
    effname,
    yes,
  } = spec;

  if (yes === undefined)
    return {
      bin: bin ?? effname,
      check: check ?? DEFAULT_CHECK,
      effname,
      overrides: Object.freeze({},),
    };

  /**
   Availability set and per-manager overrides parsed from the supplied `yes` array.
   */
  const {
    available,
    overrides,
  } = parseYes(yes,);
  return {
    available,
    bin: bin ?? effname,
    check: check ?? DEFAULT_CHECK,
    effname,
    overrides,
  };
}

/**
 Parses a `yes` availability array into a frozen availability set
 and a frozen overrides map.
 
 @param yes - Array of manager names or `[manager, packageName]` tuples
 
 @returns Availability set and per-manager name overrides extracted from tuples
 */
function parseYes(
  yes: readonly (PackageManager | readonly [
    PackageManager,
    string,
  ])[],
): {
  readonly available: ReadonlySet<PackageManager>;
  readonly overrides: Readonly<Record<string, string>>;
} {
  /**
   Managers that can supply this package; populated as the `yes` array is walked.
   */
  const available = new Set<PackageManager>();
  /**
   Per-manager package-name overrides extracted from `[manager, packageName]` tuples.
   */
  const overrides: Record<string, string> = {};
  for (const entry of yes) {
    if ((typeof entry) === 'string')
      available.add(entry,);
    else {
      /**
       Tuple split: explicit manager plus its custom package name.
       */
      const [manager, packageName,] = entry;
      available.add(manager,);
      overrides[manager] = packageName;
    }
  }
  return {
    available: Object.freeze(available,),
    overrides: Object.freeze(overrides,),
  };
}

//endregion Internal builders
