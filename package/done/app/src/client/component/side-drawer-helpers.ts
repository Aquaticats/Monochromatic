/**
 DOM building helpers for `<side-drawer>`.
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 Builds a nav element with the standard link set.
 
 @returns Navigation element with inbox, in-progress, settings, and contact links
 
 @example
 ```ts
 const nav = buildNav();
 shadow.append(nav);
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
      h({
        tag: 'a',
        attrs: { href: '#', },
        text: 'Contact',
      },),
    ],
  },);
}

/**
 Builds a header row with a name label and an optional close button.
 
 @param closeButton - Close button element, omitted for read-only headers
 
 @returns Header div element
 
 @example
 ```ts
 const header = buildHeader(buildCloseButton('Close drawer'));
 ```
 */
export function buildHeader(closeButton?: HTMLElement,): HTMLElement {
  /**
   Accumulator for the header children so an optional close button can be appended.
   */
  const children: HTMLElement[] = [
    h({
      tag: 'span',
      style: { fontSize: '1.25rem', },
      text: 'Firstname',
    },),
  ];
  if (closeButton !== undefined)
    children.push(closeButton,);
  return h({
    tag: 'div',
    class: 'header',
    children,
  },);
}

/**
 Builds a close button with an X SVG icon.
 
 @param label - Accessible aria-label for the button
 
 @returns Button element with SVG close icon
 
 @example
 ```ts
 const closeBtn = buildCloseButton('Close drawer');
 ```
 */
export function buildCloseButton(label: string,): HTMLElement {
  /**
   Bare button shell so the inline SVG can be injected via innerHTML below.
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
