/**
 Ambient module declaration for picomatch 4.
 
 picomatch ships without `.d.ts` files and `@types/picomatch` does not
 publish v4-shaped types in this workspace's catalog. This shim
 declares the narrow subset `globFilter` needs: the pattern compiler
 call and the matcher predicate it returns. Replace with upstream
 types if picomatch ever publishes them or `@types/picomatch` lands
 in the catalog.
 */
declare module 'picomatch' {
  /**
   Compiled matcher: takes a path string, returns whether it matches
   the pattern this matcher was compiled from.
   */
  type Matcher = (test: string,) => boolean;

  /**
   Compiles a glob pattern (or list of patterns) into a {@link Matcher}.
   
   @param glob - one or more glob patterns
   
   @returns matcher function
   */
  function picomatch(glob: string | readonly string[],): Matcher;

  export = picomatch;
}
