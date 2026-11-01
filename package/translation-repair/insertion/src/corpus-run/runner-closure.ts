import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import { readFile, } from 'node:fs/promises';
import { basename, } from 'node:path';

//region Runner closure
// WHICH CODE A RUN ACTUALLY EXECUTED, which the pipeline digest cannot say.
//
// `digestPipeline` hashes the whole built tree, so any two runs separated by any
// commit carry different stamps, including commits the runner never loads. That
// makes the one question a band measurement has to answer unanswerable from the
// recorded identity: two runs of byte-identical audit code report different
// digests, and nothing distinguishes that from a real change. The settled audit hit
// exactly this and had to argue the point in prose instead of reading it off the file.
//
// THE ANSWER IS ALREADY ON DISK AND FREE. Rolldown emits content-hashed chunks,
// and the built entry names its own dependency closure in its import statements.
// Two runs whose closure matches ran the same code, checkable by string equality
// with no rebuild.
//
// BOTH ARE WORTH KEEPING. The tree digest answers "did the repository move at
// all", which is a real question about reproducibility. The closure answers "did
// THIS run's code move", which is the one a comparison needs.
//
// READ AT RUN START, never at the end, for the reason the settled audit learned
// the expensive way: a rebuild mid-run otherwise stamps a build that never ran.

/**
 Import specifiers a built entry can carry, in both the spaced form a reader
 writes and the tight form a minifier emits.

 THE TIGHT FORM IS THE ONE THAT MATTERS and the one that gets forgotten. The
 built runners here are minified onto a single line, so their imports read
 `from"./chunk.mjs"` with no space. A scan expecting `from './` finds nothing
 and reports a clean closure for a file full of imports, which is a false null
 that looks exactly like a self-contained bundle.
 */
const IMPORT_MARKERS = [
  'from"./',
  'from\'./',
  'from "./',
  'from \'./',
] as const;

/**
 What a run executed, or a positive statement that it could not be read.

 A TAGGED ABSENCE rather than an empty list, because an entry that imports
 nothing and an entry nobody could read are opposite findings. The first is a
 self-contained bundle whose closure is itself; the second says nothing at all,
 and comparing two of them for equality would call two unknown builds the same.

 @example
 ```ts
 const closure: RunnerClosure = { kind: 'read', entry: 'probe.mjs', chunks: [], };
 ```
 */
export type RunnerClosure = {
  readonly kind: 'read';

  /**
   Entry file this describes, by basename, since the directory is an artifact
   of where the run happened rather than of what it ran.
   */
  readonly entry: string;

  /**
   Chunks the entry imports, sorted and deduplicated so two runs of one build
   compare equal regardless of import order.

   EMPTY IS A REAL ANSWER: an entry with everything inlined imports nothing,
   and its closure is itself.
   */
  readonly chunks: readonly string[];
} | {
  readonly kind: 'unavailable';

  /**
   Why, in enough detail to tell a source run from a missing file.
   */
  readonly reason: string;
};

/**
 Reads one import specifier that begins just past a marker.

 @param text - whole entry file

 @param from - index of the first character of the specifier

 @returns Specifier up to its closing quote, empty when the quote never closes

 @example
 ```ts
 const chunk = specifierAt({ text, from: 12, },);
 ```
 */
function specifierAt(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): string {
  /**
   Where the specifier ends, at whichever quote closes it first.
   */
  const single = text.indexOf(
    '\'',
    from,
  );

  /**
   The other quote style, since a bundler may emit either.
   */
  const double = text.indexOf(
    '"',
    from,
  );

  /**
   Closest closing quote, ignoring whichever was not found.
   */
  const end = Math.min(
    (single === (-1)) ? text.length : single,
    (double === (-1)) ? text.length : double,
  );

  if (end === text.length)
    return '';
  return text.slice(
    from,
    end,
  );
}

/**
 Reads an entry file, reporting failure as a value.

 ITS OWN FUNCTION so the caller has no mutable binding at its root: a `let`
 assigned inside a `try` leaks scope to every statement after it, and the
 failure it exists to carry is exactly the one this returns instead.

 @param entryPath - built entry to read

 @returns Its text, or why there is none

 @example
 ```ts
 const source = await readEntryText({ entryPath, },);
 ```
 */
async function readEntryText(
  { entryPath, }: { readonly entryPath: string; },
): Promise<{
  readonly kind: 'read';
  readonly text: string;
} | {
  readonly kind: 'unavailable';
  readonly reason: string;
}> {
  try {
    return {
      kind: 'read',
      text: await readFile(
        entryPath,
        'utf8',
      ),
    };
  }
  catch (error) {
    return {
      kind: 'unavailable',
      reason: `could not read ${entryPath}: ${String(error,)}`,
    };
  }
}

/**
 Reads the chunks a built entry imports.

 A LINEAR SCAN RATHER THAN A REGEX, per `RG1`: the rule is "a specifier begins
 after one of four fixed markers and ends at the next quote", which `indexOf`
 states directly and a pattern would only obscure.

 @param entryPath - built entry the run is executing, ordinarily
 the script the runtime ran, as the command line reader hands it on (`line.script`)

 @returns Its closure, or why it could not be read

 @example
 ```ts
 const closure = await readRunnerClosure({ entryPath: line.script, },);
 ```
 */
export async function readRunnerClosure(
  { entryPath, }: { readonly entryPath: string; },
): Promise<RunnerClosure> {
  if (entryPath === '')
    return {
      kind: 'unavailable',
      reason: 'no entry path was given',
    };

  /**
   Entry as built, or why it could not be had.
   */
  const source = await readEntryText({ entryPath, },);
  if (source.kind === 'unavailable')
    return source;

  /**
   Its contents.
   */
  const { text, } = source;

  /**
   Every relative specifier the entry imports, in file order, duplicates and
   all.
   */
  const found = IMPORT_MARKERS.flatMap(function scanFor(marker,): readonly string[] {
    /**
     Specifiers this marker turns up.
     */
    const hits: string[] = [];

    /**
     Cursor, advanced past each hit so the scan terminates.
     */
    let at = text.indexOf(marker,);
    while (at !== (-1)) {
      /**
       Specifier body, which starts after the marker's own `./`.
       */
      const chunk = specifierAt({
        text,
        from: at + marker.length,
      },);
      if (chunk !== '')
        hits.push(chunk,);
      at = text.indexOf(
        marker,
        at + marker.length,
      );
    }
    return hits;
  },);

  return {
    kind: 'read',
    entry: basename(entryPath,),
    chunks: textsInCodePointOrder({ texts: [...new Set(found,),], },),
  };
}

//endregion Runner closure
