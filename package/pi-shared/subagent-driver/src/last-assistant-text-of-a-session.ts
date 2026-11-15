/**
 Reads the last assistant message a Pi session wrote, whole: the message a review that ended without writing its review file leaves
 its findings in.

 The summary's own `lastAssistantText` cannot be the source: it is cut to `FEED_PREVIEW_CHARACTERS.assistantText`, which is 900
 characters, and a review of six findings is longer than that. A session file may also be one a resumed Pi is still appending to, so
 the record it ends with may be one the writer has not finished, and that cut text is left alone rather than parsed.

 @module
 */

import {
  fieldsOf,
  isRecord,
} from './read-pi-fields.ts';
import { wholeLinesOf, } from './whole-lines-of-a-feed.ts';

/**
 Reads the text of the last assistant message in a session's records.

 The session file is Pi's own, and it is read while a resumed Pi may still be appending to it, so the record it ends with may be
 one the writer has not finished: that cut text is no record and is left alone rather than parsed.

 @param shown - the session file, one JSON record per line

 @returns the message's text parts joined, or empty text when the session holds no assistant message

 @example
 ```ts
 const text = lastAssistantTextIn({ shown: '{"type":"message","message":{"role":"assistant","content":[{"type":"text","text":"a"}]}}', },);
 ```
 */
export function lastAssistantTextIn({ shown, }: { readonly shown: string },): string {
  /**
   The last assistant message found so far.
   */
  const parts: string[][] = [];
  for (const line of wholeLinesOf({ shown, },)) {
    if (line === '') {
      continue;
    }
    /**
     One record of the session.
     */
    const record: unknown = JSON.parse(line,);
    if ((!isRecord(record,)) || (record.type !== 'message')) {
      continue;
    }
    /**
     The message the record carries.
     */
    const message = fieldsOf(record.message,);
    if (message.role !== 'assistant') {
      continue;
    }
    parts.push((Array.isArray(message.content) ? message.content : [])
      .flatMap(function textOf(part,) {
        return (isRecord(part,) && (part.type === 'text')
          && ((typeof part.text) === 'string'))
          ? [part.text,]
          : [];
      },));
  }
  return (parts.at(-1) ?? []).join('\n',)
    .trim();
}
