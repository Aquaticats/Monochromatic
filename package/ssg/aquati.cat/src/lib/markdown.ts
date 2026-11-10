/**
 MDX-to-HTML rendering pipeline.
 
 Uses `@mdx-js/mdx` to compile and evaluate MDX source, with a minimal
 JSX runtime (`jsx-to-html`) that produces HTML strings directly instead
 of virtual DOM nodes. This follows the intended MDX flow; parse, compile
 to JS, evaluate; so JSX comments, expressions, and imports are handled
 correctly by the MDX compiler.
 
 Syntax highlighting is handled client-side via the CSS Custom Highlight API
 with Lezer parsers. The pipeline outputs plain `<pre><code class="language-xxx">`
 blocks that the client script picks up.
 */
import { evaluate, } from '@mdx-js/mdx';
import rehypeAutolinkHeadings from 'rehype-autolink-headings';
import rehypeSlug from 'rehype-slug-custom-id';
import remarkGfm from 'remark-gfm';
import remarkSectionize from 'remark-sectionize';

import { createMdxComponents, } from '../component/index.ts';
import type { Locale, } from '../i18n/index.ts';

import {
  Fragment,
  jsx,
  jsxs,
  type SafeHtml,
} from './jsx-to-html.ts';
import rehypeHighlight from './rehype-highlight.ts';

/**
 Renders an MDX body string to an HTML string.
 
 Pipeline stages (handled internally by `@mdx-js/mdx` and the JSX runtime):
 1. `remark-parse` + `remark-mdx`: parse markdown and MDX syntax (built-in)
 2. `remark-gfm`: GitHub Flavored Markdown (tables, autolinks, task lists)
 3. `remark-sectionize`: wrap headings in `<section>` elements
 5. `remark-rehype`: convert markdown AST to HTML AST (built-in)
 6. `rehype-slug`: add `id` attributes to headings
 7. `rehype-autolink-headings`: add anchor links to headings
 8. `rehype-highlight`: pre-compute Lezer syntax highlight ranges
 9. Compile to JS and evaluate with string-producing JSX runtime
 
 JSX comments (`{/* ... *\/}`) are naturally stripped during JS compilation,
 just as they would be in any JSX/JavaScript file.
 
 @param body - raw MDX body content (frontmatter already stripped)
 
 @param lang - locale the post is written in, used to localize the chrome that
 MDX components render
 
 @returns rendered HTML string
 
 @example
 ```ts
 const html = await renderMdx({
   body: '# Hello\n\n{/* this comment disappears *\/}\n\nWorld',
   lang: 'en',
 });
 // '<section><h1 id="hello">...</h1><p>World</p></section>'
 ```
 */
export async function renderMdx(
  {
    body,
    lang,
  }: {
    readonly body: string;
    readonly lang: Locale;
  },
): Promise<string> {
  /**
   Default-exported MDX component returned by {@link evaluate} is the JSX entry point.
   */
  const { default: MDXContent, } = await evaluate(
    body,
    {
      jsx,
      jsxs,
      Fragment,
      useMDXComponents() {
        return createMdxComponents(lang,);
      },
      remarkPlugins: [
        remarkGfm,
        remarkSectionize,
      ],
      rehypePlugins: [
        [
          rehypeSlug,
          {
            enableCustomId: true,
            maintainCase: true,
            removeAccents: true,
          },
        ],
        rehypeAutolinkHeadings,
        rehypeHighlight,
      ],
      // Content is trusted filesystem input authored by the site owner,
      // not user-submitted data, so allowing raw HTML is safe here.
      remarkRehypeOptions: { allowDangerousHtml: true, },
    },
  );

  /* oxlint-disable no-unsafe-type-assertion -- `MDXContent` returns the library's untyped component value, so the assertion narrows it to the runtime-known `SafeHtml` shape produced by our JSX runtime. */
  /**
   Invocation of the evaluated MDX component produces the {@link SafeHtml} payload consumed by callers.
   */
  const result = MDXContent({},) as SafeHtml;
  /* oxlint-enable no-unsafe-type-assertion */
  return result.html;
}
