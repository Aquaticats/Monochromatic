# Not Planned Improvements

This document outlines improvements that were considered but are explicitly **NOT PLANNED** for implementation,
 along with the rationale for each decision.

## 1. Error Handling & Retry Logic

**Decision**:
 No automatic retry logic by design

**Rationale**:

- 1.1 Users are expected to monitor logs and remove failing feeds manually
- 1.2 Automatic retries can mask persistent issues
- 1.3 Manual curation of feeds ensures quality
- 1.4 Simplifies the application logic
- 1.5 Aligns with the philosophy of user control

## 2. User Experience Enhancements

**Decision**:
 Not planned due to time constraints

**Potential features not being implemented**:

- 2.1 Search functionality across items
- 2.2 Advanced filtering options
- 2.3 Categories/tags for feeds
- 2.4 Read/unread status tracking
- 2.5 Keyboard shortcuts
- 2.6 Infinite scrolling
- 2.7 RSS feed discovery
- 2.8 OPML export
- 2.9 Dark mode toggle

**Rationale**:

- 2.10 Current UI is functional and sufficient
- 2.11 Limited development time available
- 2.12 Users can fork for additional features
- 2.13 Focus on core functionality over nice-to-haves

## 3. Security Enhancements

**Decision**:
 Security handled at infrastructure level

**Already handled by Caddy**:

- 3.1 Content Security Policy (CSP) headers
- 3.2 Rate limiting
- 3.3 Authentication/authorization if needed
- 3.4 HTTPS termination
- 3.5 CORS configuration

**Already handled by application**:

- 3.6 Iframe sandboxing for feed content prevents:
  - 3.6.1 JavaScript execution from feed content
  - 3.6.2 Form submission
  - 3.6.3 Plugin execution
  - 3.6.4 Top navigation
  - 3.6.5 Same-origin access

**Rationale**:

- 3.7 Iframe sandboxing is considered sufficient for content isolation
- 3.8 Additional HTML sanitization would be redundant
- 3.9 Infrastructure-level security is more robust
- 3.10 Separation of concerns between app and infrastructure

## 4. Monitoring & Observability

**Decision**:
 Not relevant for current use case

**Features not being implemented**:

- 4.1 Application Performance Monitoring (APM)
- 4.2 Distributed tracing
- 4.3 Custom metrics collection
- 4.4 Performance dashboards
- 4.5 Error tracking integration (e.g.,
   Sentry)
- 4.6 Log aggregation

**Rationale**:

- 4.7 Small-scale personal project
- 4.8 Logs are sufficient for debugging
- 4.9 Additional complexity not justified
- 4.10 Can monitor via system tools if needed

## 5. Additional Features

**Decision**:
 Users can fork for additional features

**Examples of features not in scope**:

- 5.1 Webhook support for real-time updates
- 5.2 Feed autodiscovery from URLs
- 5.3 JSON feed support
- 5.4 Feed validation and health scoring
- 5.5 Feed statistics and analytics
- 5.6 Support for authenticated/private feeds
- 5.7 Feed transformation rules
- 5.8 Feed aggregation and deduplication
- 5.9 Multi-user support
- 5.10 Mobile app
- 5.11 Browser extension
- 5.12 Email notifications
- 5.13 Social media integration

**Rationale**:

- 5.14 Keep the core application simple
- 5.15 Open source allows customization
- 5.16 Avoid feature creep
- 5.17 Focus on doing one thing well

## 6. Design Philosophy

These decisions align with the project's core philosophy:

1. 6.1 **Simplicity over features**:
    A simple,
    working solution is better than a complex,
    feature-rich one
2. 6.2 **Manual control**:
    Users should have explicit control over their feeds
3. 6.3 **Infrastructure-first**:
    Leverage infrastructure (Caddy) for cross-cutting concerns
4. 6.4 **No magic**:
    Transparent,
    predictable behavior without hidden automation
5. 6.5 **Fork-friendly**:
    Keep codebase simple enough for others to customize

## 7. Reconsidering Decisions

While these items are not planned,
 they could be reconsidered if:

- 7.1 The project scope significantly expands
- 7.2 Multiple users request specific features
- 7.3 Security vulnerabilities are discovered that require app-level fixes
- 7.4 Performance issues arise that require monitoring
- 7.5 The project transitions from personal to community-driven

## 8. Contributing

If you need any of these features:

1. 8.1 Fork the repository
2. 8.2 Implement the features you need
3. 8.3 Consider submitting a PR if the feature might benefit others
4. 8.4 Or maintain your own fork with your specific requirements
