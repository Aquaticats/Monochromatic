import { PLAIN_TEXT, } from './plain-module.ts';

/**
 Returns the statically imported plain value, so the bundle keeps the
 untagged `import` declaration alive.
 
 @returns text the plain module exports
 */
export function getPlainText(): string {
  return PLAIN_TEXT;
}
