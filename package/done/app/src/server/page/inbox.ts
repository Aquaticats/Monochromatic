/**
 Inbox page handler.
 
 Queries the DB for inbox tasks, then delegates to the shared `renderPage()` shell.
 The `entryScriptPath` tells the browser to load `/dist/client/inbox.js`,
 which reads `pageData` from the embedded JSON blob and builds the UI into `<main id="app">`.
 */
import {
  listBlockedInboxTasks,
  listInboxUnblockedTasks,
} from '../../lib/db/tasks-queries.ts';
import { renderPage, } from './layout.ts';

/**
 Renders the inbox page with unblocked ({@link listInboxUnblockedTasks}) and
 blocked ({@link listBlockedInboxTasks}) task lists via {@link renderPage}.
 
 @returns HTML response for the inbox page
 
 @example
 ```ts
 const response = await inboxPage();
 ```
 */
export async function inboxPage(): Promise<Response> {
  /**
   Unblocked inbox tasks; surface row for both `suggestedTasks` and `allTasks`.
   */
  const inboxTasks = await listInboxUnblockedTasks();
  /**
   Raw blocker-to-blocked rows fed to the grouping below.
   */
  const blockedLinks = await listBlockedInboxTasks();
  /**
   Pre-grouped map so the client can render blocked children under each blocker.
   */
  const blockedTasksByBlocker = Object.groupBy(
    blockedLinks,
    function byBlocker(link,) {
      return link.blockerId;
    },
  );

  return renderPage({
    title: 'Inbox - Done',
    heading: 'Inbox',
    entryScriptPath: '/dist/client/inbox.js',
    pageData: {
      suggestedTasks: inboxTasks,
      allTasks: inboxTasks,
      blockedTasksByBlocker,
    },
  },);
}
