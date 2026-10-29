/**
 Shared field-probe types used by registry, repository, and parser helpers.
 
 @example
 ```ts
 import type { NpmPackage, RepositoryInfo } from './probe-field-types.ts';
 ```
 */

//region Public types

/**
 Subset of npm registry package-level response that the probe consumes.
 */
export type NpmPackage = {
  readonly 'dist-tags'?: { readonly latest?: string; };
  readonly time?: { readonly created?: string; };
  readonly versions?: Readonly<Record<string, NpmVersion>>;
};

/**
 Subset of one version's manifest.
 */
export type NpmVersion = {
  readonly repository?:
    | string
      | {
        readonly type?: string;
        readonly url?: string;
        readonly directory?: string;
      };
  readonly dependencies?: Readonly<Record<string, string>>;
  readonly dist?: { readonly unpackedSize?: number; };
  readonly license?: string | { readonly type?: string; };
};

/**
 Output of `repository` normalisation.
 
 Parsers return this type or a descriptive absence sentinel; an unparseable
 repository field yields {@link REPO_UNPARSEABLE} rather than a nullish union.
 */
export type RepositoryInfo = {
  readonly host: 'github' | 'other';
  readonly owner: string;
  readonly repo: string;
  readonly directory?: string;
  /**
   Raw URL parsed, useful for the tooltip.
   */
  readonly url: string;
};

/**
 License classes used for filter/color groupings.
 */
export type LicenseClass = 'permissive' | 'copyleft' | 'non-oss' | 'unknown';

//endregion Public types
