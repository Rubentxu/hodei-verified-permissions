# E2E Tests Guide - Hodei Verified Permissions Frontend

## Overview

This guide explains how to run and manage E2E tests for the Hodei Verified Permissions frontend using Playwright.

## Installation

```bash
# Install Playwright
npm install @playwright/test --save-dev

# Install browsers
npm run playwright:install

# Install system dependencies (Linux/macOS)
npm run playwright:install-deps
```

## Running Tests

### All Tests
```bash
npm run test:e2e
```

### UI Mode (Interactive)
```bash
npm run test:e2e:ui
```

### Headed Mode (See Browser)
```bash
npm run test:e2e:headed
```

### Debug Mode
```bash
npm run test:e2e:debug
```

### Specific Browser
```bash
npm run test:e2e:chrome
npm run test:e2e:firefox
npm run test:e2e:webkit
```

### Mobile Testing
```bash
npm run test:e2e:mobile
```

### All Browsers
```bash
npm run test:e2e:all
```

## Test Files

### `tests/e2e/dashboard.spec.ts`
Tests for the main dashboard and navigation:
- Dashboard title display
- Connection status
- Navigation to different sections
- Sidebar toggle functionality

### `tests/e2e/schemas.spec.ts`
Tests for the Schema Editor:
- Schema creation
- Schema validation
- Schema saving
- Schema duplication
- Schema deletion

### `tests/e2e/policies.spec.ts`
Tests for the Policy Editor:
- Policy editor page display
- Policy creation wizard
- Wizard step completion
- Policy validation
- Policy saving

### `tests/e2e/templates.spec.ts`
Tests for the Templates System:
- Template creation
- Template parameter management
- Template search
- Template filtering by category
- Template validation

### `tests/e2e/api.spec.ts`
Tests for API endpoints:
- Health check endpoint
- Authorization requests
- Policy store creation
- Error handling

## Configuration

The Playwright configuration is defined in `playwright.config.ts`:

- **Base URL**: http://localhost:3000
- **Test Directory**: tests/e2e
- **Browsers**: Chromium, Firefox, WebKit
- **Mobile Devices**: Pixel 5, iPhone 12
- **Reporters**: HTML report
- **Screenshots**: On failure
- **Videos**: On failure
- **Traces**: On first retry

## Continuous Integration

For CI/CD pipelines:

```bash
# Run tests in CI mode (single worker, retries enabled)
CI=true npm run test:e2e
```

## Debugging

### View Test Report
```bash
npx playwright show-report
```

### Run Single Test
```bash
npx playwright test tests/e2e/dashboard.spec.ts
```

### Run Tests Matching Pattern
```bash
npx playwright test -g "should display"
```

### Update Snapshots
```bash
npx playwright test --update-snapshots
```

## Best Practices

1. **Wait for Elements**: Use proper wait strategies instead of fixed delays
2. **Use Data Attributes**: Add `data-testid` attributes for reliable element selection
3. **Test User Flows**: Test complete user journeys, not just individual components
4. **Keep Tests Independent**: Each test should be able to run independently
5. **Use Page Objects**: For complex pages, consider using Page Object Model pattern
6. **Mock External APIs**: Mock gRPC calls for faster, more reliable tests

## Troubleshooting

### Tests Timeout
- Increase timeout in `playwright.config.ts`
- Check if dev server is running on port 3000

### Element Not Found
- Use `--headed` mode to see what's happening
- Check selectors in browser DevTools

### Flaky Tests
- Add proper waits for elements
- Avoid fixed delays, use `waitForSelector`
- Check for race conditions

## Adding New Tests

1. Create a new `.spec.ts` file in `tests/e2e/`
2. Import test utilities:
   ```typescript
   import { test, expect } from '@playwright/test';
   ```
3. Write test cases:
   ```typescript
   test('should do something', async ({ page }) => {
     await page.goto('/');
     // Your test code
   });
   ```
4. Run tests to verify

## Performance Monitoring

Tests include performance metrics:
- Page load time
- Element interaction time
- API response time

Check the HTML report for detailed metrics.

## Integration with CI/CD

Add to your CI pipeline:

```yaml
- name: Install Playwright
  run: npm run playwright:install

- name: Run E2E Tests
  run: npm run test:e2e

- name: Upload Test Results
  if: always()
  uses: actions/upload-artifact@v3
  with:
    name: playwright-report
    path: playwright-report/
```

## Resources

- [Playwright Documentation](https://playwright.dev)
- [Playwright Best Practices](https://playwright.dev/docs/best-practices)
- [Debugging Tests](https://playwright.dev/docs/debug)
