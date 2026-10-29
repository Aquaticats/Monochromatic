// Polls one GitHub issue until a maintainer acts on it or the deadline passes, then exits so the caller is woken.
// Exit codes: 10 = something to act on (details on stdout), 0 = deadline reached with nothing to act on,
// 2 = too many consecutive gh failures.
// Environment: POLL_REPO, POLL_ISSUE, POLL_OWN_LOGIN, POLL_DEADLINE_SECONDS, POLL_INTERVAL_SECONDS,
// POLL_BASELINE (`now` ignores comments that exist at start; `none` treats them all as new, for testing).
import { execFileSync } from 'node:child_process';

const repo = process.env.POLL_REPO ?? 'pnpm/pnpm';
const issue = process.env.POLL_ISSUE ?? '17485';
const ownLogin = process.env.POLL_OWN_LOGIN ?? 'Aquaticat';
const deadlineSeconds = Number(process.env.POLL_DEADLINE_SECONDS ?? 86400,);
const intervalSeconds = Number(process.env.POLL_INTERVAL_SECONDS ?? 300,);
const baselineMode = process.env.POLL_BASELINE ?? 'now';
const maintainerAssociations = new Set(['OWNER', 'MEMBER', 'COLLABORATOR',],);

type Comment = { readonly id: number; readonly user: { readonly login: string; readonly type: string; }; readonly author_association: string; readonly body: string; readonly html_url: string; };
type Issue = { readonly state: string; readonly assignees: readonly { readonly login: string; }[]; };

const gh = (path: string,): string => execFileSync('gh', ['api', '--paginate', path,], { encoding: 'utf8', timeout: 120000, },);
const log = (message: string,): void => console.error(`${new Date().toISOString()} ${message}`,);
const sleep = (seconds: number,): Promise<void> => new Promise((resolve,) => setTimeout(resolve, seconds * 1000,),);

const readComments = (): readonly Comment[] => JSON.parse(`[${gh(`repos/${repo}/issues/${issue}/comments`,).replaceAll('][', ',',).replace(/^\[/, '',).replace(/\]\s*$/, '',)}]`,) as Comment[];
const readIssue = (): Issue => JSON.parse(gh(`repos/${repo}/issues/${issue}`,),) as Issue;

const isMaintainerComment = (comment: Comment,): boolean =>
  comment.user.type !== 'Bot' && comment.user.login !== ownLogin && maintainerAssociations.has(comment.author_association,);

const started = Date.now();
const seen = new Set<number>();
let failures = 0;
let initialAssignees: string | undefined;

if (baselineMode === 'now') for (const comment of readComments()) seen.add(comment.id,);

log(`polling ${repo}#${issue} every ${intervalSeconds}s for ${deadlineSeconds}s, ${seen.size} existing comments ignored`,);

while ((Date.now() - started) / 1000 < deadlineSeconds) {
  try {
    const comments = readComments();
    const current = readIssue();
    const assignees = current.assignees.map((assignee,) => assignee.login,).join(',',);
    initialAssignees ??= assignees;
    failures = 0;

    const fresh = comments.filter((comment,) => !seen.has(comment.id,),);
    for (const comment of fresh) seen.add(comment.id,);
    const maintainerComments = fresh.filter(isMaintainerComment,);

    if (maintainerComments.length > 0) {
      console.log(JSON.stringify({ trigger: 'maintainer-comment', comments: maintainerComments.map((comment,) => ({ by: comment.user.login, association: comment.author_association, url: comment.html_url, body: comment.body.slice(0, 1500,), }),), },),);
      process.exit(10,);
    }
    if (current.state !== 'open') {
      console.log(JSON.stringify({ trigger: 'issue-not-open', state: current.state, },),);
      process.exit(10,);
    }
    if (assignees !== initialAssignees) {
      console.log(JSON.stringify({ trigger: 'assignees-changed', from: initialAssignees, to: assignees, },),);
      process.exit(10,);
    }
  } catch (error) {
    failures += 1;
    log(`poll failed (${failures}): ${String(error,).slice(0, 200,)}`,);
    if (failures >= 20) process.exit(2,);
  }
  await sleep(intervalSeconds,);
}

console.log(JSON.stringify({ trigger: 'deadline', },),);
process.exit(0,);
