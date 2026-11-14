// Attributes a `node --cpu-prof` profile of the commit-region workload: which
// functions hold the self time inside the parse, which library files hold it,
// and how much sits in suggestion or edit-distance code.

import { readFileSync, } from 'node:fs';
import { basename, } from 'node:path';

//region Profile shape

/**
 * One frame in a V8 CPU profile.
 */
type CallFrame = {
  readonly functionName: string;
  readonly url: string;
  readonly lineNumber: number;
  readonly columnNumber: number;
};

/**
 * One node of the sampled call tree.
 */
type ProfileNode = {
  readonly id: number;
  readonly callFrame: CallFrame;
  readonly children?: readonly number[];
};

/**
 * Shape of a `.cpuprofile` document, limited to the fields read here.
 */
type CpuProfile = {
  readonly nodes: readonly ProfileNode[];
  readonly samples: readonly number[];
  readonly timeDeltas: readonly number[];
  readonly startTime: number;
  readonly endTime: number;
};

//endregion Profile shape

//region Classification

/**
 * Function names counted as suggestion or edit-distance work. Upstream names
 * its edit distance `levenshteinDistance` and its candidate search
 * `findSimilar` in every measured version.
 */
const SUGGESTION_NAME_PATTERN = /suggest|similar|levenshtein|distance|typo/i;

/**
 * Suffix of the file that declares `parseCommitRegion`.
 */
const PARSER_FILE_SUFFIX = '/src/parser/commit.ts';

/**
 * Function name that marks the start of a direct `parseSync()` call, used for
 * shapes that call the library straight from the runner.
 */
const PARSE_SYNC_FUNCTION = 'parseSync';

/**
 * Directory fragment of the wrapper's own copied sources.
 */
const WRAPPER_SOURCE_FRAGMENT = '/src/';

/**
 * Directory fragment of the installed library.
 */
const OPTIQUE_FRAGMENT = '/node_modules/@optique/core/';

/**
 * Shortens a profile URL to something readable in a report.
 *
 * @param url - Raw frame URL, possibly empty.
 *
 * @returns File name, or a placeholder for frames without a URL.
 */
function shortFile(url: string,): string {
  if (url === '') {
    return '(v8)';
  }
  if (url.startsWith('node:',)) {
    return url;
  }

  return basename(url,);
}

//endregion Classification

//region Analysis

/**
 * Builds a child-to-parent map from the profile's children lists.
 *
 * @param nodes - Every node of the profile.
 *
 * @returns Parent node id per child node id.
 */
function buildParentMap(nodes: readonly ProfileNode[],): Map<number, number> {
  /** Parent node id per child node id. */
  const parents = new Map<number, number>();

  for (const node of nodes) {
    for (const child of node.children ?? []) {
      parents.set(
        child,
        node.id,
      );
    }
  }

  return parents;
}

/**
 * Decides whether a sample sits inside a parse call by walking up its call
 * stack to any frame of the parser file.
 *
 * @param nodeId - Sampled node id.
 *
 * @param nodesById - Node lookup.
 *
 * @param parents - Parent map.
 *
 * @param parseRootIds - Node ids of `parseCommitRegion` entry frames.
 *
 * @returns Whether the sample is inside a parse.
 */
function isInsideParse(
  nodeId: number,
  nodesById: Map<number, ProfileNode>,
  parents: Map<number, number>,
  parseRootIds: ReadonlySet<number>,
): boolean {
  /** Node currently walked. */
  let current: number | undefined = nodeId;

  while (current !== undefined) {
    if (parseRootIds.has(current,)) {
      return true;
    }
    current = parents.get(current,);
  }

  return false;
}

/**
 * Attributes a profile to functions, files, and code categories.
 *
 * @param profilePath - Path of the `.cpuprofile` file.
 *
 * @param root - `commit` for the wrapper's parser file, `parseSync` for shapes
 * that call the library directly.
 *
 * @returns Report object printed as JSON.
 */
function analyse(profilePath: string, root: 'commit' | 'parseSync',): Record<string, unknown> {
  /** Parsed profile document. */
  const profile = JSON.parse(
    readFileSync(
      profilePath,
      'utf8',
    ),
  ) as CpuProfile;
  /** Node lookup by id. */
  const nodesById = new Map<number, ProfileNode>(
    profile.nodes.map(function byId(node,) {
      return [
        node.id,
        node,
      ];
    },),
  );
  /** Parent node id per child node id. */
  const parents = buildParentMap(profile.nodes,);
  /** Nodes that declare the parse entry point. */
  const parseRootIds = new Set<number>(
    profile.nodes
      .filter(function isParserEntry(node,) {
        return root === 'commit'
          ? node.callFrame.url.endsWith(PARSER_FILE_SUFFIX,)
          : node.callFrame.functionName === PARSE_SYNC_FUNCTION;
      },)
      .map(function idOf(node,) {
        return node.id;
      },),
  );
  /** Profile wall time in milliseconds. */
  const totalMs = (profile.endTime - profile.startTime) / 1000;
  /** Sampling interval actually achieved. */
  const samplingIntervalMs = profile.samples.length === 0
    ? 0
    : totalMs / profile.samples.length;

  /** Self time inside parses, per function key. */
  const selfByFunction = new Map<string, { name: string; file: string; micros: number; }>();
  /** Self time inside parses, per file. */
  const selfByFile = new Map<string, number>();
  /** Microseconds inside parses. */
  let parseMicros = 0;
  /** Samples inside parses. */
  let parseSamples = 0;
  /** Microseconds inside suggestion or edit-distance functions. */
  let suggestionMicros = 0;
  /** Microseconds inside the installed library. */
  let optiqueMicros = 0;
  /** Microseconds inside the wrapper's own copied sources. */
  let wrapperMicros = 0;

  for (const [index, nodeId,] of profile.samples.entries()) {
    /** Time charged to this sample. */
    const delta = profile.timeDeltas[index] ?? 0;
    /** Node this sample landed in. */
    const node = nodesById.get(nodeId,);

    if (node === undefined) {
      continue;
    }
    if (!isInsideParse(
      nodeId,
      nodesById,
      parents,
      parseRootIds,
    )) {
      continue;
    }

    parseMicros += delta;
    parseSamples += 1;

    /** Readable file name for this frame. */
    const file = shortFile(node.callFrame.url,);
    /** Aggregation key distinguishing same-named functions in other files. */
    const functionKey = `${file}#${node.callFrame.functionName}`;
    /** Running total for this function. */
    const existing = selfByFunction.get(functionKey,);

    selfByFunction.set(
      functionKey,
      {
        name: node.callFrame.functionName === '' ? '(anonymous)' : node.callFrame.functionName,
        file,
        micros: (existing?.micros ?? 0) + delta,
      },
    );
    selfByFile.set(
      file,
      (selfByFile.get(file,) ?? 0) + delta,
    );

    if (SUGGESTION_NAME_PATTERN.test(node.callFrame.functionName,)) {
      suggestionMicros += delta;
    }
    if (node.callFrame.url.includes(OPTIQUE_FRAGMENT,)) {
      optiqueMicros += delta;
    }
    if (
      node.callFrame.url.includes(WRAPPER_SOURCE_FRAGMENT,)
      && !node.callFrame.url.includes(OPTIQUE_FRAGMENT,)
    ) {
      wrapperMicros += delta;
    }
  }

  /**
   * Formats a microsecond amount as milliseconds with a share of the parse.
   *
   * @param micros - Microseconds to report.
   *
   * @returns Report entry.
   */
  function share(micros: number,): { ms: number; shareOfParse: number; } {
    return {
      ms: Number((micros / 1000).toFixed(3),),
      shareOfParse: parseMicros === 0 ? 0 : Number((micros / parseMicros).toFixed(4),),
    };
  }

  return {
    profilePath,
    samplingIntervalMs: Number(samplingIntervalMs.toFixed(4),),
    samples: profile.samples.length,
    totalMs: Number(totalMs.toFixed(3),),
    parseMs: Number((parseMicros / 1000).toFixed(3),),
    parseSamples,
    parseShareOfProfile: totalMs === 0 ? 0 : Number((((parseMicros / 1000)) / totalMs).toFixed(4),),
    parserEntryFrames: parseRootIds.size,
    buckets: {
      suggestionOrEditDistance: share(suggestionMicros,),
      optiqueLibrary: share(optiqueMicros,),
      wrapperSources: share(wrapperMicros,),
    },
    topFunctions: [...selfByFunction.values(),]
      .sort(function byMicros(left, right,) {
        return right.micros - left.micros;
      },)
      .slice(
        0,
        20,
      )
      .map(function format(entry,) {
        return {
          function: entry.name,
          file: entry.file,
          ...share(entry.micros,),
        };
      },),
    topFiles: [...selfByFile.entries(),]
      .sort(function byMicros(left, right,) {
        return right[1] - left[1];
      },)
      .slice(
        0,
        12,
      )
      .map(function format(entry,) {
        return {
          file: entry[0],
          ...share(entry[1],),
        };
      },),
  };
}

//endregion Analysis

//region Entry point

/**
 * Analyses the profile named by `--file=` and prints the report.
 */
function main(): void {
  /** Profile path from the command line. */
  const fileArgument = process.argv
    .slice(2,)
    .find(function isFile(argument,) {
      return argument.startsWith('--file=',);
    },);
  /** Root shape from the command line. */
  const rootArgument = process.argv
    .slice(2,)
    .find(function isRoot(argument,) {
      return argument.startsWith('--root=',);
    },);

  if (fileArgument === undefined) {
    throw new Error('missing --file=<path.cpuprofile>',);
  }

  /** Which entry frame marks the parse. */
  const root = rootArgument?.slice('--root='.length,) === 'parseSync' ? 'parseSync' : 'commit';

  process.stdout.write(`${
    JSON.stringify(
      analyse(
        fileArgument.slice('--file='.length,),
        root,
      ),
    )
  }\n`,);
}

main();

//endregion Entry point
