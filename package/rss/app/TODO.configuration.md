# Configuration Management

**Priority**:
 Low

## 1. Objectives

Externalize hardcoded values to make the application more flexible and configurable without code changes.
 Current defaults are reasonable,
 so this is optional.

## 2. Current Hardcoded Values

<table>
<thead>
<tr>
<th>Value</th>
<th>Current</th>
<th>Location</th>
<th>Purpose</th>
</tr>
</thead>
<tbody>
<tr>
<td>`FETCH_INTERVAL_MS`</td>
<td>300000 ms / 5 minutes</td>
<td>`interval.ts`</td>
<td>Fetch-cache time bucket</td>
</tr>
<tr>
<td>Display limit</td>
<td>100 items</td>
<td>`html.ts`</td>
<td>Maximum items shown in UI</td>
</tr>
<tr>
<td>Poll interval</td>
<td>1000ms</td>
<td>`client.ts`</td>
<td>Client-side asset check frequency</td>
</tr>
<tr>
<td>Fetch timeout</td>
<td>30 seconds</td>
<td>Various</td>
<td>HTTP request timeout</td>
</tr>
<tr>
<td>File paths</td>
<td>Hardcoded</td>
<td>Various</td>
<td>JSONL, OPML locations</td>
</tr>
</tbody>
</table>

## 3. Tasks

### 3.1 Create Configuration Module

- [ ] 3.1.1 Create `src/config.ts` with all configurable values
- [ ] 3.1.2 Support environment variables with defaults
- [ ] 3.1.3 Add type-safe configuration schema
- [ ] 3.1.4 Validate configuration on startup

### 3.2 Configuration Schema

```typescript
// src/config.ts
import { z, } from 'zod';

const ConfigSchema = z.object({
  // Server configuration
  server: z.object({
    port: z.number().default(3000,),
    host: z.string().default('localhost',),
  },),

  // Feed configuration
  feed: z.object({
    fetchIntervalMs: z.number().default(300000,), // milliseconds
    fetchTimeout: z.number().default(30000,), // milliseconds
    maxRetries: z.number().default(0,), // no retries by design
    concurrencyLimit: z.number().default(1,), // sequential by default
  },),

  // Display configuration
  display: z.object({
    maxItems: z.number().default(100,),
    pollInterval: z.number().default(1000,), // milliseconds
  },),

  // Storage configuration
  storage: z.object({
    ignorePath: z.string().default('./ignore.jsonl',),
    opmlPath: z.string().optional(), // from env
  },),

  // Cache configuration (if using Caddy proxy)
  cache: z.object({
    enabled: z.boolean().default(false,),
    proxyUrl: z.string().optional(),
  },),

  // Environment
  environment: z
    .enum(['development', 'production', 'test',],)
    .default('production',),
},);

export const config = ConfigSchema.parse({
  server: {
    port: process.env.PORT ? parseInt(process.env.PORT,) : 3000,
    host: process.env.HOST || 'localhost',
  },
  feed: {
    fetchIntervalMs: process.env.RSS_FETCH_INTERVAL_MS
      ? parseInt(process.env.RSS_FETCH_INTERVAL_MS,)
      : 300000,
    fetchTimeout: process.env.FETCH_TIMEOUT
      ? parseInt(process.env.FETCH_TIMEOUT,)
      : 30000,
    maxRetries: 0, // Intentionally no retries
    concurrencyLimit: process.env.CONCURRENCY_LIMIT
      ? parseInt(process.env.CONCURRENCY_LIMIT,)
      : 1,
  },
  display: {
    maxItems: process.env.MAX_ITEMS
      ? parseInt(process.env.MAX_ITEMS,)
      : 100,
    pollInterval: process.env.POLL_INTERVAL
      ? parseInt(process.env.POLL_INTERVAL,)
      : 1000,
  },
  storage: {
    ignorePath: process.env.IGNORE_PATH || './ignore.jsonl',
    opmlPath: process.env.OPMLS,
  },
  cache: {
    enabled: process.env.CACHE_ENABLED === 'true',
    proxyUrl: process.env.CACHE_PROXY_URL,
  },
  environment: process.env.NODE_ENV as any || 'production',
},);
```

### 3.3 Environment Variables

- [ ] 3.3.1 Document all environment variables
- [ ] 3.3.2 Create `.env.example` file
- [ ] 3.3.3 Add validation for required variables
- [ ] 3.3.4 Support multiple environment files

### 3.4 Usage Migration

Replace hardcoded values throughout the codebase:

```typescript
// Before
const FETCH_INTERVAL_MS = 300000;

// After
import { config, } from './config';
const FETCH_INTERVAL_MS = config.feed.fetchIntervalMs;
```

Files to update:

- [ ] 3.4.1 `interval.ts`:
       Fetch-cache time bucket
- [ ] 3.4.2 `html.ts`:
       Display limit
- [ ] 3.4.3 `client.ts`:
       Poll interval
- [ ] 3.4.4 `feed.ts`:
       Fetch timeout
- [ ] 3.4.5 `ignore.ts`:
       File paths
- [ ] 3.4.6 `opmls.ts`:
       OPML path

### 3.5 Configuration Documentation

- [ ] 3.5.1 Document all configuration options
- [ ] 3.5.2 Provide example configurations
- [ ] 3.5.3 Explain defaults and rationale
- [ ] 3.5.4 Add configuration troubleshooting

## 4. Benefits

- 4.1 **Flexibility**:
   Change behavior without code changes
- 4.2 **Environment-specific**:
   Different configs for dev/prod
- 4.3 **Type Safety**:
   Zod validation ensures correctness
- 4.4 **Documentation**:
   Clear list of all configurable options
- 4.5 **Testing**:
   Easy to override for tests

## 5. Considerations

- 5.1 Current defaults work well for most use cases
- 5.2 Adding configuration adds complexity
- 5.3 Need to balance flexibility with simplicity
- 5.4 Should maintain backwards compatibility

## 6. Success Criteria

- 6.1 All hardcoded values externalized
- 6.2 Configuration validated on startup
- 6.3 Environment variables documented
- 6.4 No breaking changes for existing deployments
- 6.5 Easy to override for testing
