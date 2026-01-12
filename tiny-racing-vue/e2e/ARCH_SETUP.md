# Arch Linux Setup for Playwright

On Arch Linux, it's recommended to use system browsers instead of Playwright's bundled browsers for better compatibility and performance.

## Setup Steps

### 1. Install System Browsers

```bash
sudo pacman -S chromium firefox
```

### 2. Skip Browser Download (Optional but Recommended)

Add this to your `~/.bashrc` or `~/.zshrc`:

```bash
export PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1
```

Then reload your shell:
```bash
source ~/.bashrc  # or source ~/.zshrc
```

### 3. Use System Browsers in Tests

Set the environment variable before running tests:

```bash
PLAYWRIGHT_USE_SYSTEM_BROWSERS=1 npm run test:e2e
```

Or add it to your shell config:

```bash
export PLAYWRIGHT_USE_SYSTEM_BROWSERS=1
```

## Benefits

- ✅ Native Arch builds
- ✅ Zero compatibility hacks
- ✅ Much faster security updates
- ✅ No broken ICU/libffi dependencies
- ✅ Smaller disk footprint

## Notes

- WebKit is not available as a system package on Arch, so webkit tests will be skipped when using system browsers
- If you don't set `PLAYWRIGHT_USE_SYSTEM_BROWSERS=1`, Playwright will use its bundled browsers (which may have dependency issues on Arch)

