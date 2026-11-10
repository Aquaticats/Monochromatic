/**
 Navigation DOM builders for the `\<side-drawer\>` component.
 
 Pure functions that construct `\<nav\>`, `\<header\>`, and close-button elements.
 Used by both the inline sidebar and popover panel trees.
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 Builds a nav element with the standard link set.
 
 Every entry leads to a route this app serves; a destination that exists only to
 move the scroll position would be an action that does nothing.
 
 @returns Navigation element with app links
 
 @example
 ```ts
 const nav = buildNav();
 sidebar.append(nav);
 ```
 */
export function buildNav(): HTMLElement {
  return h({
    tag: 'nav',
    children: [
      h({
        tag: 'a',
        attrs: { href: '/', },
        text: 'Inbox',
      },),
      h({
        tag: 'a',
        attrs: { href: '/in-progress', },
        text: 'In Progress',
      },),
      h({
        tag: 'a',
        attrs: { href: '/settings', },
        text: 'Settings',
      },),
    ],
  },);
}

/**
 Builds the drawer header, which holds the close button the phone layout shows.
 
 The design puts a profile identity above the name, but the app has no user data
 contract to read one from, so the row carries no placeholder name until it does.
 
 @param closeButton - Close button element, omitted for the inline sidebar
 
 @returns Header element
 
 @example
 ```ts
 const header = buildHeader(closeButton);
 ```
 */
export function buildHeader(closeButton?: HTMLElement,): HTMLElement {
  return h({
    tag: 'div',
    class: 'header',
    children: closeButton === undefined ? [] : [closeButton,],
  },);
}

/**
 Builds a close button with an X SVG icon.
 
 @param label - Accessible label for the button
 
 @returns Close button element
 
 @example
 ```ts
 const button = buildCloseButton('Close menu');
 ```
 */
export function buildCloseButton(label: string,): HTMLElement {
  /**
   Button element built first so its innerHTML can be set imperatively below.
   */
  const button = h({
    tag: 'button',
    class: 'close',
    attrs: { 'aria-label': label, },
  },);
  // innerHTML for SVG: h() creates HTML-namespace elements, SVG needs SVG namespace
  button.innerHTML =
    `<svg viewBox="0 0 48 48" fill="none" stroke="currentColor" stroke-width="4"><line x1="14" y1="14" x2="34" y2="34"/><line x1="34" y1="14" x2="14" y2="34"/></svg>`;
  return button;
}
