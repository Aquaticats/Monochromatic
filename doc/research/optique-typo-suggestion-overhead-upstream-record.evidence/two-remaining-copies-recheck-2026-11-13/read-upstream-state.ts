// Reads upstream's state with read-only GitHub calls, so that every "as of" written into a document today rests on a
// reading made today: the head of main, open pull requests, pull requests and issues changed today, the newest
// commits on the three files this work touches, and the comments on the commit thread the maintainer wrote on.
// Nothing is written to GitHub.
//
// Run: node read-upstream-state.ts > keep/upstream-state.jsonl

import { spawnSync, } from 'node:child_process';

function gh(args: string[],): unknown {
  const child = spawnSync('gh', args, { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, },);

  if (child.status !== 0) return { failed: child.stderr.slice(0, 300,), };

  try {
    return JSON.parse(child.stdout,);
  }
  catch {
    return child.stdout.trim().slice(0, 2000,);
  }
}

console.log(JSON.stringify({ readAt: new Date().toISOString(), },),);
console.log(JSON.stringify({ main: gh(['api', 'repos/dahlia/optique/commits/main', '--jq', '{sha: .sha, date: .commit.committer.date, message: (.commit.message | split("\n")[0])}',],), },),);
console.log(JSON.stringify({ openPullRequests: gh(['pr', 'list', '--repo', 'dahlia/optique', '--state', 'open', '--json', 'number,title,createdAt,author,headRefName',],), },),);
console.log(JSON.stringify({ newestPullRequests: gh(['pr', 'list', '--repo', 'dahlia/optique', '--state', 'all', '--limit', '6', '--json', 'number,title,state,createdAt,mergedAt',],), },),);
console.log(JSON.stringify({ newestIssues: gh(['issue', 'list', '--repo', 'dahlia/optique', '--state', 'all', '--limit', '6', '--json', 'number,title,state,createdAt,closedAt',],), },),);

for (const path of ['packages/core/src/modifiers.ts', 'packages/core/src/primitives.ts', 'packages/core/src/input-trace.ts',]) {
  console.log(JSON.stringify({
    path,
    newestCommits: gh(['api', '--method', 'GET', 'repos/dahlia/optique/commits', '-f', `path=${path}`, '-f', 'per_page=3', '--jq', '[.[] | {sha: .sha[0:10], date: .commit.committer.date, message: (.commit.message | split("\n")[0])}]',],),
  },),);
}

console.log(JSON.stringify({
  commitThread: gh(['api', 'repos/Aquaticats/Monochromatic/commits/4879a44e6a/comments', '--jq', '[.[] | {user: .user.login, created: .created_at, updated: .updated_at, start: (.body | .[0:80])}]',],),
},),);
