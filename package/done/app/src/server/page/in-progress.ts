/**
 In-progress page handler.
 
 Queries tasks with active timers, then delegates to the shared `renderPage()` shell.
 Client entry: `/dist/client/in-progress.js` (src/client/in-progress.ts)
 */
import { listInProgressTasks, } from '../../lib/db/tasks-queries.ts';
import { renderPage, } from './layout.ts';

/**
 Renders the in-progress page listing tasks with active timers, queried via
 {@link listInProgressTasks} and rendered through {@link renderPage}.
 
 @returns HTML response for the in-progress page
 
 @example
 ```ts
 const response = await inProgressPage();
 ```
 */
export async function inProgressPage(): Promise<Response> {
  /**
   Active timer rows forwarded as the page payload.
   */
  const tasks = await listInProgressTasks();

  return renderPage({
    title: 'In Progress - Done',
    heading: 'In Progress',
    entryScriptPath: '/dist/client/in-progress.js',
    pageData: { tasks, },
  },);
}
