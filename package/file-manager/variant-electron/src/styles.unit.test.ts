/* oxlint-disable no-restricted-syntax/no-regex -- This file inspects a stylesheet's text, so the CSS value shapes it matches (color literals, custom-property uses, fallbacks) are the fixtures themselves; no untrusted input reaches these patterns. */

/**
 Unit tests for the pane strip stylesheet.
 
 The stylesheet is the layout engine and the only visual channel that tells
 states apart, so these tests read the staged copy the Electron app loads and
 assert the CSS-quality minima the project requires of it: colors from custom
 properties, and a visible focus ring plus a 48px touch target on interactive
 elements.
 
 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/**
 Stylesheet the staged app loads.
 */
const STYLESHEET_URL = new URL('../dist/app/styles.css', import.meta.url,);

/**
 Matches a hex or functional color literal inside one declaration.
 */
const COLOR_LITERAL = /#[0-9a-f]{3,8}\b|\b(?:rgba?|hsla?)\(/iu;

/**
 Matches a declaration that defines a color token, the one place a literal may
 appear.
 */
const COLOR_TOKEN_DECLARATION = /^\s*--color-[a-z0-9-]+:/u;

/**
 Matches a custom-property use that carries a fallback value.
 */
const CUSTOM_PROPERTY_WITH_FALLBACK = /var\(\s*--[a-z0-9-]+\s*,/iu;

/**
 Matches one custom-property name, whether it is being defined or used.
 */
const CUSTOM_PROPERTY_NAME = /--color-[a-z0-9-]+/gu;

/**
 Value `indexOf` returns when the searched text is absent.
 */
const NOT_FOUND = -1;

/**
 Minimum touch target, expressed in `rem` as the CSS-quality minimum asks
 (48px at the default root font size).
 */
const MINIMUM_TOUCH_TARGET = '3rem';

/**
 Focus ring every interactive element carries.
 */
const FOCUS_RING = 'outline: 2px solid var(--color-focus);';

/**
 Gap between the focus ring and the element edge.
 */
const FOCUS_RING_OFFSET = 'outline-offset: 2px;';

/**
 Stylesheet text under test.
 */
const stylesheet = await readFile(STYLESHEET_URL, 'utf8',);

/**
 Splits stylesheet text into trimmed declaration lines.
 
 @param text - Stylesheet text to split.
 
 @returns Non-empty trimmed lines.
 */
function declarationLinesOf({ text, }: { readonly text: string; },): string[] {
  return text
    .split('\n',)
    .map(function trimmed(line,): string {
      return line.trim();
    },)
    .filter(function nonEmpty(line,): boolean {
      return line.length > 0;
    },);
}

/**
 Reads one rule's body from the stylesheet.
 
 The stylesheet is flat, so a rule body runs from the selector's opening brace
 to the next closing brace.
 
 @param selector - Selector text without the opening brace.
 
 @returns The rule body, opening brace included.
 
 @throws Error when the selector or its closing brace is missing.
 */
function ruleBodyOf({ selector, }: { readonly selector: string; },): string {
  /**
   Offset of the rule's opening brace.
   */
  const openBrace = stylesheet.indexOf(`${selector} {`,);

  if (openBrace === NOT_FOUND)
    throw new Error(`styles.css has no ${selector} rule`,);

  /**
   Offset of the rule's closing brace.
   */
  const closeBrace = stylesheet.indexOf('}', openBrace,);

  if (closeBrace === NOT_FOUND)
    throw new Error(`styles.css leaves the ${selector} rule unterminated`,);

  return stylesheet.slice(
    openBrace,
    closeBrace,
  );
}

await describe({
  name: '',
  children: [
    describe({
      name: 'styles.css',
      children: [
        it({
          name: 'paints every color from a custom property',
          fn: async (): Promise<void> => {
            const offenders = declarationLinesOf({ text: stylesheet, },)
              .filter(function hasColorLiteral(line,): boolean {
                return COLOR_LITERAL.test(line,);
              },)
              .filter(function isNotTokenDefinition(line,): boolean {
                return !COLOR_TOKEN_DECLARATION.test(line,);
              },);

            expect(offenders,).toEqual([],);
          },
        },),
        it({
          name: 'uses only custom properties it defines, without fallbacks',
          fn: async (): Promise<void> => {
            const lines = declarationLinesOf({ text: stylesheet, },);
            const defined = new Set(lines
              .filter(function isTokenDefinition(line,): boolean {
                return COLOR_TOKEN_DECLARATION.test(line,);
              },)
              .map(function tokenNameOf(line,): string {
                return line.slice(
                  0,
                  line.indexOf(':',),
                )
                  .trim();
              },),);
            const used = new Set(lines
              .flatMap(function namesIn(line,): string[] {
                return [...line.matchAll(CUSTOM_PROPERTY_NAME,),]
                  .map(function matchedName(match,): string {
                    return match[0];
                  },);
              },),);

            expect([...used,]
              .filter(function isUndefined(name,): boolean {
                return !defined.has(name,);
              },),).toEqual([],);
            expect(lines
              .filter(function hasFallback(line,): boolean {
                return CUSTOM_PROPERTY_WITH_FALLBACK.test(line,);
              },),).toEqual([],);
          },
        },),
        it({
          name: 'rings the pane close control and gives it a touch target',
          fn: async (): Promise<void> => {
            const body = ruleBodyOf({ selector: '.pane-close', },);
            const focusBody = ruleBodyOf({ selector: '.pane-close:focus-visible', },);

            expect(body,).toContain(`min-inline-size: ${MINIMUM_TOUCH_TARGET};`,);
            expect(body,).toContain(`min-block-size: ${MINIMUM_TOUCH_TARGET};`,);
            expect(focusBody,).toContain(FOCUS_RING,);
            expect(focusBody,).toContain(FOCUS_RING_OFFSET,);
          },
        },),
        it({
          name: 'rings a listing entry and gives it a touch target',
          fn: async (): Promise<void> => {
            const body = ruleBodyOf({ selector: '.entry', },);
            const focusBody = ruleBodyOf({ selector: '.entry:focus-visible', },);

            expect(body,).toContain(`min-inline-size: ${MINIMUM_TOUCH_TARGET};`,);
            expect(body,).toContain(`min-block-size: ${MINIMUM_TOUCH_TARGET};`,);
            expect(focusBody,).toContain(FOCUS_RING,);
            expect(focusBody,).toContain(FOCUS_RING_OFFSET,);
          },
        },),
      ],
    },),
  ],
},);
