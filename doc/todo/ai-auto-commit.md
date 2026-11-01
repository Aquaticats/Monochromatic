# TODO: AI Auto-Commit Automation

Build an intelligent auto-commit system with debouncing and AI-generated commit messages
to replace manual "forgot what changes" commits.

## Problem Statement and Motivation

### Current Approach (Broken Workflow)

The current development workflow suffers from a fundamental commit discipline problem:

1. **Extended Development Sessions**:
    Work for hours/days/weeks without committing
2. **Memory Loss**:
    By the time a commit happens,
    the specific changes and reasoning are forgotten
3. **Generic Commit Messages**:
    Resort to meaningless messages like "forgot what changes",
    "various updates",
    "WIP"
4. **Lost Development History**:
    No granular record of the development process or decision-making
5. **Recovery Challenges**:
    When things break,
    no logical restore points to analyze or revert to
6. **Code Review Impossibility**:
    Massive commits are impossible to review effectively

### Specific Pain Points

- **Decision Fatigue**:
   "Should I commit now?
  " paralysis prevents regular commits
- **Message Crafting Overhead**:
   The mental energy required to write good commit messages
- **Context Switching**:
   Interrupting flow state to commit feels disruptive
- **Perfectionism**:
   Waiting for "logical units" that never materialize in organic development
- **Fear of Noise**:
   Concern that frequent commits will pollute the Git history

### Why AI Auto-Commits Are The Solution

This approach addresses the core psychological and workflow barriers:

1. **Eliminates Decision Fatigue**:
    No manual "should I commit?
   " decisions
2. **Preserves Context**:
    AI can analyze diffs while changes are fresh
3. **Maintains Flow State**:
    Automatic background process doesn't interrupt coding
4. **Better Than Nothing**:
    Even generic AI messages > "forgot what changes"
5. **Granular History**:
    Recovery points every few minutes instead of massive commits
6. **Upgrade Path**:
    Provides foundation for gradually improving commit quality

### Expected Outcomes

- **90% Reduction** in "forgot what changes" commits
- **Automated Backup System** preventing work loss from crashes/mistakes
- **Browsable Development Timeline** instead of archaeological mega-commits
- **Improved Code Review** through logical,
   smaller changesets
- **Reduced Stress** around version control discipline
- **Foundation for Better Practices** as commit habits improve over time

### Success Metrics

- Average commit size drops from 500+ lines to <100 lines
- Time between commits drops from days/weeks to minutes/hours
- Commit messages contain actual semantic information about changes
- Zero "forgot what changes" or equivalent meaningless messages
- Recovery/debugging becomes feasible through granular history

## Current State Analysis

### Existing Git Habits

- **Frequency**:
   Commits happen every few days to weeks
- **Size**:
   Typically 300-1000+ line changes across multiple files
- **Messages**:
   Generic,
   non-descriptive ("updated stuff",
   "changes",
   "forgot what changes")
- **Review Process**:
   Impossible due to commit size and lack of context
- **Recovery**:
   Difficult due to large time gaps between commits

### Development Environment

- **Project Structure**:
   Monorepo with mise task orchestration
- **Language**:
   TypeScript with strict typing
- **Tools**:
   VSCode,
   mise,
   Bun,
   existing linting/testing infrastructure
- **Workflow**:
   Rapid iterative development with frequent file switching
- **Session Length**:
   Extended coding sessions (2-8 hours common)

### Integration Requirements

- **Must Not Interrupt Flow**:
   Zero disruption to active development
- **Performance**:
   Minimal system resource usage
- **Reliability**:
   Must work consistently without manual intervention
- **Compatibility**:
   Work alongside existing tools and workflows
- **Configurability**:
   Adaptable to different project types and preferences

## Phase 1: Basic File Watching and Debouncing

- [ ] **Research file watching solutions**
  - Evaluate `fswatch` (cross-platform) vs `inotifywait` (Linux) vs Node.
    js `chokidar`
  - Test performance with large codebases
  - Assess filtering capabilities (ignore node_modules,
     dist,
     etc.)

- [ ] **Implement debouncing logic**
  - Create configurable debounce delay (default:
     3 minutes)
  - Handle multiple rapid changes within debounce window
  - Reset timer on new file changes
  - Add manual trigger override

- [ ] **Basic commit automation**
  - Auto-stage all changes (`git add -A`)
  - Generate timestamp-based commit messages as fallback
  - Handle git repository detection
  - Skip commits when no actual changes exist

## Phase 2: AI-Powered Commit Messages

- [ ] **Implement diff analysis**
  - Extract meaningful diffs using `git diff --cached`
  - Filter out noise (whitespace-only changes,
     generated files)
  - Identify file types and change patterns
  - Detect new files vs modifications vs deletions

- [ ] **AI integration options**
  - **Option A**:
     GitHub Copilot CLI integration (`gh copilot suggest -t git`)
  - **Option B**:
     OpenAI API with custom prompts
  - **Option C**:
     Local LLM integration (ollama,
     etc.)
  - **Option D**:
     Rule-based message generation with AI fallback

- [ ] **Message quality improvements**
  - Follow Conventional Commits format (`feat:`,
     `fix:`,
     `refactor:`,
     etc.)
  - Detect scope from file paths (e.g.,
     `feat(module-es): add string utilities`)
  - Include relevant context from code changes
  - Fallback to generic messages when AI fails

## Phase 3: Configuration and Safety

- [ ] **Configuration system**
  - Create `.ai-commit.config.json` for per-project settings
  - Configure debounce delays,
     ignore patterns,
     AI providers
  - Set commit message templates and fallbacks
  - Enable/disable per file type or directory

- [ ] **Safety mechanisms**
  - Detect when user is actively typing (prevent interruptions)
  - Skip commits during active terminal sessions
  - Backup branch creation before auto-commits
  - Rollback capabilities for bad auto-commits

- [ ] **Integration with existing workflows**
  - Respect `.gitignore` and custom ignore patterns
  - Detect and preserve existing commit hooks
  - Handle merge conflicts and rebases gracefully
  - Work with monorepo structure (Bun workspace)

## Phase 4: Advanced Features

- [ ] **Smart commit squashing**
  - Group related auto-commits before push
  - Detect logical boundaries (test runs,
     build completions)
  - Pre-push hook to review and squash commits
  - Interactive mode for commit review

- [ ] **Development session awareness**
  - Detect testing sessions (don't commit during test runs)
  - Pause during build processes
  - Resume after development tool exits
  - Integration with VSCode/IDE status

- [ ] **Commit quality metrics**
  - Track AI message quality over time
  - Learn from manual commit message corrections
  - Improve prompts based on project patterns
  - Generate periodic reports on auto-commit effectiveness

## Phase 5: Deployment and Integration

- [ ] **Packaging and distribution**
  - Create standalone executable or npm package
  - Cross-platform compatibility (Linux,
     macOS,
     Windows)
  - Easy installation and setup process
  - Integration with project's existing tooling

- [ ] **Documentation and examples**
  - Setup instructions for different development environments
  - Configuration examples for various project types
  - Best practices and troubleshooting guide
  - Integration with mise/Bun workspace and existing tools

- [ ] **Testing and validation**
  - Unit tests for core logic
  - Integration tests with real git repositories
  - Performance testing with large codebases
  - User acceptance testing with the existing workflow

## Technical Architecture Decisions

### File Watcher Implementation

- Use Node.
  js with `chokidar` for cross-platform compatibility
- Leverage existing TypeScript tooling in the project
- Integrate with mise task system for consistency

### AI Provider Strategy

- Start with GitHub Copilot CLI (already authenticated)
- Add OpenAI API as secondary option
- Include rule-based fallback for reliability
- Make provider selection configurable

### Storage and State

- Use simple JSON files for configuration
- Store commit history metadata for learning
- Maintain lightweight state between sessions
- No external databases required

## Integration Points

- [ ] **mise workspace integration**
  - Add as a new package under `package/build/ai-commit/`
  - Use existing TypeScript configuration
  - Leverage shared utilities from `module-es`
  - Follow project's coding standards

- [ ] **Git hooks integration**
  - Install as optional pre-commit hook
  - Respect existing hook configurations
  - Provide easy enable/disable mechanism
  - Work alongside existing quality checks

## Success Criteria

- [ ] Reduces "forgot what changes" commits by 90%
- [ ] Generates meaningful commit messages 80% of the time
- [ ] Operates without interrupting development flow
- [ ] Easy to set up and configure for new projects
- [ ] Provides clear upgrade path from current manual workflow

## Risk Mitigation

- [ ] **Data loss prevention**
  - Never destructive operations without explicit confirmation
  - Automatic backup creation before major operations
  - Clear rollback procedures documented
  - Git reflog integration for recovery

- [ ] **Performance impact**
  - Minimal CPU usage during file watching
  - Efficient diff processing for large changes
  - Rate limiting for AI API calls
  - Configurable resource limits

- [ ] **Integration conflicts**
  - Detect and avoid conflicts with existing Git workflows
  - Graceful degradation when tools are unavailable
  - Clear error messages and troubleshooting guidance
  - Optional modes for different development scenarios
