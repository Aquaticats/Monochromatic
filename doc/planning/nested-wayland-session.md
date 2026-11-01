# Planning: a tiny fake screen for testing our apps

Status:
 plan of record.
 The approach is decided,
 the risky assumptions were checked on real hardware,
 and the work is not built yet.
 Authored 2026-07-05.
 Tracks issue #272 (the main tool) and issue #273 (a harder follow-up left for later).

This document assumes you know nothing about this project,
 programming,
 or how a computer draws windows.
Every unfamiliar word is explained the first time it appears,
 and a plain-word glossary is in the Glossary
section.

## In one sentence

We are going to build a small helper program that gives one of our apps a private,
 pretend screen to draw on,
photographs what the app draws,
 and can press its buttons,
 so that automated checks can confirm the app looks
and behaves correctly,
 on any computer,
 installed through our project's normal one-command setup.

## The apps we are testing

The project builds a few desktop apps.
A desktop app is just a program with a window,
 buttons,
 and controls;
 the main one here is a music player.
Like anything with a screen,
 an app can look wrong or misbehave:
 a button in the wrong place,
 a slider that
sticks,
 artwork that fails to appear.
We want a machine to catch those problems automatically,
 over and over,
 so that a person does not have to open
the app and check it by eye after every change.

## Why checking an app's screen is surprisingly hard

Here is a picture to hold onto.
An app is like an actor,
 and to watch an actor perform you need a stage.
On this kind of computer,
 the "stage" is a background program that hands each app a rectangle to draw into and
shows it on the monitor.
The technical name for such a program is a compositor,
 and people sometimes loosely call it a window manager;
in this document we will just call it a screen program.

Two things make automated checking hard.

First,
 an app can draw in two different ways:
 a slow way done by the computer's general processor,
 and a fast,
more realistic way that uses the computer's graphics card (the dedicated chip built for drawing).
The fast graphics-card way is the one real users get,
 and an app can behave differently on it,
 so that is the
way we most want to check.

Second,
 the fast graphics-card way only works when a real stage is present.
But the machines that run our checks automatically,
 such as build servers and automated helper programs,
 usually
have no monitor and no stage at all.
So there is nothing for the app to draw on,
 and the check cannot run.

## What we do today, and why it hurts

Today,
 to give a stage to a machine that has none,
 we borrow a full,
 general-purpose screen program called niri.
It works,
 but it is a whole theater when all we need is one stage.
It is large,
 it does far more than we need,
 and on this project's computers (a locked-down flavor of Linux)
installing it requires a messy change to the whole system that our normal setup tool cannot make cleanly.
Every contributor who wants to run these checks runs into that friction.
Removing that friction is the entire reason this work exists.

## What we are going to build

A tiny,
 single-purpose stage:
 a small program that

- gives exactly one app a private pretend screen,
- keeps that app filling the whole screen,
- shuts itself down cleanly when the app closes,
- saves a photo of the screen as an image file when asked,
- can click at a chosen spot,
   press keys,
   and type text into the app,
- can change the screen size,
   and
- takes all these commands over a simple local channel and answers each with a plain success or failure.

Because we write it ourselves,
 it will install through our project's ordinary one-command setup,
 with no messy
system-wide change.
That is the whole point:
 less friction for everyone.

## Why build our own instead of using something that already exists

This was the central question,
 and we did not answer it by guessing.
We surveyed the ready-made screen programs that exist in the world,
 read their source code,
 and ran the most
promising one against our actual app on real hardware to confirm it works.

Several ready-made options would technically do the job,
 and one of them,
 called cage,
 we confirmed can host our
app correctly using the fast graphics-card path.
But every ready-made option shares one disqualifying problem:
 none can be installed by our project's one-command
setup tool without the same messy system-wide change we are trying to get rid of.
They are shipped as system software,
 not as the kind of self-contained download our setup tool understands.
A small program we write ourselves can be published as a single self-contained download that the setup tool
installs in one step.
That is the deciding factor,
 and it is why "use what already exists" does not win here.

Building our own is also less work than it sounds,
 because we are not starting from nothing.
We build on an existing,
 well-regarded open-source toolkit (named Smithay) that already provides the hard
graphics parts.
Our own share of the code is small,
 and,
 because we own the screen program,
 taking photos and pressing buttons
become simple built-in features rather than extra tools we would have to bolt on.

For the curious,
 the ready-made options we set aside,
 in plain terms:
 one is enormous and bundles video
streaming and a web browser we would never use;
 another popular one turned out not to speak the exact modern
graphics language our app uses,
 so it was a risky fit;
 and the rest are all blocked by the install problem above.

## The plan, in two stages

We are deliberately doing the easier half first.

Stage one,
 build now.
The version that works on an ordinary computer that already has a screen,
 such as a contributor's laptop or
desktop.
This is the well-trodden,
 low-risk path,
 and it is enough to run the checks during everyday development.

Stage two,
 left for later,
 tracked separately as issue #273.
The version that works on a bare machine with no screen at all,
 such as a build server in a data center (the
plain word for such a machine is headless).
This is the genuinely tricky part,
 so we are not rushing it.
Splitting it off means the useful first version does not have to wait for the hard part to be solved.

## How we will know it works

Success,
 in plain terms,
 looks like this.
One command starts our app on the pretend screen,
 saves a photo,
 clicks something or presses a key,
 changes the
window size,
 and saves a second photo,
 and the two photos show the app drawing itself correctly at both sizes
using the fast graphics-card path.
A repeating mode also exists:
 it photographs the screen every so often and produces a sequence of images.

## What this is not

- Not a replacement for a normal desktop;
   it hosts one app and nothing else.
- Not a video recorder;
   it saves still images.
- Not something a person keeps open;
   it is a checking tool that starts,
   does its job,
   and exits.

## Glossary of plain words

- App,
   or desktop app:
   a program with a window,
   such as the music player.
- Screen program:
   the background program that gives apps a rectangle to draw in and shows it on the monitor.
  Our tiny tool is a very small one of these.
  Its technical name is a compositor;
   Wayland is the name of the modern system on this kind of computer that it
  is part of;
   a window manager is a loosely related term.
- Graphics card,
   or GPU:
   the dedicated chip that draws fast,
   realistic graphics.
  The path we most want to check.
- Headless:
   a computer with no monitor attached,
   such as a server.
- Rust:
   the programming language we will write the tool in.
- Smithay:
   an existing open-source toolkit that provides the hard graphics building blocks,
   so we do not have to
  write them ourselves.
- mise:
   the single command our project uses to install everything a contributor needs.
  Whatever we build has to be installable by it;
   that requirement shaped the whole plan.
- Issue #272 and issue #273:
   the project's task tickets,
   one for the main tool and one for the harder
  no-screen version left for later.

## Where the detailed reasoning lives

This document is the plain-language overview.
The full technical decision,
 the on-hardware measurements,
 and the source reviews of every option we considered
are recorded in doc/decision/nested-wayland-session.md and the `nested-wayland-session-vet-report/` folder beside it.

The `nested-wayland-session-vet-reports/` folder name predates the 2026-07-15 singularization.
The folder became `nested-wayland-session-vet-report/` on 2026-07-15 (`bd19d57e20`),
so the `nested-wayland-session-vet-reports/` spelling no longer resolves.
