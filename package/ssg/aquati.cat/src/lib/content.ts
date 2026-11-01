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
import { parse as parseYaml, } from 'yaml';

import {
  isLocale,
  type Locale,
} from '../i18n/index.ts';

import { sha256, } from './cache-hash.ts';
import {
  readAuthoredDates,
  type AuthoredDateFields,
} from './frontmatter-dates.ts';

//region Frontmatter

/**
 Opening delimiter for YAML frontmatter blocks.
 */
const FRONTMATTER_OPEN = '---';

/**
 Unicode Byte Order Mark code point, stripped from file content before parsing.
 */
const BOM = 0xFE_FF;

/**
 Parses YAML frontmatter delimited by `---` from a raw string.
 
 Splits at the first two `---` lines to extract the YAML block and body.
 Returns empty data when no valid frontmatter is found.
 
 @param raw - full file content including frontmatter
 
 @returns parsed YAML data and remaining body content
 
 @example
 ```ts
 const { data, content } = parseFrontmatter('---\ntitle: Hello\n---\nbody');
 // data = { title: 'Hello' }, content = 'body'
 ```
 */
function parseFrontmatter(raw: string,): {
  data: Record<string, unknown>;
  content: string;
} {
  /* Strip optional leading BOM. */
  /**
   BOM-trimmed input used for every subsequent index computation.
   */
  const str = raw.codePointAt(0,)
    === BOM ? raw.slice(1,) : raw;

  if (!str.startsWith(FRONTMATTER_OPEN,)) {
    return {
      data: {},
      content: str,
    };
  }

  /* Skip past the opening `---` and its trailing newline. */
  /**
   Index of the first newline after the opening fence, or `-1` when malformed.
   */
  const afterOpen = str.indexOf(
    '\n',
    FRONTMATTER_OPEN.length,
  );
  if (afterOpen === (-1)) {
    return {
      data: {},
      content: str,
    };
  }

  /**
   Scan for the closing `---` that sits at the start of a line.
   Start searching from the character right after the first newline.
   */
  /**
   Starting offset for the closing-fence scan; first character after the opening newline.
   */
  const searchFrom = afterOpen + 1;
  /**
   Cursor advanced through the loop while hunting for a column-zero closing fence.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-function-root-let -- parser cursor advanced across multiple loop iterations
  let closeStart = searchFrom;

  for (;;) {
    const idx = str.indexOf(
      FRONTMATTER_OPEN,
      closeStart,
    );
    if (idx === (-1)) {
      return {
        data: {},
        content: str,
      };
    }

    /* The delimiter must be at column 0 or immediately after a newline. */
    if ((idx === 0) || (str[idx - 1]
      === '\n')) {
      /**
       Offset just past the closing fence; the next char must be newline or EOF for a valid close.
       */
      const afterDelim = idx + FRONTMATTER_OPEN
        .length;

      /* Next char must be a newline or EOF for a valid closing fence. */
      if (
        (afterDelim === str
          .length)
          || (str[afterDelim]
            === '\n')
          || (str[afterDelim]
            === '\r')
      ) {
        /**
         YAML body between the opening and closing fences fed to {@link parseYaml}.
         */
        const yamlBlock = str.slice(
          searchFrom,
          idx,
        );
        /**
         Body start cursor advanced past CR/LF so the post body excludes the closing fence.
         */
        let bodyStart = afterDelim;
        if (str[bodyStart]
          === '\r')
          bodyStart += 1;
        if (str[bodyStart]
          === '\n')
          bodyStart += 1;

        return {
          // oxlint-disable-next-line no-unsafe-type-assertion -- `yaml`'s `parse` returns `any` for untyped frontmatter; the assertion narrows to the documented record contract, re-validated downstream by `postFileFrontmatterSchema` in `loadContent`.
          data: (parseYaml(yamlBlock,)
            ?? {}) as Record<string, unknown>,
          content: str.slice(bodyStart,),
        };
      }
    }

    closeStart = idx + 1;
  }
}

//endregion Frontmatter

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
 
 @throws on frontmatter validation failure
 
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
      } = parseFrontmatter(raw,);
      /**
       Schema-validated frontmatter fields authored in the MDX file.
       */
      const fileData = v.parse(
        postFileFrontmatterSchema,
        rawData,
      );
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
