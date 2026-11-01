import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

/**
 Inputs that determine real-Git executable lookup.
 
 Omitted values come from current process environment.
 
 @example
 ```ts
 const options: ResolveRealGitOptions = {
   pathEnv: '/fixture/bin:/usr/bin',
   commonGitPaths: ['/usr/bin/git'],
 };
 ```
 */
export type ResolveRealGitOptions = {
  /**
   PATH-like string whose directories expose executable candidates.
   */
  readonly pathEnv?: string;
  /**
   Runtime platform controlling executable names and path identity.
   */
  readonly platform?: NodeJS.Platform;
  /**
   Windows executable extensions in shell lookup order.
   */
  readonly pathExtensions?: string;
  /**
   Working directory used to absolutize empty and relative PATH entries.
   */
  readonly cwd?: string;
  /**
   Absolute Git paths promoted when corresponding candidates are exposed by PATH.
   */
  readonly commonGitPaths?: readonly string[];
  /**
   Environment supplying PATH and platform installation roots when explicit options are absent.
   */
  readonly environment?: ForeignBorrowed<NodeJS.ProcessEnv>;
};
