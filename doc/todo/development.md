# Development Environment Todo

## Git Workflow and Branch Management

### Branch Strategy

**Branch Structure**:

- `main`:
   Protected production-ready branch
  - All code in main should be stable and deployable
  - Direct pushes are prevented via branch protection rules
  - Changes only land here via pull requests from `dev`
- `dev`:
   Active development branch
  - Default branch for all development work
  - All new features and fixes go here first
  - Regularly merged to `main` via pull requests

### Daily Development Workflow

#### Working on Dev Branch

**Standard workflow**:

```bash
# Switch to dev branch
git checkout dev

# Pull latest changes
git pull origin dev

# Make your changes, then commit
git add .
git commit -m "feat(scope): description"

# Push to dev
git push origin dev
```

#### Creating Pull Requests to Main

**When ready to release changes to main**:

```bash
# Ensure dev is up to date
git checkout dev
git pull origin dev

# Create PR using GitHub CLI (if available)
gh pr create --base main --head dev --title "Release: description" --body "Changes being merged to main"

# Or create PR via GitHub web UI
# Visit: https://github.com/Aquaticat/Monochromatic/compare/main...dev
```

#### Feature Branch Workflow (Optional)

For larger features or experimental work:

```bash
# Create feature branch from dev in its own worktree
git checkout dev
git pull origin dev
git worktree add -b feature/my-feature ../Monochromatic-feature-my-feature dev
cd ../Monochromatic-feature-my-feature

# Work on feature, commit changes
git add .
git commit -m "feat(scope): implement feature"

# Push feature branch
git push origin feature/my-feature

# Create PR to merge feature into dev
gh pr create --base dev --head feature/my-feature --title "feat: my feature" --body "Description"
```

### Branch Protection Configuration

**Main branch protection rules** (configured in GitHub web UI):

1.  Navigate to:
     Repository Settings → Branches
2.  Add branch protection rule for `main`
3.  Enable:
     "Require a pull request before merging"
4.  Optional settings to consider later:
    - Require approvals
    - Require status checks to pass
    - Require conversation resolution
    - Require branches to be up to date

**Current configuration**:
 Prevent direct pushes only

### Initial Setup Commands

**Create dev branch** (one-time setup):

```bash
# Create dev branch from current main in its own worktree
git checkout main
git pull origin main
git worktree add -b dev ../Monochromatic-dev main
cd ../Monochromatic-dev
git push -u origin dev
```

**Switch default branch** (optional,
 via GitHub web UI):

1.  Go to:
     Repository Settings → General → Default branch
2.  Change from `main` to `dev`
3.  This makes `dev` the default for new clones and PRs

### Common Scenarios

#### Hotfix to Main

For critical production fixes:

```bash
# Create hotfix branch from main in its own worktree
git checkout main
git pull origin main
git worktree add -b hotfix/critical-fix ../Monochromatic-hotfix-critical-fix main
cd ../Monochromatic-hotfix-critical-fix

# Make fix and commit
git add .
git commit -m "fix: critical production issue"

# Push and create PR to main
git push origin hotfix/critical-fix
gh pr create --base main --head hotfix/critical-fix

# After merging to main, merge main back into dev
git checkout dev
git pull origin dev
git merge main
git push origin dev
```

#### Sync Dev with Main

If main has changes that dev needs:

```bash
git checkout dev
git pull origin dev
git merge main
git push origin dev
```

### Troubleshooting

**Cannot push to main**:

```bash
remote: error: GH006: Protected branch update failed
```

This is expected.
 Create a PR instead of pushing directly.

**PR shows too many commits**:
Ensure you're comparing the right branches (dev → main,
 not feature → main).

**Merge conflicts**:

```bash
# Pull latest from both branches
git checkout dev
git pull origin dev
git pull origin main

# Resolve conflicts, then commit
git add .
git commit -m "chore: resolve merge conflicts"
git push origin dev
```

## IDE and Editor Configuration

### IDE Integration Improvements

1.  **Set up IDE integration**:
    - Configure format-on-save for all developers
    - Ensure TypeScript language server is properly configured
    - Document recommended VS Code extensions in the project

## Development Tools and Environment

### Performance Monitoring

#### File Operation Performance

- Monitor file watching performance
- Benchmark development server startup times
- Track hot reload performance

#### Development Server Optimization

- Configure efficient file watching patterns
- Set up proper caching strategies
- Monitor memory usage during development

### Environment Validation

#### Setup Validation Scripts

- **validateSetup task**:
   Successfully implemented to help diagnose environment issues
- **Validation scripts**:
   checkTools,
   checkDependencies,
   checkBuild,
   checkGitHooks are working correctly

#### Fresh Clone Verification

Ensure that fresh clones of the repository work correctly when users follow the setup instructions in `README.md`.

**Current Setup Instructions Being Tested**:

```bash
# Run project setup and build
mise run prepareAndBuild
```

### Development Workflow Improvements

#### Hot Reload and File Watching

- Optimize file watching patterns for better performance
- Configure proper excludes for node_modules and build outputs
- Monitor CPU usage during development

#### Development Server Configuration

- Configure efficient development server settings
- Set up proper proxy configurations for API calls
- Implement efficient static asset serving
- Configure HTTPS for local development when needed

## Container and Virtualization

### Docker Integration

- Configure proper volume mounts for development
- Optimize container startup times

### Development Container Configuration

- Consider implementing dev containers for consistent environments
- Configure proper tool installations in containers
- Set up efficient bind mounts and caching
- Test cross-platform compatibility

## Environment Variables and Configuration

### Configuration Management

- Implement proper environment variable management
- Set up development vs production configuration
- Configure secure handling of sensitive data
- Implement configuration validation

### Tool Configuration

- Standardize tool configurations across the team
- Version control all necessary configuration files
- Document required environment setup
- Provide setup automation scripts

## Platform-Specific Considerations

### Linux (Primary Platform)

- Native Fedora development environment
- Ensure all tooling works with current Fedora packages
- Monitor performance characteristics on native Linux

### macOS Support

- Test and document macOS development setup
- Ensure tool compatibility across platforms
- Provide platform-specific installation instructions

## Future Enhancements

### Development Environment Automation

- Create automated environment setup scripts
- Implement environment consistency checks
- Automate tool installation and configuration
- Provide environment troubleshooting guides

### Performance Monitoring

- Implement development metrics collection
- Monitor build performance over time
- Track development server response times
- Analyze development workflow bottlenecks
