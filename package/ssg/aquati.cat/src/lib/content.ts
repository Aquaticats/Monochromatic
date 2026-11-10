/**
 Content loading and organization for MDX blog posts.
 
 Globs MDX files from `src/content/{lang}/`, parses YAML frontmatter with
 an inline parser backed by the `yaml` package, validates with Zod, and
 provides grouping utilities.
 Filesystem paths give `lang` and `name` directly; no string splitting needed.
 
 Uses a custom frontmatter parser instead of `gray-matter` because `gray-matter`
 pulls in a large dependency tree and uses `eval` internally (via `js-yaml`'s
 unsafe loading), which is both a security concern and unnecessary for trusted
 YAML-only frontmatter.
 */
import { readFile, } from 'node:fs/promises';
import {
  basename,
  dirname,
} from 'node:path';

import readdir from 'tiny-readdir-glob';
import * as v from 'valibot';

import {
  isLocale,
  type Locale,
} from '../i18n/index.ts';

import { sha256, } from './cache-hash.ts';
import {
  parseFrontmatter,
  type ParsedFrontmatter,
} from './frontmatter.ts';
import {
  readAuthoredDates,
  type AuthoredDateFields,
} from './frontmatter-dates.ts';


//region Schema

/**
 Valibot schema for the author-written portion of MDX post frontmatter.
 
 Covers only the fields the human writes in the YAML header.
 Publication and update dates are derived from git history at build
 time rather than stored in frontmatter; see `./git-dates.ts`.
 */
export const postFileFrontmatterSchema: v.GenericSchema<{
  title: string;
  description: string;
  tags: string[];
}> = v.object({
  title: v.string(),
  description: v.string(),
  tags: v.array(v.string(),),
},);

/**
 Coerces string, number, or Date inputs into a Date instance.
 Lets the same schema validate both native `Date` objects (from freshly
 derived dates) and ISO strings (from JSON-deserialized cache entries).
 */
const coerceDateSchema = v.pipe(
  v.union([
    v.string(),
    v.number(),
    v.date(),
  ],),
  v.transform(function toDate(input,) {
    return new Date(input,);
  },),
  v.date(),
);

/**
 Valibot schema for the fully-resolved post frontmatter used downstream
 (rendering, RSS, sort keys).
 
 Combines the author-written file schema with `published`/`updated`
 dates resolved by `./git-dates.ts`. {@link coerceDateSchema} accepts
 both native `Date` objects (from freshly derived dates) and ISO
 strings (from JSON-deserialized cache entries).
 */
export const postFrontmatterSchema: v.GenericSchema<
  {
    title: string;
    description: string;
    tags: string[];
    published: string | number | Date;
    updated: string | number | Date;
  },
  {
    title: string;
    description: string;
    tags: string[];
    published: Date;
    updated: Date;
  }
  > = v.object({
    title: v.string(),
    description: v.string(),
    tags: v.array(v.string(),),
    published: coerceDateSchema,
    updated: coerceDateSchema,
  },);

//endregion Schema

//region Diagnostics

/**
 Names the field and the problem of one validation issue.
 
 @param issue - valibot issue to describe
 
 @returns dotted field path plus the issue message
 
 @example
 ```ts
 describeIssue({ path: [{ key: 'title' }], message: 'Invalid type' });
 // 'title: Invalid type'
 ```
 */
function describeIssue(issue: v.BaseIssue<unknown>,): string {
  /**
   Dotted field path of the issue, or a generic name for an issue at the root.
   */
  const path = (issue.path ?? [])
    .map(function keyOf(
      item: v.IssuePathItem,
      index: number,
    ) {
      return 'key' in item ? String(item.key,) : String(index,);
    },)
    .join('.',);
  return `${path === '' ? 'frontmatter' : path}: ${issue.message}`;
}

/**
 Describes a frontmatter failure with the MDX file it came from.
 
 @param detail - what went wrong, in the failing library's own words
 
 @param filePath - MDX file the frontmatter came from
 
 @returns diagnostic naming the file and what to change
 */
function frontmatterMessage(
  {
    detail,
    filePath,
  }: {
    readonly detail: string;
    readonly filePath: string;
  },
): string {
  return `Frontmatter of ${filePath} could not be read: ${detail}. Fix that file's frontmatter block; git history remains the source of truth for rendered dates.`;
}

/**
 Builds an error that names the MDX file a frontmatter failure came from.
 
 A bare valibot or YAML error names only a field or a YAML line, which leaves
 the operator bisecting the content directory by hand.
 
 @param error - failure raised by the YAML parser or the frontmatter schema
 
 @param filePath - MDX file the frontmatter came from
 
 @returns error naming the file, the field, and the underlying cause
 
 @example
 ```ts
 throw frontmatterError({ error, filePath: 'src/content/en/post.mdx' });
 ```
 */
function frontmatterError(
  {
    error,
    filePath,
  }: {
    readonly error: unknown;
    readonly filePath: string;
  },
): Error {
  if (v.isValiError(error,)) {
    return new Error(
      frontmatterMessage({
        detail: error.issues
          .map(describeIssue,)
          .join('; ',),
        filePath,
      },),
      { cause: error, },
    );
  }

  if (Error.isError(error,)) {
    return new Error(
      frontmatterMessage({
        detail: error.message,
        filePath,
      },),
      { cause: error, },
    );
  }

  return new Error(
    frontmatterMessage({
      detail: 'an unrecognized value was thrown',
      filePath,
    },),
    { cause: error, },
  );
}

/**
 Parses frontmatter, naming the file when the YAML is unparsable.
 
 @param raw - full file content including frontmatter
 
 @param filePath - MDX file the content came from
 
 @returns parsed YAML data and remaining body
 
 @throws when the YAML block cannot be parsed
 */
function parseFrontmatterForFile(
  {
    raw,
    filePath,
  }: {
    readonly raw: string;
    readonly filePath: string;
  },
): ParsedFrontmatter {
  try {
    return parseFrontmatter(raw,);
  }
  catch (error) {
    throw frontmatterError({
      error,
      filePath,
    },);
  }
}

/**
 Validates author-written frontmatter, naming the file when a field is wrong.
 
 @param rawData - parsed YAML frontmatter record
 
 @param filePath - MDX file the frontmatter came from
 
 @returns validated author-written frontmatter
 
 @throws when a required field is missing or mistyped
 */
function validateFrontmatter(
  {
    rawData,
    filePath,
  }: {
    readonly rawData: Readonly<Record<string, unknown>>;
    readonly filePath: string;
  },
): LoadedPost['fileData'] {
  try {
    return v.parse(
      postFileFrontmatterSchema,
      rawData,
    );
  }
  catch (error) {
    throw frontmatterError({
      error,
      filePath,
    },);
  }
}

//endregion Diagnostics

//region Types

/**
 Fully-resolved frontmatter as carried by `Post.data`.
 */
export type PostFrontmatter = {
  readonly title: string;
  readonly description: string;
  readonly tags: readonly string[];
  readonly published: Date;
  readonly updated: Date;
};

/**
 Post loaded from disk with author-written frontmatter validated,
 but `published`/`updated` not yet resolved.
 
 Returned by {@link loadContent}. Downstream consumers receive fully-resolved
 {@link Post} objects built by {@link attachDates} after git dates are derived.
 */
export type LoadedPost = {
  /**
   Two-letter language code derived from parent directory name, validated against known locales.
   */
  readonly lang: Locale;
  /**
   Post slug derived from filename without extension.
   */
  readonly name: string;
  /**
   Validated author-written frontmatter (title, description, tags).
   */
  readonly fileData: {
    readonly title: string;
    readonly description: string;
    readonly tags: readonly string[];
  };
  /**
   Optional human-authored date fields retained only for divergence warnings.
   */
  readonly authoredDates: AuthoredDateFields;
  /**
   Raw MDX body content (frontmatter stripped).
   */
  readonly body: string;
  /**
   Absolute path to the source MDX file.
   */
  readonly filePath: string;
  /**
   SHA-256 hex digest of the raw file contents, computed during loading.
   */
  readonly contentHash: string;
};

/**
 Blog post with fully-resolved frontmatter (author-written + git-derived dates).
 */
export type Post = {
  /**
   Two-letter language code derived from parent directory name, validated against known locales.
   */
  readonly lang: Locale;
  /**
   Post slug derived from filename without extension.
   */
  readonly name: string;
  /**
   Fully-resolved frontmatter data.
   */
  readonly data: PostFrontmatter;
  /**
   Raw MDX body content (frontmatter stripped).
   */
  readonly body: string;
  /**
   Absolute path to the source MDX file.
   */
  readonly filePath: string;
  /**
   SHA-256 hex digest of the raw file contents, computed during loading.
   */
  readonly contentHash: string;
};

//endregion Types

//region Loading

/**
 Loads all MDX posts from the content directory.
 
 Globs `src/content` for `.mdx` files, extracts `lang` from the parent
 directory name and `name` from the filename (minus extension).
 Does not resolve `published`/`updated`: those are derived from git
 separately by `./git-dates.ts` and attached via `attachDates`.
 
 @param contentDir - path to content directory containing `{lang}/*.mdx`
 
 @returns array of loaded posts awaiting date resolution
 
 @throws on frontmatter validation failure, naming the offending MDX file
 
 @example
 ```ts
 const loaded = await loadContent('src/content');
 ```
 */
export async function loadContent(contentDir: string,): Promise<LoadedPost[]> {
  /**
   Glob expansion result; `.files` holds the matched paths used downstream.
   */
  const result = await readdir(`${contentDir}/**/*.mdx`,);
  /**
   MDX file paths feeding the per-file parse fan-out.
   */
  const filePaths = result.files;

  return Promise.all(
    filePaths.map(async function parsePost(filePath,) {
      /**
       Raw file text used for both hashing and frontmatter parsing.
       */
      const raw = await readFile(
        filePath,
        'utf8',
      );
      /**
       Content-addressed hash used as the cache invalidation key.
       */
      const contentHash = sha256(raw,);
      /**
       Destructured parse result; renamed fields disambiguate from outer post data.
       */
      const {
        data: rawData,
        content: body,
      } = parseFrontmatterForFile({
        raw,
        filePath,
      },);
      /**
       Schema-validated frontmatter fields authored in the MDX file.
       */
      const fileData = validateFrontmatter({
        rawData,
        filePath,
      },);
      /**
       Optional date fields retained for warnings when they contradict git history.
       */
      const authoredDates = readAuthoredDates({
        rawData,
        filePath,
      },);
      /**
       Locale segment of the file path before narrowing to the {@link Locale} type.
       */
      const rawLang = basename(dirname(filePath,),);
      if (!isLocale(rawLang,)) {
        throw new Error(
          `Unknown locale "${rawLang}" for ${filePath}. Expected one of the configured locales.`,
        );
      }
      /**
       Narrowed locale used as the post `lang`.
       */
      const lang: Locale = rawLang;
      /**
       Slug name derived from the filename minus the `.mdx` extension.
       */
      const name = basename(
        filePath,
        '.mdx',
      );

      return {
        lang,
        name,
        fileData,
        authoredDates,
        body,
        filePath,
        contentHash,
      };
    },),
  );
}

/**
 Derived dates for one post, keyed by absolute file path in `attachDates`.
 */
export type ResolvedDates = {
  readonly published: Date;
  readonly updated: Date;
};

/**
 Combines loaded posts with resolved git dates and returns sorted `Post[]`.
 
 The caller (typically `build.ts`) is responsible for producing the
 `datesByFilePath` map; either from cache (when HEAD is unchanged)
 or by calling {@link getPostDates} for files missing from the cache.
 
 Posts are returned sorted by `updated` descending, matching the previous
 behavior of the combined loader.
 
 @param loadedPosts - posts produced by `loadContent`
 
 @param datesByFilePath - map from absolute file path to resolved dates
 
 @returns fully-resolved posts, sorted by `updated` descending
 
 @throws when a loaded post has no entry in `datesByFilePath`
 
 @example
 ```ts
 const posts = attachDates({ loadedPosts, datesByFilePath });
 ```
 */
export function attachDates(
  {
    loadedPosts,
    datesByFilePath,
  }: {
    readonly loadedPosts: readonly LoadedPost[];
    readonly datesByFilePath: ReadonlyMap<string, ResolvedDates>;
  },
): Post[] {
  /**
   Hydrated posts assembled before the stable multi-key sort.
   */
  const posts: Post[] = loadedPosts.map(function toPost(lp,) {
    /**
     Resolved dates for this post; missing entries indicate a caller bug.
     */
    const dates = datesByFilePath.get(lp.filePath,);
    if (dates === undefined) {
      throw new Error(
        `Missing resolved dates for ${lp.filePath}. Caller must populate datesByFilePath for every loaded post.`,
      );
    }
    return {
      lang: lp.lang,
      name: lp.name,
      data: {
        title: lp.fileData
          .title,
        description: lp.fileData
          .description,
        tags: lp.fileData
          .tags,
        published: dates.published,
        updated: dates.updated,
      },
      body: lp.body,
      filePath: lp.filePath,
      contentHash: lp.contentHash,
    };
  },);

  /* Two posts often share an `updated` timestamp because a single commit
   * (e.g., a monorepo-wide formatting sweep) touched multiple files with
   * the same author date. Fall back to `published` desc, then to `name`,
   * so the order stays stable and meaningful across builds. */
  return posts.toSorted(function byUpdatedThenPublishedThenName(
    a,
    b,
  ) {
    /**
     Primary sort key in milliseconds; descending.
     */
    const updatedDelta = b.data
      .updated
      .getTime()
      - a
        .data
        .updated
        .getTime();
    if (updatedDelta !== 0)
      return updatedDelta;
    /**
     Tie-break used when many posts share the same updated timestamp.
     */
    const publishedDelta = b.data
      .published
      .getTime()
      - a
        .data
        .published
        .getTime();
    if (publishedDelta !== 0)
      return publishedDelta;
    return a.name
      .localeCompare(b.name,);
  },);
}

//endregion Loading
