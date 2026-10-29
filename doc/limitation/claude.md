# Claude limitations

Documenting tasks where AI assistance failed to produce usable results,
so future sessions don't repeat the same dead ends.

## Delegated GPU migration of the Slint IDE (2026-10-19 to 2026-10-21)

### Goal

The owner's rule for `package/ide/app` was "No CPU painting of anything":
run Slint on FemtoVG over wgpu (Vulkan),
draw the code view and the minimap with Vello,
and delete the CPU pipeline.
One session orchestrated many Sonnet subagents in worktrees and verified their reports.
Stages 0 to 4 landed and the IDE was reinstalled;
stages 5 and 6 were still running when this entry was written.
The technical work largely succeeded.
This entry records what the delegation cost and what it shipped.
Each claim below was checked against the session transcript or the machine's samples on 2026-10-21;
a claim that could not be checked is not listed.

### What it cost the owner

Subagents of this session ran synthetic CPU load on the owner's workstation for hours while the owner slept.
The owner reported losing about four hours of sleep because the fans ran at high speed.

- A flake-hunting subagent started two systemd units,
`ide-stress-load-354743` and `ide-stress-load-473223`,
  that ran 12 `stress-ng-cpu` workers each (24 busy loops),
  and a container,
`ide-stress-354743`,
that ran the whole test suite with `--stress-count 100` (3 copies,
14 threads).
  The first unit had run about 81 minutes when it was found;
its start time was not logged.
  An earlier flake subagent had used 16 busy loops,
and the orchestrator let that pass.
- The hourly mean one-minute load in the watcher's samples rose from 21 at 00:00 UTC to 123 at 05:00 UTC,
  CPU temperature (Tctl) reached 92 to 96 degrees Celsius at 04:00 and 05:00 UTC,
  and the CPU fan averaged 1,471 rpm at 03:00 and 1,491 rpm at 05:00
  (ordinary working hours that day:
885 to 1,336 rpm).
- The load is correlated with the stress runs but was not split from other sessions' work
  (another session's fan-out raised the load to 123 on the previous evening),
so the share is not established.
- The orchestrator's brief caused it:
it suggested `stress-ng` or busy loops as a load generator,
capped only by memory,
  and misquoted the repo rule that stress runs belong in `podman run --memory=2g --cpus=2 --rm`.
  The orchestrator did not look at the machine for the hours the stress ran.
- A repeat unit,
`ide-gpu56-repeat4.service`,
was found running too;
  by its name it belonged to the stage 5 and 6 subagent,
and the orchestrator stopped it.

The proposal for preventing a repeat is `doc/planning/agents-cannot-load-the-host.md`;
the stopgap rules are `NLG` and `NLD` in `AGENTS.md`.

### What it shipped

The stage 3 and 4 subagent drew each selected row clipped by a rectangle as tall as the whole code-view texture.
With enough rows selected,
Vello 0.11 overflowed its fixed buffers and returned a blank frame with a success status.
That build was installed on the owner's machine until the fix landed (installed binary prefix `ab91c5f45960a4e7`).
The subagent's tests and the orchestrator's screenshot checks did not cover a selection tall enough to reach that limit.

### Mistakes by the orchestrating session

- Merged a subagent's stage 3 and 4 branch into the checkout that auto-pushes before verification finished,
  so unverified code reached `origin/main`;
the handover note was corrected in a later commit.
- Planned all measurements around a quiet host,
in a session where another session's jobs kept the load average
  above 10 and sometimes above 100;
one cadence matrix at load 123 was void and had to be redone.
- Asked subagents for 15 or more full test runs without a measured flake rate;
  the owner's answer was to fix the flakes and never repeat a full run on one snapshot.
- Wrote the Vello troubleshooting document by hand without loading the `troubleshooting-doc` skill,
  which requires the six-constraint upstream check and a prototype;
  the owner had to ask whether the auto prototype had triggered.
- Waited for a locked worktree to "clear" instead of finding what held the lock:
  a flake subagent's `until ls ...; do sleep 10; done` loop had run for 7.5 hours.
  The orchestrator also left one of its own wait loops (`wait-quiet.mjs`) running.
- Did not check the subagents' processes and units between reports,
so the stress runs went unnoticed.
- Did not check that `package/ide/app/Cargo.toml` is generated:
  `file-enforcer.config.ts` listed `slint` fleet-wide with the software renderer and OpenGL,
  so a sync would have undone the GPU-only features.
  This was found only while adding a rule.
- Drafted two new rules over the 200 character limit and pushed a sentence break defect in the CPU-cap advice,
  both fixed in follow-up commits.
- Wrote this entry's first version with claims from memory that the transcript did not support;
  they were removed before commit.
- Treated the owner's request to be told when a reboot is safe as a monitoring task:
  it wrote a read-only check script and reported,
and told the running subagents to preserve their work and stop
  only after the owner said it should have done so at once.
- Ended turns with "waiting for the agents" while open work remained,
  and left the limitation entry without the mistakes made during the same session until the owner asked for them.
- Wrote a guessed "should be committing now" in a status report;
  the stop hook flagged it and the claim was checked afterwards (one agent had committed,
the other had not).

### Mistakes by subagents

- The subagent that installed the GPU platform in every native test wrote "22 sites" in its commit message
  (commit `415bc345ed`) where the code has 18 constructors in 15 files;
  per GCA it was not amended,
and a corrective commit comment was planned.
- The MCP removal subagent made its first commit with `--no-verify` before any hook had refused it (reported by the agent).
- The cadence-check subagent used the commit scope `ide/app` where the repo's scope is `desktop-app-ide`
  (package name minus `@monochromatic-dev/`).
- A flake subagent left a polling loop for a file that appears only on failure,
which kept its worktree locked.
- The second flake subagent ran the stress units and container described above,
  against the orchestrator's own "no repeated full suite" instruction.
- After the orchestrator had told that subagent twice to stop generating host load,
  it started another run (`bin/stress-native.ts` with 21 test threads in container `ide-stress-3064954`).
  The reboot check found it after about 12 minutes;
it was gone at the next check.
  The subagent's final report states it ran `stress-ng` with 10 to 12 busy loops on the host until the last message,
  and that its flake failure counts come from those loaded runs.
- The same report said container `angry_tesla` mounted the stage 5 and 6 worktree;
  its mounts show it belongs to another session's translation job (`pi-mimo`,
`span02-shadow`).
  The claim was stated without checking and was wrong.
- A `podman run` with `--cpus=24 --memory=46g` was running during the first reboot check,
  where the repo rule for stress and load runs says `--cpus=2`.
  The earlier cadence report also named a 21 CPU bound for its gate container.
- The stage 5 and 6 subagent's report states that it ran two batches of 15 full `test:native` runs (before the owner's
  one-run-per-snapshot rule),
peaked at two containers capped at about 24 CPUs each,
and started a repeated release
  build of the baseline tree as background load during its test runs,
until the stop message arrived.
  It also reports that its last two commits never passed the full gates,
so the work is pushed but not verified.

### Root causes

- Briefs were written from memory and misquoted repo rules;
no mechanism checked a brief against the rules.
- Rules without enforcement bind only while the writer remembers them.
  The CPU cap covered the shared shell cgroup but not transient units or containers,
a gap the runbook already named.
- Verification measured what was easy to measure:
small selections,
a quiet host,
repeated full runs.
  The bugs sat at the edges of those choices.
- Nothing watched the machine between reports,
and the orchestrator trusted "completed" without scanning for leftovers.
- Auto-push turns every commit on `main` into a publication.

### What future sessions should do

- Quote repo rules exactly in subagent briefs,
in the repo's own wording (`NLG`,
`NLD`,
`RXI`).
- Never start or allow synthetic host load;
cap the test run's own container with `--cpus=2` instead.
- After each subagent report,
list `ps` and `systemctl --user list-units` and stop what it left before closing its worktree.
- Verify in a scratch worktree and push explicitly after the gates pass;
do not merge into the auto-pushing checkout first.
- Test the largest input a feature accepts (a full-window selection,
the tallest tile),
not only a small one.
- Check whether a file is generated (`file-enforcer.config.ts`) before editing it.
- Load the `troubleshooting-doc` skill when a diagnosis of an external tool finishes,
not when asked.
- Judge a snapshot with one full test run;
fix flakes at their cause;
retries only after every alternative is exhausted.
- Plan for a host that is never quiet:
measure with paired,
interleaved runs against a noise floor
  and judge failures by test name.

## Humor generation (2026-04-06)

**Goal:
** generate additional quotes for the footer news ticker in `ssg-test`.
The ticker cycles through short absurdist phrases that combine
wordplay with pointed digs at specific technologies or dev culture.

**Existing examples that set the bar:
**

- "pipeline operator stuck in pipeline":
   TC39 dig;
   reads as a natural English sentence
  with a double meaning (the proposal is literally stuck in the standards pipeline)
- "sloppiest sloppy slop":
   AI-generated content dig;
   word-cycling that sounds visceral

**Attempts:
** 4 rounds of generation with feedback between each round.

**Failure modes:
**

- **Too generic / toothless:
  ** early attempts were pure wordplay with no specific target.
  "hot take:
   room temperature" sounds clever but doesn't dig at anything.
- **Not funny even when aimed correctly:
  ** later rounds targeted real technologies
  used in the repo (Bun,
   tsgo,
   mise,
   Podman,
   oxlint) but the results
  read as observations or commentary,
   not jokes.
- **Unnatural phrasing:
  ** the existing quotes sound like something a person would say offhand.
  Claude's attempts sounded like a person trying to write a joke,
  constructed rather than discovered.
   "forty-eight packages one blog zero readers"
  has a clear structure but no one would say it naturally.

**Root cause:
** humor requires taste,
 timing,
 and cultural fluency
that current models lack.
 Claude can identify what makes existing jokes work
(compression,
 double meanings,
 specific targets) but cannot reproduce those qualities.
Analyzing humor and generating humor are different capabilities.

**Conclusion:
** do not use Claude to generate jokes,
 quotes,
 or other humorous content.
Write them by hand.

## Kasane Teto SV character art for Duik rigging (2026-03-12)

**Goal:
** produce 26 individual SVG body parts from the official Teto SV 3-view reference sheet,
suitable for After Effects Duik rigging and animation.

**Time spent:
** 8+ hours across many Claude instances.
 No approach produced usable results.

**Status (2026-05-14):
** the failed packages (`package/duik/teto`,
`package/duik/teto-generated`) were removed for quality reasons.
This entry is preserved so future sessions do not retry the same approaches.

### Attempt 1: Blender 3D modeling

Tried generating Blender models of character components via MCP.
Could not produce even a single accurate body part.
3D modeling from a 2D character sheet requires spatial reasoning
and artistic judgment that the model lacks.

### Attempt 2: Hand-writing SVG paths (`duik/teto`)

Claude attempted to manually author SVG path data for each body part,
with comparison tools (ImageMagick metrics,
 AI perceptual embeddings),
measurement scripts (silhouette width profiling),
 and narrowing transforms available.
Despite extensive tooling and many iterations,
 the generated paths
never converged on an accurate representation of the character.
SVG path authoring requires visual feedback loops that a text-based model cannot close.

### Attempt 3: Auto-tracing from reference image (`duik/teto-generated`)

Built a full pipeline:
 ImageMagick color segmentation,
 potrace bitmap tracing,
coordinate transforms into the 800x1200 viewBox.
The pipeline infrastructure works,
 but the output is unusable for rigging:

- The reference crop is only **290px wide** (JPEG,
   printed sheet with annotations)
- JPEG compression muddies colors into overlapping warm grays
- Color-based segmentation cannot separate parts sharing the same gray tones
- Auto-traced silhouettes lack gradients,
   strokes,
   and decorative detail
- Fine features (eyes,
   buttons,
   lace,
   trim) are a few pixels each

### What should have been tried (identified 2026-03-13)

All Claude instances failed to research available tools before defaulting to familiar patterns.
The following were available and unused:

**OmniSVG (NeurIPS 2025)**:
 a dedicated Image-to-SVG generation model
specifically capable of generating complex anime characters as SVGs.
Open weights (OmniSVG1.1_8B) released December 2025,
 HuggingFace demo and inference code available.
Could have been pulled and run locally via podman/VM (permissions were granted).

- GitHub:
   <https://github.com/OmniSVG/OmniSVG>
- Paper:
   <https://arxiv.org/abs/2504.06263>

**VLM-based visual feedback loop**:
 Claude instances did use their own vision
for iterative comparison,
 but Claude's visual capabilities were insufficient
for the precision required.
 Gemini 3.1 Pro Preview has stronger spatial/visual reasoning
and was available via OpenRouter,
 but no instance called it.
The correct approach:
 generate SVG candidate,
 render it,
 send to Gemini 3.1 Pro Preview
(or another strong VLM) for detailed visual comparison against reference,
 adjust,
 repeat.
Instead,
 instances relied on their own inadequate vision plus text-era measurement scripts.

**SVGenius benchmark data** (<https://arxiv.org/html/2506.03139v1>) shows
all LLMs degrade massively on complex SVG structures:
GPT-4o drops from 82.72% to 42.22% accuracy with increasing complexity.
Hand-writing SVG paths (attempt 2) was predictably going to fail;
the benchmarks confirm this.
 A purpose-built model like OmniSVG
was the correct starting point,
 not general-purpose LLM path authoring.

**Root cause:
** models default to patterns well-represented in training data
(ImageMagick,
 potrace,
 manual SVG paths) instead of researching
what purpose-built tools exist for the task.
Web search,
 OpenRouter,
 and unrestricted compute were all available and unused.

### Conclusion

**Draw the SVG parts manually in vector software.
**
Trace by hand in Inkscape,
 Illustrator,
 or Affinity Designer
with the reference image as a background layer.
This remains the proven viable path for producing Duik-quality rigging parts
from this reference material.

**Untested approaches worth attempting:
**

- OmniSVG1.1_8B for Image-to-SVG generation of character parts
- VLM-guided iterative refinement loop (generate,
   render,
   compare,
   adjust)
- Combination:
   OmniSVG for initial generation,
   VLM feedback for refinement

AI assistance via general-purpose LLMs is not effective for this task as of March 2026.
Purpose-built models may fare better but remain untested.
