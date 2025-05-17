# Tiny Racing Vue Client

This is a Vue.js implementation of the Tiny Racing WebUI client. It connects to a WebSocket server to display racing information in real-time.

## Project Structure

The project is organized into the following components:

- `RaceHeader.vue` - Displays race information and status
- `CarsTable.vue` - Shows a table of all cars and their details
- `TrackVisualizer.vue` - Provides a linear visualization of car positions
- `TrackSvg.vue` - Displays cars on an SVG track
- `ConnectionStatus.vue` - Shows WebSocket connection status

The data is managed through a WebSocket service (`WebSocketService.ts`), which connects to the race server.

## Setup

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build
```

## Configuration

The WebSocket connection uses `ws://127.0.0.1:3030/ws` by default. If you need to change this, modify the `WebSocketService.ts` file.

## Prerequisites

- The race server should be running on port 3030 (WebSocket)
- The assets server should be running on port 8000 (HTTP)

## Recommended IDE Setup

[VSCode](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) (and disable Vetur).

## Type Support for `.vue` Imports in TS

TypeScript cannot handle type information for `.vue` imports by default, so we replace the `tsc` CLI with `vue-tsc` for type checking. In editors, we need [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) to make the TypeScript language service aware of `.vue` types.

## Customize configuration

See [Vite Configuration Reference](https://vite.dev/config/).

## Project Setup

```sh
npm install
```

### Compile and Hot-Reload for Development

```sh
npm run dev
```

### Type-Check, Compile and Minify for Production

```sh
npm run build
```

### Run Unit Tests with [Vitest](https://vitest.dev/)

```sh
npm run test:unit
```

### Run End-to-End Tests with [Playwright](https://playwright.dev)

```sh
# Install browsers for the first run
npx playwright install

# When testing on CI, must build the project first
npm run build

# Runs the end-to-end tests
npm run test:e2e
# Runs the tests only on Chromium
npm run test:e2e -- --project=chromium
# Runs the tests of a specific file
npm run test:e2e -- tests/example.spec.ts
# Runs the tests in debug mode
npm run test:e2e -- --debug
```

### Lint with [ESLint](https://eslint.org/)

```sh
npm run lint
```
