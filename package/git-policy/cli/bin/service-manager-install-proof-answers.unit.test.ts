import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  countsOfJournalLines,
  nonEmptyLines,
  statusOf,
} from './service-manager-install-proof-answers.ts';

/**
 The text a background commit wrote when it landed, as the machine printed it: the commit's own last lines and the status
 line the shell wrote for it.

 Stands for that output only, by the shape the shell line of `commit` writes; no line here is a capture of a real commit.

 @returns the text of a commit that landed
 */
function landedCommitPrintout(): string {
  return '[main 4f2c1de] install\n 1 file changed\nstatus=0\n';
}

await describe({
  name: '',
  children: [
    describe({
      name: statusOf.name,
      children: [
        it({
          name: 'read the status a landed commit wrote for itself',
          fn: async () => {
            expect(statusOf({ printed: landedCommitPrintout(), },),).toBe(0,);
          },
        }),
        it({
          name: 'read a status that follows another line of the same printout',
          fn: async () => {
            expect(statusOf({ printed: 'committed\nstatus=0\n', },),).toBe(0,);
          },
        }),
        it({
          name: 'read a negative status, which a commit killed by a signal writes',
          fn: async () => {
            expect(statusOf({ printed: 'status=-15', },),).toBe(-15,);
          },
        }),
        it({
          name: 'read a status written with leading zeros as the number it is',
          fn: async () => {
            expect(statusOf({ printed: 'status=007', },),).toBe(7,);
          },
        }),
        ...['no status line at all', 'status=', 'status=+1', 'status=1 2', 'status=one', 'xstatus=0',].map(
          function aLineThatIsNoStatus(line,) {
            return it({
              name: `answer one when the printout holds no status line: ${line}`,
              fn: async () => {
                expect(statusOf({ printed: line, },),).toBe(1,);
              },
            },);
          },
        ),
      ],
    },),
    describe({
      name: nonEmptyLines.name,
      children: [
        it({
          name: 'keep every line that holds something, in the order it was listed',
          fn: async () => {
            expect(nonEmptyLines({ listed: 'a.service\n\n b.service \n', },),).toEqual(['a.service', 'b.service',],);
          },
        }),
        it({
          name: 'answer nothing for a listing the manager wrote nothing into',
          fn: async () => {
            expect(nonEmptyLines({ listed: '', },),).toEqual([],);
          },
        }),
      ],
    },),
    describe({
      name: countsOfJournalLines.name,
      children: [
        it({
          name: 'count the starts, the EXEC failures and the start-limit hits of one journal',
          fn: async () => {
            expect(countsOfJournalLines({
              journal: [
                'systemd[1]: monochromatic-git-policy-cli@work-one.service: started release=0.6.1-alpha.20',
                'systemd[1]: monochromatic-git-policy-cli@work-two.service: started release=0.6.1-alpha.21',
                'systemd[1]: monochromatic-git-policy-cli@work-two.service: Failed to locate executable: 203/EXEC',
                'systemd[1]: monochromatic-git-policy-cli@work-two.service: start-limit-hit',
                'systemd[1]: an unrelated unit: started release=0.6.0',
              ].join('\n',),
            },),).toEqual({
              starts: 3,
              execFailures: 1,
              limits: 1,
            },);
          },
        }),
        it({
          name: 'answer zero for every count when the read reached nothing',
          fn: async () => {
            expect(countsOfJournalLines({ journal: '', },),).toEqual({
              starts: 0,
              execFailures: 0,
              limits: 0,
            },);
          },
        }),
      ],
    },),
  ],
},);
