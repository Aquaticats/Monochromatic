/**
 Attribution bar HTML rendering for the doodle widget.
 
 Produces a thin bottom bar with a "Source code" link pointing
 to the package's repository on GitHub.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 Renders a thin attribution bar with a source code link.
 
 The full URL is shown as link text so users can see and copy it
 even when the file is opened locally outside a browser.
 
 @param sourceUrl - fully resolved URL to the package source code
 
 @returns attribution bar HTML string
 
 @example
 ```ts
 const html = renderAttribution('https://github.com/Aquaticat/Monochromatic/tree/main/packages/...');
 ```
 */
export function renderAttribution(sourceUrl: string,): string {
  return h({
    tag: 'div',
    class: 'attribution',
    children: [
      h({
        tag: 'a',
        attrs: {
          href: sourceUrl,
          target: '_blank',
          rel: 'noopener noreferrer',
        },
        text: `Source code: ${sourceUrl}`,
      },),
    ],
  },);
}
