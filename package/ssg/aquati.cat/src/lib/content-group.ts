/**
 Post grouping and filtering utilities.
 
 Provides functions to organize posts by language, slug name, and tag.
 Used by the build pipeline to generate per-language and per-tag pages.
 */
// File justification: 104 lines; grouping functions share the same type
// and patterns; splitting by-lang/by-name from by-tag would break cohesion.
import type { Locale, } from '../i18n/index.ts';

import type { Post, } from './content.ts';

//region By language and name

/**
 Returns locale grouping key for one post.
 
 @param post - post to group
 
 @returns post locale
 */
function postLanguage(post: Post,): Locale {
  return post.lang;
}

/**
 Returns slug grouping key for one post.
 
 @param post - post to group
 
 @returns post slug
 */
function postName(post: Post,): string {
  return post.name;
}

/**
 Groups posts by one primitive key while preserving encounter order.
 
 @param posts - posts to group
 
 @param keyForPost - key selector
 
 @returns mutable buckets in encounter order
 */
function groupPostsBy<const Key,>({
  posts,
  keyForPost,
}: {
  readonly posts: readonly Post[];
  readonly keyForPost: (post: Post,) => Key;
}): Map<Key, Post[]> {
  /**
   Encounter-ordered groups under construction.
   */
  const groups = new Map<Key, Post[]>();
  for (const post of posts) {
    /**
     Existing bucket for current post key.
     */
    const key = keyForPost(post,);
    /**
     Existing group for current key.
     */
    const existing = groups.get(key,);
    if (existing === undefined) {
      groups.set(
        key,
        [post,],
      );
      continue;
    }
    existing.push(post,);
  }
  return groups;
}

/**
 Groups posts by language code.
 
 @param posts - all loaded posts
 
 @returns map from language codes to their posts; absent locales have no key rather than an empty bucket
 
 @example
 ```ts
 const byLang = groupByLang(posts);
 // Map { 'en' => [...], 'fr' => [...] }
 ```
 */
export function groupByLang(posts: readonly Post[],): ReadonlyMap<Locale, Post[]> {
  return groupPostsBy({
    posts,
    keyForPost: postLanguage,
  },);
}

/**
 Groups posts by slug name across all languages.
 
 @param posts - all loaded posts
 
 @returns record mapping post names to all language variants
 
 @example
 ```ts
 const byName = groupByName(posts);
 // { 'hello-world': [enPost, frPost] }
 ```
 */
export function groupByName(posts: readonly Post[],): Record<string, Post[]> {
  return Object.fromEntries(groupPostsBy({
    posts,
    keyForPost: postName,
  },),);
}

//endregion By language and name

//region By tag

/**
 Extracts all unique tags across all posts.
 
 @param posts - all loaded posts
 
 @returns deduplicated array of tag strings
 
 @example
 ```ts
 const tags = allTags(posts);
 // ['typescript', 'css', 'web']
 ```
 */
function allTags(posts: readonly Post[],): string[] {
  return [...new Set(posts.flatMap(function getTags(post,) {
    return post.data
      .tags;
  },),),];
}

/**
 Groups posts by tag, where each tag maps to its matching posts.
 
 @param posts - all loaded posts
 
 @returns record mapping tags to posts containing that tag
 
 @example
 ```ts
 const byTag = groupByTag(posts);
 // { typescript: [...], css: [...] }
 ```
 */
export function groupByTag(posts: readonly Post[],): Record<string, Post[]> {
  /**
   Distinct tag universe computed once so the inner filter loop only iterates the set, not duplicates.
   */
  const tags = allTags(posts,);
  return Object.fromEntries(
    tags.map(function tagEntry(tag,) {
      return [
        tag,
        posts.filter(function hasTag(post,) {
          return post.data
            .tags
            .includes(tag,);
        },),
      ];
    },),
  );
}

//endregion By tag
