/**
 Inbox page handler.
 
 Queries the DB for inbox tasks, then delegates to the shared `renderPage()` shell.
 The `entryScriptPath` tells the browser to load `/dist/client/inbox.js`,
 which reads `pageData` from the embedded JSON blob and builds the UI into `<main id="app">`.
 */
import {
  listBlockedInboxTasks,
  listInboxUnblockedTasks,
} from '../../lib/db/tasks.ts';
import { renderPage, } from './layout.ts';

/**
 Renders the inbox page with unblocked and blocked task lists.
 
 @returns HTML response for the inbox page
 
 @example
 ```ts
 const response = await inboxPage();
 ```
 */
export async function inboxPage(): Promise<Response> {
  /**
   Top-level inbox tasks with no remaining blockers.
   */
  const inboxTasks = await listInboxUnblockedTasks();
  /**
   Blocked tasks paired with the blocker IDs that hide them.
   */
  const blockedLinks = await listBlockedInboxTasks();
  /**
   Blocked-task links grouped by blocker ID for nested rendering in the client.
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
