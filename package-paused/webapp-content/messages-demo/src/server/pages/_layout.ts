/**
 * Shared HTML shell for the messages-demo pages.
 *
 * Builds the entire document via `h()` so every tag (including
 * `<body>`) is constructed by the hyperscript factory. Page handlers
 * pass their pre-built body HTML in via `renderPage({title, body, ...})`;
 * the layout wraps that string inside `<body><header>...</header><main>...</main><footer>composer</footer></body>`.
 */

import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

import { all, } from '../../lib/db.ts';

/**
 * A user record exposed in the identity dropdown.
 */
type SeedUser = {
  readonly id: string;
  readonly name: string;
};

/**
 * Loads the three seeded users for the identity dropdown. Cached at
 * module load; users are seeded once at migration time and never
 * change for the demo.
 */
const seedUsers: SeedUser[] = await loadUsers();

/**
 * One-shot loader called at module import; keeps the export simple.
 *
 * @returns user rows used by the identity dropdown
 *
 * @example
 * ```ts
 * const users = await loadUsers();
 * ```
 */
async function loadUsers(): Promise<SeedUser[]> {
  return await all<SeedUser>({
    sql: 'SELECT id, name FROM users ORDER BY name ASC',
  },);
}

/**
 * Returns the seeded user list (memoised). Used by handlers that mount
 * the composer.
 *
 * @returns identity dropdown options
 *
 * @example
 * ```ts
 * const users = getSeedUsers();
 * ```
 */
export function getSeedUsers(): readonly SeedUser[] {
  return seedUsers;
}

/**
 * Options accepted by `renderPage`.
 */
type PageOptions = {
  /**
   * Document `<title>`.
   */
  readonly title: string;
  /**
   * Pre-built HTML for `<main>`'s contents.
   */
  readonly body: string;
  /**
   * When set, the composer mounts in edit mode for this message.
   */
  readonly editMessageId?: number;
  /**
   * Tier hint for the client to skip its initial size probe.
   */
  // oxlint-disable-next-line eslint/no-magic-numbers -- tier discriminant
  readonly initialTier?: 1 | 2 | 3;
};

/**
 * Builds the `<head>` element.
 *
 * @param title - document title
 *
 * @returns rendered `<head>` HTML
 *
 * @example
 * ```ts
 * renderHead('messages-demo');
 * ```
 */
function renderHead(title: string,): string {
  return h({
    tag: 'head',
    children: [
      h({
        tag: 'meta',
        attrs: { charset: 'utf8', },
      },),
      h({
        tag: 'meta',
        attrs: {
          name: 'viewport',
          content: 'width=device-width, initial-scale=1',
        },
      },),
      h({
        tag: 'title',
        text: title,
      },),
      h({
        tag: 'link',
        attrs: {
          rel: 'stylesheet',
          href: '/dist/css/styles.css',
        },
      },),
      h({
        tag: 'script',
        attrs: {
          type: 'module',
          src: '/dist/client/index.js',
          defer: '',
        },
      },),
    ],
  },);
}

/**
 * Builds the site `<header>` element.
 *
 * @returns rendered `<header>` HTML
 *
 * @example
 * ```ts
 * renderSiteHeader();
 * ```
 */
function renderSiteHeader(): string {
  return h({
    tag: 'header',
    attrs: { class: 'site-header', },
    children: [
      h({
        tag: 'a',
        attrs: {
          href: '/',
          class: 'site-title',
        },
        text: 'messages-demo',
      },),
    ],
  },);
}

/**
 * Builds the composer `<footer>` element. The composer is server-
 * rendered as a plain form; the client script enhances it on focus.
 *
 * @param options - edit-mode message id and tier hint
 *
 * @returns rendered `<footer>` HTML
 *
 * @example
 * ```ts
 * renderComposerFooter({ editMessageId: 42 });
 * ```
 */
function renderComposerFooter(
  options: {
    readonly editMessageId?: number;
    // oxlint-disable-next-line eslint/no-magic-numbers -- tier discriminant
    readonly initialTier?: 1 | 2 | 3;
  },
): string {
  /**
   * Pre-rendered `<option>` HTML for the identity select, in seed-list order.
   */
  const userOptions = seedUsers
    .map(function toOption(user,) {
      return h({
        tag: 'option',
        attrs: { value: user.id, },
        text: user.name,
      },);
    },)
    .join('',);

  /**
   * Optional `data-edit-message-id` attribute used by the composer to enter edit mode.
   */
  const editAttr = options.editMessageId
    === undefined
    ? ''
    : ` data-edit-message-id="${String(options.editMessageId,)}"`;
  /**
   * Optional `data-initial-tier` attribute that seeds the composer's tier discriminant.
   */
  const tierAttr = options.initialTier
    === undefined
    ? ''
    : ` data-initial-tier="${String(options.initialTier,)}"`;

  return h({
    tag: 'footer',
    attrs: {
      class: 'composer-footer',
      id: 'composer-mount',
    },
    html:
      `<form class="composer" id="composer" method="post" action="/api/composer-noscript"${editAttr}${tierAttr}>
        <select class="composer-identity" name="user_id" aria-label="Send as">${userOptions}</select>
        <textarea class="composer-body" name="body" placeholder="Write a message&hellip;" required></textarea>
        <button class="composer-send" type="submit">send</button>
      </form>`,
  },);
}

/**
 * Renders a complete HTML document: doctype, `<html>`, `<head>`,
 * `<body>` (with site header, supplied body, composer footer), and
 * the closing tags. Every element goes through `h()`.
 *
 * @param options - title, pre-built body HTML, optional composer mode
 *
 * @returns full HTML document string
 *
 * @example
 * ```ts
 * return new Response(renderPage({ title: 'messages-demo', body }));
 * ```
 */
export function renderPage(options: PageOptions,): string {
  /**
   * Composed `<body>` HTML; embedded inside the doctype envelope below.
   */
  const bodyHtml = renderSiteHeader()
    + h({
      tag: 'main',
      attrs: { class: 'feed-main', },
      html: options.body,
    },)
    + renderComposerFooter(options,);

  return `<!DOCTYPE html>${
    h({
      tag: 'html',
      attrs: { lang: 'en', },
      children: [
        renderHead(options.title,),
        h({
          tag: 'body',
          html: bodyHtml,
        },),
      ],
    },)
  }`;
}

/**
 * Convenience helper: full HTML for a page that does not stream chunks.
 * Used for empty-state messages, error pages, etc.
 *
 * @param options - title and body content
 *
 * @returns full HTML document
 *
 * @example
 * ```ts
 * return new Response(renderSimplePage({ title: 'Gone', body: '...' }));
 * ```
 */
export function renderSimplePage(
  options: {
    readonly title: string;
    readonly body: string;
  },
): string {
  return renderPage({
    title: options.title,
    body: options.body,
  },);
}
