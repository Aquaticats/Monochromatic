# Testing Infrastructure

**Priority**:
 Critical

## 1. Objectives

Establish comprehensive test coverage for the RSS reader application to ensure reliability,
 catch regressions,
 and enable confident refactoring.

## 2. Tasks

### 2.1 Test Framework Setup

- [ ] 2.1.1 Set up testing framework
- [ ] 2.1.2 Configure test coverage reporting
- [ ] 2.1.3 Set up test watch mode for development
- [ ] 2.1.4 Configure test environments (node,
       jsdom)

### 2.2 Unit Tests

Create unit tests for each module with focus on:

#### 2.2.1 Feed Module (`feed.test.ts`)

- [ ] 2.2.1.1 Test feed fetching with mock responses
- [ ] 2.2.1.2 Test RSS parsing logic
- [ ] 2.2.1.3 Test Atom parsing logic
- [ ] 2.2.1.4 Test error handling for malformed feeds
- [ ] 2.2.1.5 Test feed transformation to internal format
- [ ] 2.2.1.6 Test observable subscription behavior

#### 2.2.2 Item Module (`item.test.ts`)

- [ ] 2.2.2.1 Test item extraction from RSS feeds
- [ ] 2.2.2.2 Test item extraction from Atom feeds
- [ ] 2.2.2.3 Test date parsing and normalization
- [ ] 2.2.2.4 Test item sorting by date
- [ ] 2.2.2.5 Test handling of missing fields
- [ ] 2.2.2.6 Test content sanitization

#### 2.2.3 Outline Module (`outline.test.ts`)

- [ ] 2.2.3.1 Test OPML parsing
- [ ] 2.2.3.2 Test nested outline handling
- [ ] 2.2.3.3 Test file:
      // protocol support
- [ ] 2.2.3.4 Test http:
      // protocol support
- [ ] 2.2.3.5 Test invalid OPML handling
- [ ] 2.2.3.6 Test outline flattening logic

#### 2.2.4 Ignore Module (`ignore.test.ts`)

- [ ] 2.2.4.1 Test JSONL file reading
- [ ] 2.2.4.2 Test JSONL file writing
- [ ] 2.2.4.3 Test item filtering logic
- [ ] 2.2.4.4 Test duplicate prevention
- [ ] 2.2.4.5 Test file creation if not exists
- [ ] 2.2.4.6 Test concurrent write handling

#### 2.2.5 Asset Module (`asset.test.ts`)

- [ ] 2.2.5.1 Test SHA-256 hash generation
- [ ] 2.2.5.2 Test file watching mechanism
- [ ] 2.2.5.3 Test HTML parsing with happy-dom
- [ ] 2.2.5.4 Test CSS/JS injection
- [ ] 2.2.5.5 Test cache busting logic
- [ ] 2.2.5.6 Test asset change detection

### 2.3 Integration Tests

- [ ] 2.3.1 Test endpoint `GET /`,
       renders the RSS reader interface
- [ ] 2.3.2 Test endpoint `POST /api/ignore/new`,
       appends an ignored item to JSONL storage
- [ ] 2.3.3 Remove or explicitly plan the old `/feed/update`,
       `/feed/ignore`,
       `/api/ignore`,
       `/api/ignore/clear`,
       and `/api/assetHash` endpoints before writing tests for them
- [ ] 2.3.6 Test rate limiting behavior
- [ ] 2.3.7 Test CORS headers

### 2.4 End-to-End Tests

- [ ] 2.4.1 Test complete feed aggregation flow
- [ ] 2.4.2 Test OPML file changes triggering updates
- [ ] 2.4.3 Test client-side auto-reload on asset changes
- [ ] 2.4.4 Test scroll-based auto-ignore functionality
- [ ] 2.4.5 Test iframe sandboxing
- [ ] 2.4.6 Test 100-item display limit

### 2.5 Test Coverage Goals

- [ ] 2.5.1 Achieve minimum 80% code coverage
- [ ] 2.5.2 Set up coverage thresholds in CI
- [ ] 2.5.3 Generate coverage reports
- [ ] 2.5.4 Identify and test edge cases

## 3. Testing Strategy

### 3.1 Test Data

- 3.1.1 Create fixture files for:
  - 3.1.1.1 Sample RSS feeds
  - 3.1.1.2 Sample Atom feeds
  - 3.1.1.3 Sample OPML files
  - 3.1.1.4 Malformed/edge case feeds

### 3.2 Mocking Strategy

- 3.2.1 Mock external HTTP requests
- 3.2.2 Mock file system operations where appropriate
- 3.2.3 Mock date/time for deterministic tests
- 3.2.4 Mock Observable subscriptions

### 3.3 Continuous Integration

- 3.3.1 Run tests on every commit
- 3.3.2 Block merge if tests fail
- 3.3.3 Block merge if coverage drops below threshold
- 3.3.4 Generate and publish coverage reports

## 4. Success Criteria

- 4.1 All modules have comprehensive test suites
- 4.2 Tests are maintainable and well-documented
- 4.3 Tests run quickly (< 30 seconds for unit tests)
- 4.4 Coverage meets or exceeds 80% threshold
- 4.5 Tests catch real bugs before production
