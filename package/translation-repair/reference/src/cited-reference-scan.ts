import { nextSchemeStart, } from './scheme-start-scan.ts';

//region Cited-reference scan
// The pages an original links, read off its `http://` and `https://` spans.
// CLASS THIRTY-FIVE (2026-09-16): the archive's human translator wrote "older
// sister who is also trans" from the blog the original cites, four critics
// filed it as accuracy/addition on absence alone, and the editor deleted a
// true fact. The owner's answer: fetch what the original links and show it
// to the critics and the panel, so "not in the source" can be answered.
//
// PURE TEXT, no web and no disk; `cited-reference-lookup.ts` buys the pages.
// Measured over the pinned corpus on 2026-09-16: 59 of 92 originals link
// somewhere, 116 links in all, twitter and github the commonest hosts.

/**
 Most references bought for one entry, in order of citation. Bounds what a
 sheet carries: at `REFERENCE_TEXT_CHARACTERS` each this is the whole budget.
 */
export const MAX_CITED_REFERENCES = 8;

/**
 Characters a link never runs into: whitespace and the Markdown, bracket and
 quote delimiters an original wraps a link in.
 */
const URL_STOPS: ReadonlySet<string> = new Set([
  ' ',
  '\n',
  '\t',
  '\r',
  ')',
  '）',
  '(',
  '（',
  '>',
  '<',
  '"',
  '\'',
  '`',
  '[',
  ']',
  '\\',
],);

/**
 Punctuation a sentence leaves after a link, which is not part of it.
 */
const TRAILING_PUNCTUATION: ReadonlySet<string> = new Set([
  '.',
  ',',
  ';',
  ':',
  '!',
  '?',
  '。',
  '，',
  '；',
  '：',
  '！',
  '？',
  '」',
  '』',
  '”',
],);

/**
 Hosts of the corpus itself, whose pages are the very documents under
 repair and are read from disk, never fetched.
 */
const OWN_HOSTS: ReadonlySet<string> = new Set([
  'one-among.us',
  'www.one-among.us',
],);

/**
 Hosts where a one-segment path is a person's profile, which says nothing
 about the entry: measured on the Mio probe of 2026-09-16, a GitHub user
 page came back as repository names and a bilibili space as page chrome.
 */
const PROFILE_HOSTS: ReadonlySet<string> = new Set([
  'github.com',
  'twitter.com',
  'www.twitter.com',
  'x.com',
  'www.x.com',
  't.me',
],);

/**
 Hosts that serve nothing but profiles.
 */
const PROFILE_ONLY_HOSTS: ReadonlySet<string> = new Set(['space.bilibili.com',],);

/**
 Path prefix of a profile on zhihu.
 */
const ZHIHU_PEOPLE = '/people/';

/**
 Whether a parsed link is a person's profile rather than a page that says
 something.

 @param link - parsed link

 @returns Whether it is a profile

 @example
 ```ts
 isProfile({ link: new URL('https://github.com/someone',), },);
 // => true
 ```
 */
function isProfile({ link, }: { readonly link: URL; },): boolean {
  /**
   Host and path of the link.
   */
  const {
    hostname,
    pathname,
  } = link;
  if (PROFILE_ONLY_HOSTS.has(hostname,))
    return true;
  if (hostname.endsWith('zhihu.com',) && pathname.startsWith(ZHIHU_PEOPLE,))
    return true;
  if (!PROFILE_HOSTS.has(hostname,))
    return false;
  /**
   Path segments with content.
   */
  const segments = pathname
    .split('/',)
    .filter(function hasContent(segment,): boolean {
      return segment !== '';
    },);
  return segments.length <= 1;
}

/**
 Whether a link is left out rather than bought: the corpus's own site, a
 person's profile, or a link that does not parse at all.

 @param url - link as written

 @returns Whether it points into the corpus site, at a profile, or nowhere

 @example
 ```ts
 isLeftOut({ url: 'https://one-among.us/people/x', },);
 // => true
 ```
 */
function isLeftOut({ url, }: { readonly url: string; },): boolean {
  if (!URL.canParse(url,))
    return true;
  /**
   Parsed link.
   */
  const link = new URL(url,);
  return OWN_HOSTS.has(link.hostname,) || isProfile({ link, },);
}

/**
 Index of the first stop character at or after a position, or the text's
 end.

 @param text - original document

 @param start - where the link begins

 @returns Exclusive end of the link's run

 @example
 ```ts
 stopFrom({ text: 'https://a.example/x more', start: 0, },);
 // => 19
 ```
 */
function stopFrom(
  {
    text,
    start,
  }: {
    readonly text: string;
    readonly start: number;
  },
): number {
  for (let end = start; end < text.length; end += 1) {
    if (URL_STOPS.has(text.slice(
      end,
      end + 1,
    ),))
      return end;
  }
  return text.length;
}

/**
 Link with the sentence punctuation a writer left after it removed.

 @param url - link run as cut at its stop

 @returns Link without trailing punctuation

 @throws Error when `url` is all trailing punctuation, which no caller
 passes: `citedReferenceUrlsOf` cuts every run where `nextSchemeStart` found
 a scheme, so each opens with the scheme's letters

 @example
 ```ts
 withoutTrailingPunctuation({ url: 'https://a.example/x.', },);
 // => 'https://a.example/x'
 ```
 */
function withoutTrailingPunctuation({ url, }: { readonly url: string; },): string {
  for (let cut = url.length; cut > 0; cut -= 1) {
    if (!TRAILING_PUNCTUATION.has(url.slice(
      cut - 1,
      cut,
    ),))
      return url.slice(
        0,
        cut,
      );
  }
  throw new Error(
    `unreachable: ${JSON.stringify(url,)} is entirely trailing punctuation, but every link run opens with `
      + 'the scheme\'s letters',
  );
}

/**
 Every page an original links, once each, in order of first citation, the
 corpus's own pages and people's profiles left out, at most
 `MAX_CITED_REFERENCES`.

 ONE LINEAR PASS: `nextSchemeStart` finds where each link begins, the link is
 read to its first stop character, and the search resumes there. A run that
 consumed nothing cannot leave the cursor standing, since
 `withoutTrailingPunctuation` throws on the empty run before the cursor is
 read again.

 @param text - original document

 @returns Links as the original writes them, trailing punctuation removed

 @example
 ```ts
 citedReferenceUrlsOf({ text: '参考链接：[博客](https://a.example/post)（[存档](https://web.archive.org/web/1/https://a.example/post)）', },);
 // => ['https://a.example/post', 'https://web.archive.org/web/1/https://a.example/post']
 ```
 */
export function citedReferenceUrlsOf(
  { text, }: { readonly text: string; },
): readonly string[] {
  /**
   Links found so far, in order.
   */
  const found: string[] = [];
  /**
   Links already taken, for the once-each rule.
   */
  const taken = new Set<string>();
  for (
    let cursor = nextSchemeStart({
      text,
      from: 0,
    },);
    (cursor < text.length) && (found.length < MAX_CITED_REFERENCES);
  ) {
    /**
     Exclusive end of the link's run.
     */
    const end = stopFrom({
      text,
      start: cursor,
    },);
    /**
     Link as the original writes it.
     */
    const url = withoutTrailingPunctuation({
      url: text.slice(
        cursor,
        end,
      ),
    },);
    /**
     Whether this link is new and a page worth buying.
     */
    const wanted = (!taken.has(url,)) && (!isLeftOut({ url, },));
    if (wanted) {
      taken.add(url,);
      found.push(url,);
    }
    cursor = nextSchemeStart({
      text,
      from: end,
    },);
  }
  return found;
}

//endregion Cited-reference scan
