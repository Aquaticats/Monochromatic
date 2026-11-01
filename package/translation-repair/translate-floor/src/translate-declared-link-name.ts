import {
  carriesHandleToken,
  isHandleCharacter,
  standsAsHandle,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  carriesName,
  type DeclaredNamePair,
  nameProjection,
  projectName,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  type Link,
  linksOf,
} from '@monochromatic-dev/translation-repair-rendering-audit/ts';

//region Declared link name floor
// THE ONE HUNDRED FOURTEENTH CLASS (yingying9, 2026-09-24). The page-name
// glossary tells every sheet that a declared name inside a linked title takes
// its declared form (class eighty-six), and the bench overruled it: two of
// three translate judges chose the archive's own "Farewell. I miss you,
// Sakura." as "the archive's established rendering for this link text", and
// the repair lane's rewrite to "Yingying" was reverted by an introduced-defect
// probe calling the declared form a change. yingying4 to 8 had written the
// declared form on the judges' own call. A rule the judges read and overrule
// is a floor, the same step classes eighty and ninety-seven took: where the
// original's link text carries a name the front matter declares, the
// rendering's link text under the same href carries the declared form.
//
// COMPARED ON THE NAME PROJECTION (letters and digits, lowercased) that the
// declared-name survival guard uses, so `Mittens'` or an escaped underscore
// is still the name, and read as a name rather than as letters, so `tomcat`
// carries no `Tom` (`name-projection.ts`, ledger B23). Silent without
// declared pairs, where the original's link names nobody declared, and where
// the rendering carries no link under that href: a dropped link is the link
// floor's finding, not this one's.
//
// THE ORIGINAL'S LINK TEXT AND ITS MENTIONS ARE READ AT HANDLE EDGES (ledger
// B23, `handle-token.ts`). A Latin source form names the person only where no
// handle character runs on from it, so `Tomcat` and `@Tom_Cat` name no `Tom`,
// and a page handle is carried only whole, so `@mi-mi-420` does not carry
// `@mi-mi-42`.

// AN @-MENTION MAY CARRY THE ACCOUNT HANDLE (owner, 2026-09-27, "Account
// handle"). Zhihu question titles @-mention the entry's subject by display
// name (如何评价知乎用户@倉山静葉？), and every archive writes the account's
// handle there (@Cang_Shan_Jing_Ye), the name that finds the account. Where
// the original's link text @-mentions the declared person, a rendering's link
// text under the same href passes carrying either the declared form or an
// @-handle the page writes under that href.

/**
 Mark an account mention opens with.
 */
const MENTION_MARK = '@';

/**
 Offset just past the handle that starts at an offset.

 @param text - link text

 @param from - first offset after the mention mark

 @returns First offset that does not continue the handle

 @example
 ```ts
 handleEnd({ text: '@mao-xj ?', from: 1, },); // 7
 ```
 */
function handleEnd(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): number {
  for (let at = from; at < text.length; at += 1) {
    if (!isHandleCharacter({ character: text.charAt(at,), },))
      return at;
  }
  return text.length;
}

/**
 Account handles a link text writes after the mention mark, each with the
 mark, in order.

 @param text - link text

 @returns Handles such as `@Cang_Shan_Jing_Ye`

 @example
 ```ts
 mentionHandles({ text: 'Zhihu user @mao-xj ?', },); // ['@mao-xj']
 ```
 */
function mentionHandles({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Handles found so far.
   */
  const handles: string[] = [];
  for (
    let at = text.indexOf(MENTION_MARK,);
    at !== (-1);
    at = text.indexOf(
      MENTION_MARK,
      at + 1,
    )
  ) {
    /**
     Offset just past this handle.
     */
    const end = handleEnd({
      text,
      from: at + 1,
    },);
    if (end > (at + 1)) {
      handles.push(text.slice(
        at,
        end,
      ),);
    }
  }
  return handles;
}

/**
 Whether a link text @-mentions a form: the mention mark, any spaces, then
 the form, with no handle running on from it (`@Tomcat` mentions no `Tom`).

 @param text - link text

 @param form - declared source form

 @returns Whether some mention names the form

 @example
 ```ts
 mentionsForm({ text: '如何评价知乎用户 @猫小姐？', form: '猫小姐', },); // true
 ```
 */
function mentionsForm(
  {
    text,
    form,
  }: {
    readonly text: string;
    readonly form: string;
  },
): boolean {
  for (
    let at = text.indexOf(MENTION_MARK,);
    at !== (-1);
    at = text.indexOf(
      MENTION_MARK,
      at + 1,
    )
  ) {
    /**
     Text after the mark.
     */
    const rest = text.slice(at + 1,);

    /**
     Text after the mark from the mentioned name on, past any spaces.
     */
    const named = rest.trimStart();

    /**
     Where the mentioned name starts.
     */
    const nameAt = (at + 1) + (rest.length - named.length);
    if (text.startsWith(
      form,
      nameAt,
    ) && standsAsHandle({
      text,
      at: nameAt,
      needle: form,
    },))
      return true;
  }
  return false;
}

/**
 One source link naming a declared person, with the pair it names.
 */
type NamingLink = {
  /**
   Link as the original writes it.
   */
  readonly link: Link;

  /**
   Declared pair the link text names.
   */
  readonly pair: DeclaredNamePair;
};

/**
 Source links whose text carries a declared source form, one entry per
 link and pair.

 @param sourceText - original slice

 @param declared - name pairs the front matter declares

 @returns Naming links in source order

 @example
 ```ts
 const naming = namingLinks({ sourceText, declared, },);
 ```
 */
function namingLinks(
  {
    sourceText,
    declared,
  }: {
    readonly sourceText: string;
    readonly declared: readonly DeclaredNamePair[];
  },
): readonly NamingLink[] {
  return linksOf({ text: sourceText, },)
    .flatMap(function named(link,): readonly NamingLink[] {
      return declared
        .filter(function carried(pair,): boolean {
          return carriesHandleToken({
            text: link.text,
            needle: pair.source,
          },);
        },)
        .map(function paired(pair,): NamingLink {
          return {
            link,
            pair,
          };
        },);
    },);
}

/**
 Findings for every link the rendering carries under a naming link's href
 without the declared form.

 @param sourceText - original slice

 @param candidateText - proposed rendering of it

 @param declared - name pairs the front matter declares, none when the page
 declares no name on both sides

 @param pageText - text the rendering would replace, read for the account
 handles an @-mention may carry instead; empty where the page has none

 @returns One finding per naming link rendered without its declared form,
 written for the model that wrote the rendering

 @example
 ```ts
 const findings = declaredLinkNameFindings({ sourceText, candidateText, declared, pageText, },);
 ```
 */
export function declaredLinkNameFindings(
  {
    sourceText,
    candidateText,
    declared,
    pageText = '',
  }: {
    readonly sourceText: string;
    readonly candidateText: string;
    readonly declared: readonly DeclaredNamePair[];
    readonly pageText?: string;
  },
): readonly string[] {
  if (declared.length === 0)
    return [];
  /**
   Links the rendering carries.
   */
  const rendered = linksOf({ text: candidateText, },);
  /**
   Links the page carries, read for the handles an @-mention may carry.
   */
  const paged = linksOf({ text: pageText, },);
  return namingLinks({
    sourceText,
    declared,
  },)
    .flatMap(function checked(
      {
        link,
        pair,
      },
    ): readonly string[] {
      /**
       Rendered link texts under this href.
       */
      const texts = rendered
        .filter(function sameHref(candidate,): boolean {
          return candidate.href === link.href;
        },)
        .map(function textOf(candidate,): string {
          return candidate.text;
        },);
      /**
       Declared form as the comparison reads it.
       */
      const declaredKey = nameProjection({ text: pair.rendering, },);
      /**
       Account handles the page writes under this href, which an @-mention of
       the declared person may carry instead; none off a mention.
       */
      const handles = mentionsForm({
        text: link.text,
        form: pair.source,
      },)
        ? paged
          .filter(function sameHref(page,): boolean {
            return page.href === link.href;
          },)
          .flatMap(function handlesOf(page,): readonly string[] {
            return mentionHandles({ text: page.text, },);
          },)
        : [];
      /**
       Whether some rendered link text carries the declared form.
       */
      const carriesDeclared = texts.some(function declaredIn(text,): boolean {
        return carriesName({
          projection: projectName({ text, },),
          key: declaredKey,
        },);
      },);
      /**
       Whether some rendered link text carries a handle the page writes, whole.
       */
      const carriesHandle = texts.some(function handleIn(text,): boolean {
        return handles.some(function carried(handle,): boolean {
          return carriesHandleToken({
            text,
            needle: handle,
          },);
        },);
      },);
      /**
       Whether the rendering carries no link under this href at all, which
       the link floor names rather than this one.
       */
      const unrendered = (texts.length === 0);
      /**
       Whether nothing here is this floor's to refuse.
       */
      const passes = unrendered
        || carriesDeclared
        || carriesHandle;
      if (passes)
        return [];
      /**
       Rendered link texts as the finding quotes them.
       */
      const quoted = texts.join('", "',);
      /**
       Handle alternative the finding offers, empty off a mention.
       */
      const handleClause = (handles.length === 0)
        ? ''
        : ` As an @-mention it may instead carry the account handle the existing translation writes there (${
          handles.join(', ',)
        }).`;
      return [
        `The link text for ${link.href} names ${pair.source}, whom this page's front matter declares "${pair.rendering}", but your link text "${quoted}" does not carry "${pair.rendering}". A declared name inside a title takes its declared form, whatever the existing translation calls the person there; keep the rest of the title as you rendered it.${handleClause}`,
      ];
    },);
}

//endregion Declared link name floor
