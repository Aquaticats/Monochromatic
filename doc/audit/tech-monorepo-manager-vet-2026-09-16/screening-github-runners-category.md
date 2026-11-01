# Category fit (G1) screening: general runners group

Input: `screening-runners.json`, 322 repositories.
Verdicts: 14 monorepo manager, 1 component, 307 category mismatch (20 of them marked ambiguous and quoted).
Entries keep input order, which is the discovery pass star order.

## Promoted (monorepo manager or component)

- `earthly/earthly`
  - URL: https://github.com/earthly/earthly
  - verdict: monorepo manager (borderline: per-directory Earthfiles with cross-directory target references that Earthly
  - builds first; no workspace discovery, affected selection, or run-in-all-projects command)
  - decisive quote: "Earthly is great for both monorepos and polyrepos. You can split your build logic across multiple
  - Earthfiles, placing some deeper inside the directory structure or even in other repositories. Referencing targets
  - from other Earthfiles is easy regardless of where they are stored." (README.md); "references the `html` target of
  - the directory `../html` ... Earthly executes its build for the `html` target" (examples/monorepo/README.md)
  - archived: false
- `oxequa/realize`
  - URL: https://github.com/oxequa/realize
  - verdict: component (borderline: Go-centric live-reload watcher; the HTTP interface is its web panel's websocket,
  - which pushes project state as JSON and accepts settings JSON; no relationships between projects)
  - decisive quote: "Manage multiple projects at the same time." and "--server -> Enable the web server" (README.md);
  - `e.GET("/ws", s.projects)` sends `json.Marshal(s.Parent)` and writes received JSON through
  - `s.Parent.Settings.Write` (realize/server.go)
  - archived: false
- `BobBuildTool/bob`
  - URL: https://github.com/BobBuildTool/bob
  - verdict: monorepo manager (borderline: bitbake/portage-style package build orchestrator for embedded systems, where
  - recipes define packages with dependencies; not a source-workspace task runner)
  - decisive quote: "Bob closely tracks the input of all packages. This includes all checked out sources and the
  - dependencies to other packages. If something is changed Bob can accurately determine which packages have to be
  - rebuilt." (doc/tutorial/fingerprints.rst)
  - archived: false
- `omio-labs/myke`
  - URL: https://github.com/omio-labs/myke
  - verdict: monorepo manager (borderline: discovers projects and runs a task across all or tagged projects; no
  - dependency ordering between projects)
  - decisive quote: "`myke build` runs build in all projects" and "`myke <tag>/build` runs build in all projects tagged
  - `<tag>`" (README.md)
  - archived: false
- `no0dles/hammerkit`
  - URL: https://github.com/no0dles/hammerkit
  - verdict: monorepo manager
  - decisive quote: "In a monorepo every package needs the same `install`/`build` steps, and packages depend on each
  - other in a specific order. Hammerkit handles both with includes (reuse a task definition in each package's
  - directory) and references (wire up cross-package order)." (docs/guides/monorepo.md)
  - archived: false
- `tuist/once`
  - URL: https://github.com/tuist/once
  - verdict: monorepo manager
  - decisive quote: "The Once graph describes the named parts of a workspace and what Once can do with them." and
  - "Targets live in package-level `once.toml` files" (web/priv/docs/guide/graph/index.md)
  - archived: false
- `nadlejs/nadle`
  - URL: https://github.com/nadlejs/nadle
  - verdict: monorepo manager
  - decisive quote: "Monorepo-native — first-class support for multi-package workspaces" (README.md)
  - archived: false
- `kraken-build/kraken`
  - URL: https://github.com/kraken-build/kraken
  - verdict: monorepo manager (Gradle-style project and subproject tree with task addresses; its README says the OSS
  - components are poorly documented)
  - decisive quote: under the heading "Building subprojects": "The `kraken` CLI supports running in a sub project, but
  - requires that you point it to the root of your project using the `-p,--project-dir` option."
  - (docs/docs/krakenw.md); the repository's own `.kraken.py` calls `project.subproject("kraken-build")`
  - archived: false
- `fbecart/zinoma`
  - URL: https://github.com/fbecart/zinoma
  - verdict: monorepo manager (borderline: named projects imported explicitly, with target dependencies across
  - projects; no workspace discovery; watch mode has no RPC interface)
  - decisive quote: "Imported projects need to have a name defined. This name becomes a key to the `imports` object.
  - Imported targets should be referred to with their fully qualified name: `project_name::target_name`." (CHANGELOG.md)
  - archived: false
- `metaist/ds`
  - URL: https://github.com/metaist/ds
  - verdict: monorepo manager (borderline: reads npm, uv, rye, and Cargo workspace members and runs a task in each; no
  - dependency ordering between members)
  - decisive quote: "Workspaces are a way of managing multiple sub-projects from a top-level." (docs/workspaces.md);
  - `ds --workspace '*' test   # special match that means "all workspaces"` (docs/workspaces.md)
  - archived: false
- `GriffinCanCode/bldr`
  - URL: https://github.com/GriffinCanCode/bldr
  - verdict: monorepo manager
  - decisive quote: "High-performance build system for polyglot monorepos." and "bldr query 'deps(//src:app)'"
  - (README.md)
  - archived: false
- `OctaHive/octa`
  - URL: https://github.com/OctaHive/octa
  - verdict: monorepo manager
  - decisive quote: "A root Octafile can automatically discover Octafiles in monorepo projects" and "Wildcards work
  - uniformly for both forms, such as `octa 'packages:*:build'`" (README.md)
  - archived: false
- `Jomy10/beaver`
  - URL: https://github.com/Jomy10/beaver
  - verdict: monorepo manager (borderline: C/C++-centric build system with named projects, including imported CMake,
  - Cargo, SwiftPM, and Meson projects, and target dependencies across projects)
  - decisive quote: "Dependencies from other projects are referred to using the project:target syntax"
  - (examples/multi-project/beaver.rb); "Beaver also integrates with other build systems so you can have dependencies
  - that use other build systems." (README.md)
  - archived: false
- `meta-company/makex`
  - URL: https://github.com/meta-company/makex
  - verdict: monorepo manager (borderline: a Bazel-like workspace root with `//path` task references across Makex
  - files; no per-project units)
  - decisive quote: "Workspaces define the roots or boundaries of projects or a repository. Workspaces use the special
  - `//` prefix marker in paths to refer to Tasks consistently in a workspace" (documents/source/workspaces.md); README
  - features list "Dependency Graphs" and "Workspaces"
  - archived: false
- `gregnazario/must`
  - URL: https://github.com/gregnazario/must
  - verdict: monorepo manager
  - decisive quote: "Demonstrates: Multi-package monorepo with cross-package dependencies, per-package workdir, and
  - umbrella recipes." with layout `apps/api/Mustfile.toml`, `apps/web/Mustfile.toml`, `libs/core/Mustfile.toml`
  - (examples/monorepo-workspace/Mustfile.toml)
  - archived: false

## Category mismatch

- `go-task/task`: Makefile-style task runner; includes only import Taskfiles, with no project graph (ambiguous): "If
- you want to share tasks between different projects (Taskfiles), you can use" includes
- (website/src/latest/docs/guide.md)
- `cake-build/cake`: C# build automation scripts without a workspace model
- `SCons/scons`: file-level Make-style build tool
- `pydoit/doit`: file-dependency task runner for one project
- `nat-n/poethepoet`: Python project task runner; monorepo support is include files, with no project graph (ambiguous):
- "collect tasks from multiple projects into one ... organize your code in a monorepo" via include files
- (docs/guides/include_guide.rst)
- `apenwarr/redo`: Make-style file build tool
- `stepchowfun/toast`: containerized task runner for one repository
- `jacobdeichert/mask`: Markdown-defined command runner
- `joerdav/xc`: Markdown-defined command runner
- `sigoden/argc`: Bash CLI framework and command runner
- `wagoodman/bashful`: YAML command sequencer
- `dotnetcore/FlubuCore`: C# build and deployment scripts
- `cirocosta/cr`: concurrent task runner without a project model
- `pypyr/pypyr`: pipeline runner
- `jolicode/castor`: PHP task runner
- `thomhurst/ModularPipelines`: C# CI pipeline definitions
- `go-godo/godo`: Go task runner with a file watcher but no RPC, IPC, or HTTP interface
- `TekWizely/run`: Runfile script runner
- `xonixx/makesure`: Make-style task runner
- `aappleby/hancho`: single-project Python build system ("Hancho should suffice for small to medium sized projects")
- `taskctl/taskctl`: Make alternative with watchers but no RPC, IPC, or HTTP interface
- `dnephin/dobi`: Docker build automation
- `zaaack/foy`: Node task runner
- `rliebz/tusk`: YAML task runner
- `opsxcq/tasker`: Docker-based task runner
- `dreadl0ck/zeus`: single-project Make alternative whose web interface is unfinished (ambiguous): "The Webinterface
- will allow to track the build status ... NOTE: This is still work in progress"
- `zyedidia/knit`: Make/mk-style build tool
- `gulien/orbit`: command runner with file templating
- `ali77gh/bake-rs`: script runner
- `ejholmes/walk`: Make/redo-style target graph
- `go-gilbert/gilbert`: task runner for Go projects
- `pawamoy/duty`: Python task runner
- `dazuma/toys`: Ruby command-line tool framework
- `sakejs/sake-cli`: JavaScript Make-style build tool
- `nur-taskrunner/nur`: Nushell task runner
- `lets-cli/lets`: Make alternative CLI task runner
- `dogtools/dog`: task runner
- `sschmid/bee`: plugin-based Bash automation
- `droundy/fac`: file-tracking Make-style build system
- `kt3k/saku` (archived): Markdown task runner
- `faqtor/faqtor`: Node promise-based task runner
- `leopardslab/dunner`: Docker task runner
- `kt3k/node-saku` (archived): Markdown task runner
- `scriptype/salinger`: npm scripts companion task runner
- `sagebind/rote` (archived): Lua task runner
- `kcmerrill/alfred`: task runner and automator
- `devrc-hub/devrc`: YAML task runner
- `shannonmoeller/ygor`: Node task toolkit
- `suzuki-shunsuke/cmdx`: task runner with prompts
- `jez/bask`: Bash task runner
- `sakerbuild/saker.build`: language-agnostic build system whose daemon serves only its own builds (ambiguous): "you
- can optionally use a build daemon for your builds ... they keep data in memory and watch the file system for changes"
- `upcmd/up`: provisioning and workflow automation CLI
- `cesar-douady/open-lmake`: Make-like build system
- `jjzcru/elk`: YAML task runner
- `jasonwhite/button` (archived): file-level build system
- `yourbase/yb`: per-project build environment tool
- `zuke-build/zuke`: Deno build automation scripts
- `DannyBen/runfile`: Ruby command-line runner
- `doowb/composer`: JavaScript task library with watch but no RPC, IPC, or HTTP interface
- `omnilib/thx`: Python project command runner
- `stylemistake/juke-build`: JavaScript DSL build system
- `JetBrains/teamcity-csharp-interactive`: C# build scripts
- `jiro4989/monit`: task runner with file watch but no RPC, IPC, or HTTP interface
- `ras0q/cute`: Markdown command runner
- `marghidanu/werk`: local CI pipeline runner
- `DevTeam/csharp-interactive`: .NET build automation scripts
- `lupincr/lupin`: Crystal task runner
- `daelvn/alfons`: Lua task runner
- `theMackabu/maid`: local maidfile task runner
- `ludicroushq/frunk`: npm scripts orchestrator within one package.json
- `EmmaTheMartian/clockwork`: language-agnostic task build tool
- `cdaringe/rad`: TypeScript build tool with make-style tasks
- `rumkin/bake`: Bash task runner
- `FakeBuild/Xake`: F# Make implementation
- `NathanVaughn/vscode-task-runner`: runs .vscode/tasks.json
- `adhamsalama/yasta`: Python task runner
- `makim-org/makim`: YAML Make alternative
- `eobrain/bajel`: Make-modeled build system
- `Junker/faber`: Scheme task runner
- `frissyn/pyke`: Make-like Python utility
- `team23/b5`: agency-wide per-project task runner
- `bab-sh/bab`: Taskfile-style runner with namespaced includes only (ambiguous): "Import tasks from other Babfiles
- (namespaced)"
- `s4m-mo/sandcastle`: YAML task runner
- `state-alchemists/zrb`: automation framework whose server is a task web UI, not a watch process (ambiguous): "zrb
- server start ... to see your tasks in a clean, user-friendly interface"
- `adamralph/simple-targets-csx` (archived): C# target runner
- `alecthomas/bit`: build tool with target dependencies
- `pyrustic/backstage`: scripting language and task automation
- `harehare/mq-task`: Markdown task runner
- `cwbaker/forge`: Lua build tool
- `shrayasr/Iko`: command-line task runner
- `FollowTheProcess/spok` (archived): Make-style build system
- `mufancom/biu`: multi-task launcher UI without a workspace model or RPC interface
- `diskuv/dk`: package fetch and build system for distributing source
- `pinefile/pine`: Node task runner
- `renatoathaydes/dartle`: Dart build system
- `sigoden/runme`: deprecated shell-script task runner
- `fkrauthan/nss-run`: Node build tool
- `zakuro9715/z`: task runner
- `mikosik/smooth-build`: functional build language
- `wrapl/rabs`: imperative build system
- `metaory/mxflow-cli`: task runner
- `FabienArcellier/alfred-cli`: subprojects only namespace commands, with no cross-project runs or relationships
- (ambiguous): "At the root of the project, you will have access to all the commands of all the subprojects using the
- subproject name ``alfred project1 ci``" (docs/source/benefits.rst)
- `giann/fourmi` (archived): Lua task runner
- `gobuffalo/grift`: Go task runner
- `riotkit-org/riotkit-do` (archived): Makefile/Gradle-style task executor
- `darkobits/nr`: JavaScript project task runner
- `koddr/yatr`: per-project Makefile replacement
- `egordm/nauman`: local job automation
- `melbahja/ron`: Bash task runner
- `quake-build/quake-old` (archived): Nushell meta-build system with no documented project model (ambiguous): "quake's
- documentation is currently undergoing a rewrite"
- `TheOnlyMrCat/runscript`: project build and run command manager
- `Zemerik/Task-Runner`: CLI task runner
- `areller/denogent`: TypeScript build pipeline
- `tamp-build/tamp`: .NET build automation framework
- `erikgiovani/denosk`: Deno task runner
- `adrianmrit/yamis` (archived): task runner
- `joarhal/piperig`: YAML pipeline runner
- `thomasleese/mo` (archived): YAML task runner
- `barraq/spinr`: Node task runner
- `MartinHelmut/litr`: language-independent task runner
- `3846masa-archived/memi` (archived): JavaScript task runner
- `function61/turbobob`: container-based build tool
- `rv178/baker`: build automation tool
- `PaulThompson/dnit`: Deno task executor
- `fn-go/fn`: Fnfile function runner
- `habilyildirim/Enmafile`: Make-like build system
- `xpybuild/xpybuild`: Python build-file build system
- `iPeluwa/rush`: single-file runner whose monorepo example hand-writes root tasks (ambiguous): "Monorepo Development"
- section defines tasks such as `cmd: npm run build --workspace=frontend`
- `frozzare/max`: YAML task runner
- `gouline/molot`: execution orchestrator
- `Fuzzlix/omm`: Lua make tool
- `buildcharts/buildcharts`: CI pipeline generator
- `defenseunicorns/maru2`: task runner
- `lyova24/wrkit`: YAML task runner
- `gird-dev/gird`: Make-like Python task runner
- `Serpent-Tools/serpentine`: workflow runner
- `fasibio/gomake`: Make alternative
- `runok-cli/runok`: npm scripts replacement
- `palfrey/tuvix` (archived): archived experimental engine with no documented workspace model (ambiguous):
- "Experimental hermetic build engine with Starlark"
- `lesiw/ops`: SDLC framework in Go
- `thomas3577/tano`: Deno task runner
- `azutoolkit/topia`: task composer with watcher but no RPC, IPC, or HTTP interface
- `ProPuke/boop`: build system
- `madjam002/nix-task`: Nix task and CI pipeline runner
- `natnat-mc/moonbuild`: make-style build script
- `Baldomo/makesh`: Bash Makefile-alike
- `reproducible-reporting/stepup-core`: dynamic workflow build tool
- `nuvrel/errand` (archived): task runner
- `wislertt/bakefile`: Python Makefile replacement
- `verifyica-team/pipeliner` (archived): local CI pipeline runner
- `mikeleppane/uvtx`: Python task runner with watch but no RPC, IPC, or HTTP interface
- `iAmNathanJ/devo`: Deno task runner
- `schmich/runx`: Ruby task runner
- `Azuyamat/pace`: Go task runner
- `skarfacegc/Gue`: Node shell task runner
- `upsight/ron`: Go build task CLI
- `CorvidLabs/fledge`: per-project dev-loop CLI
- `Joxit/runtasktic`: task sequencing with notifications
- `jjangga0214/haetae`: incremental runner that selects affected files, not projects (ambiguous): "Running only
- affected test files ... with language-specific automatic dependency graph detection" (README of haetae-org/haetae,
- the move target)
- `SujalChoudhari/Forge`: Make and GitHub Actions hybrid
- `getpipe-dev/pipe`: shell pipeline workflow
- `mdops-org/mdops-cli`: README-driven tasks
- `grablyhq/grably`: Rake-based build pipelines
- `nullbadger/neomake`: Make alternative
- `MaxChip101/Makeup`: build system
- `AcrylicShrimp/piped`: pipeline orchestrator
- `quake-build/quake`: Nushell meta-build system rewrite with no documented project model (ambiguous): "quake is a
- meta-build system powered by Nushell"
- `ForNeVeR/Meganob`: F# task-dependency build system
- `jakegut/yabs`: build system
- `ChrisMcKenzie/achieve`: development task automation
- `willemkokke/footman` (archived): Python task runner
- `mistweaverco/kimbia`: task runner
- `Noxsios/vai` (archived): task runner
- `moseschmiedel/jarvis`: task runner
- `YuKitsune/plz`: Make alternative
- `battila7/jockey`: task runner
- `UglyEgg/podCI`: containerized job runner
- `kcmerrill/mario`: task runner and build system
- `paip-web/pwbs`: automation build system
- `HelgeSverre/jake`: Make and Just style command runner
- `CoolOppo/make-ultra`: task runner
- `kevgo/atalanta`: per-directory stack task launcher
- `jamielsharief/task-runner`: project task runner
- `SierraSoftworks/Executor`: script task runner
- `Jakkoble/HexaTask`: containerized job runner prototype; its gRPC links its own services and nothing watches files
- (ambiguous): "This project is explicitly designed as an architectural prototype"
- `Ayehavgunne/mog`: task runner
- `jharrilim/runt`: polyglot task CLI generator
- `neural-chilli/fkn`: single repo-local task file (ambiguous): "one file defines how the repo is built, checked, run,
- and exposed to agents"
- `Unviray/pun`: Python task runner
- `kanarus/cargo-metask`: runs tasks from one Cargo.toml metadata table, with no documented cross-member runs
- (ambiguous): "Cargo task runner for {package, workspace}.metadata.tasks"
- `fezcode/gobake`: Go Recipe.go build orchestrator
- `nsrosenqvist/croft`: single-project dev-loop tool; its watch mode and TUI have no RPC interface (ambiguous): "Croft
- is a per-project dev-loop tool"
- `alloc/picorun`: parallel task runner
- `trinhminhtriet/tash`: YAML local and remote task runner
- `RealOrangeOne/pike`: task runner
- `alexreg/factotum`: Python task runner
- `lleyton/fae`: npm-scripts-compatible task runner
- `aiopy/python-uvtask`: pyproject.toml script runner
- `MrMaxie/bunbun`: Node task runner
- `brad-jones/drun`: Dart task runner
- `ricardobeat/taks`: JavaScript task runner
- `NWelde/better-ci`: CI pipeline engine that selects jobs by path, with no project model (ambiguous): "Jobs declare
- which files they care about. `--git-diff` runs only the jobs whose source files actually changed"
- `vincbro/godo`: Make-inspired build system
- `0918nobita/lets` (archived): experimental meta-build system
- `jeanlauliac/upd`: file update build tool
- `sn/ntask`: Python DAG task runner
- `SebTardif/MiniBuildRust`: Make-style parallel build system
- `ankurdubey521/Build-Automation-System`: build-rule executor
- `ysufender/Efile`: Makefile alternative
- `daedalus-os/erebrus`: Make-like Ruby build system
- `lfknudsen/Forge`: GNU Make alternative
- `Hexcell/HXMK`: Makefile-like Python build system
- `redthing1/wox`: make replacement recipe system
- `kaiserthe13th/makeup`: project build tool
- `GandelXIV/dhall-build-system`: build system
- `SquareRoundCurly/Pragmatic_build_system`: build system with a file dependency graph
- `love-your-parens/task-runner`: Clojure task runner
- `ParadoxicalSerenity/TypeTasker`: TypeScript task runner
- `lumenghz/ti`: task runner
- `soren-n/tickle`: task graph workflow tool
- `nakkiy/rhask`: Rhai task runner
- `twocaretcat/NoBS`: Node task runner
- `mrfootoyou/PSTaskFramework`: PowerShell build automation
- `manuel2258/instruct`: makefile-like task language
- `swarmnyc/Swarm.TaskRunner`: task execution CLI
- `JakesMD/hobnob`: YAML task runner
- `suzulabo/ttscripts`: TypeScript and JavaScript task runner
- `mgutz/task`: no-config task runner
- `btvoidx/L`: Lua task runner
- `green-threads/ndo`: CLI recipe runner
- `mitranim/gtg`: Go task group library
- `ckotzbauer/node-task-runner` (archived): Node task runner
- `abrunner94/task-rs`: script workflow runner
- `lask-task-runner/lask`: containerized typed task runner
- `withinboredom/tyche`: task runner
- `sirikon/ebro` (archived): task runner
- `or1can/ratect`: Batect reimplementation for containerized tasks
- `iamBijoyKar/zap`: YAML task runner
- `lepisma/orgo` (archived): Org-mode task runner
- `jassielof/weld` (archived): build automation CLI
- `urban233/pymake`: Python task runner
- `eight04/pyXcute`: npm-scripts-like task runner
- `akluth/do`: shell command processor
- `lukecarr/brec`: Bun task runner
- `mitranim/jtg`: JavaScript task group
- `tsk-monster/tsk`: task runner
- `leso-kn/buildtool.js`: task runner
- `ecma-make/ecmake`: JavaScript make
- `jgrey4296/doot`: doit fork
- `Hand-of-Doom/budgie`: task runner
- `neural-chilli/qp`: task runner whose MCP server is dormant (ambiguous): "This package is intentionally disabled at
- the product level for now." (internal/mcp/README.md)
- `hopenbuild/App-hopen`: build system and task runner
- `nksaraf/elf`: YAML task runner
- `matanlurey/chore.dart` (archived): task runner
- `kaliv0/koi_fish`: CLI task runner
- `hashmap-kz/smallci`: local CI runner
- `quonaro/Lota`: YAML task runner
- `Aldlevine/scripteasy` (archived): script runner
- `anantix-network/carpenter-cli`: project script runner
- `MRThugh/SAZA`: Python task-based build system
- `Qix-/tag` (archived): task runner and build specification language
- `tillahoffmann/cook-build-archive` (archived): task-centric build system
- `antonsynd/chiri`: per-package build CLI wrapper (ambiguous): "I don't plan to expand its feature set to dependency
- modeling."
- `aszecsei/please`: task runner
- `mattritterspach-personal/flowrunner`: DAG task runner
- `trashify/ok-runner`: task runner
- `AmaseCocoa/libretto`: pyproject.toml task runner
- `tmus/gake`: Go task runner
- `mhio/mash`: shell task runner
- `heinthanth/xuerun`: make-like task runner
- `tidjee-dev/doit`: Go task runner
- `troykinsella/friggen`: task runner
- `fboender/sla`: simple automator
- `freddiefujiwara/yoboo`: YAML task runner
- `kinematic-ci/machinery`: task automation web platform, not a watch process (ambiguous): "A general purpose task
- automation platform"
- `CodeTease/p`: task runner
- `cognis-digital/taskforge`: declarative task runner
- `lepton9/ztask`: workflow task runner
- `NathanFirmo/tsk` (archived): Make alternative
- `HelperVia/hv-cli`: per-project command runner
- `pirakansa/Vorbere`: file sync and task runner
- `jrop/pkgbuild`: build task runner
- `kijimad/gorun`: GitHub Actions-syntax task runner
- `nadvotsky/sharpbuilder` (archived): C# task runner
- `quike/keepup`: task runner
- `andersnormal/picasso` (archived): task runner
- `aprilahijriyan/vonzy`: task runner
- `mikey-t/swig`: series and parallel task runner
- `githuib/powerchord`: concurrent task runner
- `hxii/boku`: sequential YAML task runner
- `korchasa/taskman` (archived): Go Makefile replacement
- `Nikoro/spellbook`: project command shim
- `khalyomede/fang` (archived): parallel task runner
- `aca/qwer`: Markdown command runner
- `rosshhun/wandler`: Python YAML task runner
- `AdametherzLab/ts-task`: TypeScript task runner
- `importt-ant/pi-line`: parallel script runner
- `triole/coda`: task runner
- `doxuta/twap`: project-local command runner
- `chriso345/jot`: task runner
- `ubgo/lath`: Go-typed task and deploy runner
- `DarkWiiPlayer/spooder`: Lua task runner
- `JeffDess/lets`: Nix and Bash task runner
- `maate/task-friend`: CLI task runner
- `MohammedAl-Mahdawi/wooz`: task automation CLI
- `Pekhov14/tony`: make-inspired task runner
- `lupincr/lupin-cli`: Crystal task runner CLI

## Method notes

- Metadata:
  `gh api repos/<owner>/<repo>` for all 322 repositories, 6 concurrent requests, via `runners-g1/fetch-meta.ts`.
  Every call succeeded: no 404s, no errors, no renamed `full_name`.
  31 of the 322 are archived.
  `jjangga0214/haetae` resolves, but its README is only a pointer to `haetae-org/haetae`, which was read instead.
- Decision basis:
  the description and topics decided the clear cases.
  The discovery notes in `discovery-github.md` were used only as hints.
- READMEs:
  51 fetched with `gh api repos/<o>/<r>/readme`, all without errors.
  42 were read in full or through their relevant head.
  9 long ones (`earthly/earthly`, `state-alchemists/zrb`, `iPeluwa/rush`, `OctaHive/octa`, `neural-chilli/fkn`,
  `NWelde/better-ci`, `gregnazario/must`, `mikeleppane/uvtx`, `dreadl0ck/zeus`) were searched for workspace, project,
  watch, daemon, server, RPC, and MCP terms, and the matching sections were read.
- Beyond READMEs:
  some READMEs did not settle the question, so the following repository files were read through `gh api`:
  `go-task/task` `website/src/latest/docs/guide.md` (search only),
  `earthly/earthly` `examples/monorepo/README.md`,
  `no0dles/hammerkit` `docs/guides/monorepo.md`,
  `metaist/ds` `docs/workspaces.md`,
  `meta-company/makex` `documents/source/workspaces.md`,
  `Jomy10/beaver` `examples/multi-project/beaver.rb`,
  `gregnazario/must` `examples/monorepo-workspace/Mustfile.toml`,
  `tuist/once` `web/priv/docs/guide/graph/index.md`,
  `kraken-build/kraken` `docs/docs/index.md` and `docs/docs/concepts/tasks.md`,
  `haetae-org/haetae` `packages/docs/pages/index.mdx`,
  and `oxequa/realize` `realize/server.go`, to check what its HTTP interface does.
- Code search:
  20 `search/code` queries through `runners-g1/code-search.ts`, with a 7-second gap between queries.
  3 queries returned 0 even though the repository contains the term:
  `gregnazario/must` for `monorepo` (the example `Mustfile.toml` says "monorepo") and for `workspace`, and
  `haetae-org/haetae` for `monorepo` (its docs index says "monorepo").
  GitHub's code search index is therefore incomplete for some repositories, so a 0 result was never used as evidence of
  absence.
  Related files were read directly instead.
- Borderline rule, applied the same way to every repository:
  - promote when the tool's own docs describe multiple projects, packages, or directory units as a supported layout,
    and the tool can run work across those units (dependencies between units' tasks, running a task in every member,
    affected selection, or a graph);
  - a generic include or import feature without that multi-project framing counts as a mismatch (`go-task/task`,
  - `bab-sh/bab`);
  - `nat-n/poethepoet` is the closest mismatch and the call the rule separates least cleanly.
    Its docs do mention monorepos, but the documented mechanism copies another file's task definitions into one
    project's task list,
    optionally with a separate `cwd`.
    It documents no project units, no fan-out across members, and no dependencies between projects.
    Revisit it if include-based composition should count;
    `go-task/task` and `bab-sh/bab` would then need the same review;
  - "borderline" in a verdict names the missing piece:
    `metaist/ds` and `omio-labs/myke` run tasks across projects but have no dependency order between projects,
    and `earthly/earthly` and `meta-company/makex` link targets by path rather than modeling projects.
  - The component category was applied only to a persistent watch process that has an RPC, IPC, or HTTP interface.
    Watch modes without such an interface (`fbecart/zinoma`, `go-godo/godo`, `taskctl/taskctl`, `mikeleppane/uvtx`, and
    others) did not qualify.
    Servers that are not watch processes (`state-alchemists/zrb`, `kinematic-ci/machinery`, `Jakkoble/HexaTask`) did
    not qualify.
    A dormant MCP server (`neural-chilli/qp`) did not qualify.
- Nothing was installed or run.
  Nothing under the Monochromatic worktree was modified;
  the scratch scripts and intermediate files are in `scratchpad/runners-g1/`.
