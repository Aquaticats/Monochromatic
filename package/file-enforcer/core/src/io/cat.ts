import {
  trackGlob,
  trackRead,
} from '../tracker.ts';
import { readCached, } from './cache.ts';
import { expandGlob, } from './glob.ts';

/**
 Result of expanding a glob: each matched file with its path and content
 */
export type GlobResult = {
  readonly path: string;
  readonly content: string;
};

/**
 Array of glob results carrying the source pattern used to produce them
 */
export type GlobResults = readonly GlobResult[] & {
  readonly sourceGlob: string;
};

/**
 Creates a {@link GlobResults} array from a source glob and its matched files.
 Useful for constructing results outside of `cat()` (e.g. in tests).
 
 @param sourceGlob - Glob pattern that produced the results
 
 @param results - Matched files with paths and contents
 
 @returns Branded array with the source glob attached
 
 @example
 ```ts
 const files = globResults({
   sourceGlob: join(srcDir, '*.ts'),
   results: [{ path: join(srcDir, 'a.ts'), content: 'alpha' }],
 });
 ```
 */
export function globResults(
  {
    sourceGlob,
    results,
  }: {
    readonly sourceGlob: string;
    readonly results: readonly GlobResult[];
  },
): GlobResults {
  /* oxlint-disable typescript/no-unsafe-type-assertion -- branded array gains required property immediately after construction */
  /**
   Fresh result array receiving required source identity.
   */
  const brandedResults = [...results,] as unknown as GlobResult[] & { sourceGlob: string; };
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  brandedResults.sourceGlob = sourceGlob;
  return brandedResults;
}

/**
 Returns true when `path` contains a `*` or `?` glob metacharacter
 (the two characters the cat-array fast-path uses to decide between
 literal-path and glob-expansion handling).
 
 @param path - candidate path or pattern
 
 @returns whether the path needs glob expansion
 */
function hasGlobChars(path: string,): boolean {
  return path.includes('*',)
    || path
      .includes('?',);
}

/**
 Reads files by glob pattern, keeping them separate for {@link overwriteEach}.
 @param glob - Glob pattern to match source files
 @returns Array of matched files with their paths, contents, and source glob
 
 @example
 ```ts
 const files = await cat('./src/*.ts');
 ```
 */
export async function cat(glob: string,): Promise<GlobResults>;

/**
 Reads and concatenates listed files into a single string.
 Paths containing `*` or `?` are auto-expanded as globs before concatenation.
 @param files - Array of file paths (or glob patterns) to concatenate
 @returns Concatenated file contents joined by newlines
 
 @example
 ```ts
 const merged = await cat(['./header.txt', './body.txt']);
 ```
 */
export async function cat(files: readonly string[],): Promise<string>;

/**
 {@inheritDoc cat}
 
 @param input - Glob pattern string or array of file paths to read
 
 @returns Matched files as {@link GlobResults} for glob mode, or concatenated contents for array mode
 */
export async function cat(
  input: string | readonly string[],
): Promise<string | GlobResults> {
  if ((typeof input) === 'string') {
    /**
     Paths matched by the glob pattern
     */
    const paths = await expandGlob(input,);
    trackGlob({
      pattern: input,
      paths,
    },);
    /**
     Matched files with their contents
     */
    const results = await Promise.all(
      paths.map(async function readGlobMatch(path: string,): Promise<GlobResult> {
        trackRead(path,);
        return {
          path,
          content: await readCached(path,),
        };
      },),
    );
    return globResults({
      sourceGlob: input,
      results,
    },);
  }

  /**
   Expand any glob patterns in the array, then flatten
   */
  const expandedGroups = await Promise.all(
    input.map(
      async function expandOnePath(
        path: string,
      ): Promise<readonly string[]> {
        if (hasGlobChars(path,)) {
          /**
           Paths matched by this array-mode glob.
           */
          const paths = await expandGlob(path,);
          trackGlob({
            pattern: path,
            paths,
          },);
          return paths;
        }
        return [path,];
      },
    ),
  );

  /**
   File contents read in parallel
   */
  const contents = await Promise.all(
    expandedGroups.flat()
      .map(function readOneFile(filePath: string,): Promise<string> {
        trackRead(filePath,);
        return readCached(filePath,);
      },),
  );
  return contents.join('\n',);
}
