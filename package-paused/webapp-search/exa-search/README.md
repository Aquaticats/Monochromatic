# Exa Search Interface

A self-contained, single-file HTML search interface for the Exa AI search API,
styled after DuckDuckGo's minimalist design philosophy.

## Features

- **Single-file output**: Entire application bundled into one HTML file
- **DuckDuckGo-inspired design**: Clean, minimal interface focused on search
- **Dark/light mode support**: Automatically adapts to system preferences via `prefers-color-scheme`
- **API key management**: Store API key in localStorage with easy management
- **Full results display**: Shows all results (up to Exa's maximum of 100) without pagination
- **Responsive design**: Works well on desktop and mobile devices

## Building

From the workspace root:

```bash
mise run //packages/webapp-search/exa-search:build:js:client
```

This builds the browser bundle at `dist/client/client.js`. The h3 server composes
the HTML in memory from that bundle.

## Usage

1. Start the server with `mise run //packages/webapp-search/exa-search:dev`
2. Click "Set API Key" and enter your Exa API key (get one at [exa.ai](https://exa.ai))
3. Enter your search query and click "Search"
4. All results (up to 100) will be displayed below the search box

The API key is stored in your browser's localStorage for convenience.
You can change it anytime by clicking the "Change API Key" button.

## Technical Details

- **Build system**: tsdown
- **Language**: TypeScript
- **Styling**: Pure CSS with CSS custom properties for theming
- **Dependencies**: Server and rendering use h3, h-html, Valibot, and workspace DOM/logger utilities
- **API integration**: Direct fetch calls to Exa's search API

## Development

The source files are organized as:

- `src/index.ts`: h3 server entry point
- `src/asset*.ts`: HTML sections and server-side document composition
- `src/client*.ts`: browser behavior and API integration
- `src/index.css`: Styling with dark/light mode support
