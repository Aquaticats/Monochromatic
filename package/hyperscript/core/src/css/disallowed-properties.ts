/**
 CSS properties banned by the project's style guide.
 
 Extracted from `properties.ts` to stay within the line budget.
 */

/**
 CSS properties banned by the project's style guide.
 
 Categories:
 - **Pointless**: `clear`, `float`
 - **Deprecated**: `clip`, `font-smooth`
 - **Shorthands**: Always use longhand form
 - **Non-logical**: Use logical property equivalents
 
 @example
 ```ts
 // These properties cause type errors in StrictCssDeclarations:
 const bad: StrictCssDeclarations = {
   width: cssRem(10),   // ✗ use 'inline-size'
   margin: cssRem(1),   // ✗ use 'margin-block' / 'margin-inline'
   float: 'left',       // ✗ pointless property
 };
 ```
 */
export type DisallowedProperties =
  // Pointless properties
  | 'clear'
    | 'float'
  // Deprecated / non-standard
    | 'clip'
    | 'font-smooth'
  // Shorthand properties: always use longhand
    | 'animation'
    | 'background'
    | 'border'
    | 'border-block'
    | 'border-inline'
    | 'container'
    | 'flex'
    | 'font'
    | 'font-synthesis'
    | 'font-variant'
    | 'grid'
    | 'grid-area'
    | 'grid-template'
    | 'inset'
    | 'list-style'
    | 'margin'
    | 'scroll-margin'
    | 'mask'
    | 'offset'
    | 'outline'
    | 'overflow'
    | 'overscroll-behavior'
    | 'padding'
    | 'scroll-padding'
    | 'place-content'
    | 'place-items'
    | 'place-self'
    | 'scroll-timeline'
    | 'transition'
    | 'view-timeline'
  // Non-logical dimension properties: use inline-size / block-size equivalents
    | 'width'
    | 'height'
    | 'min-width'
    | 'min-height'
    | 'max-width'
    | 'max-height'
    | 'contain-intrinsic-width'
    | 'contain-intrinsic-height'
  // Non-logical direction properties: use logical equivalents
    | 'top'
    | 'left'
    | 'right'
    | 'bottom'
    | 'border-top'
    | 'border-top-color'
    | 'border-top-style'
    | 'border-top-width'
    | 'border-bottom'
    | 'border-bottom-color'
    | 'border-bottom-style'
    | 'border-bottom-width'
    | 'border-left'
    | 'border-left-color'
    | 'border-left-style'
    | 'border-left-width'
    | 'border-right'
    | 'border-right-color'
    | 'border-right-style'
    | 'border-right-width'
    | 'border-top-left-radius'
    | 'border-top-right-radius'
    | 'border-bottom-left-radius'
    | 'border-bottom-right-radius'
    | 'margin-top'
    | 'margin-bottom'
    | 'margin-left'
    | 'margin-right'
    | 'padding-top'
    | 'padding-bottom'
    | 'padding-left'
    | 'padding-right'
    | 'scroll-margin-top'
    | 'scroll-margin-bottom'
    | 'scroll-margin-left'
    | 'scroll-margin-right'
    | 'scroll-padding-top'
    | 'scroll-padding-bottom'
    | 'scroll-padding-left'
    | 'scroll-padding-right'
    | 'overflow-clip-margin';
