# Testing Strategy & How to Run Tests

## Overview

Agora's web application uses **Vitest** as its testing framework with **jsdom** environment for component testing. This document outlines our testing strategy, setup, and best practices.

## Testing Stack

| Tool                          | Purpose                           | Version             |
| ----------------------------- | --------------------------------- | ------------------- |
| **Vitest**                    | Test runner & assertion library   | ^3.2.4              |
| **@testing-library/react**    | React component testing utilities | ^16.3.2             |
| **@testing-library/dom**      | DOM queries and utilities         | ^10.4.0             |
| **@testing-library/jest-dom** | Extended DOM matchers             | ^6.9.1              |
| **jsdom**                     | Browser-like environment          | Built-in via Vitest |

## Quick Start

### Running Tests

```bash
# Run tests once (single pass)
pnpm test

# Run tests in CI mode with coverage
pnpm test:ci

# Run tests in watch mode (for development)
pnpm test -- --watch
```

### Test Coverage Reports

After running tests, coverage reports are generated in:

- **HTML Report**: `coverage/index.html` (open in browser)
- **JSON Report**: `coverage/coverage-final.json`
- **Terminal Output**: Coverage summary in console

**Coverage Thresholds (CI):** 80% minimum

## Project Structure

```
apps/web/
├── __tests__/
│   ├── button.test.tsx           # UI button component tests
│   ├── event-card.test.tsx       # Event card display tests
│   ├── event-location-map.test.tsx # Map component integration tests
│   ├── gift-tickets.test.ts      # Gift ticket feature tests
│   └── navbar.test.tsx           # Navigation bar tests
├── vitest.config.ts              # Vitest configuration
├── vitest.setup.ts               # Test environment setup
└── components/                   # Component implementations
```

## Test Categories

### 1. Component Tests

Test React components in isolation, focusing on:

- Rendering logic
- User interactions (clicks, form inputs)
- Props validation
- State changes

**Example: `button.test.tsx`**

```typescript
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Button } from '@/components/ui/button';

describe('Button', () => {
  it('renders with text', () => {
    render(<Button>Click me</Button>);
    expect(screen.getByText('Click me')).toBeInTheDocument();
  });

  it('calls onClick handler on click', async () => {
    const handleClick = vi.fn();
    render(<Button onClick={handleClick}>Click</Button>);

    await userEvent.click(screen.getByRole('button'));
    expect(handleClick).toHaveBeenCalled();
  });
});
```

### 2. Integration Tests

Test how multiple components work together, focusing on:

- Data flow between components
- Event handling chains
- Modal interactions

**Example: `event-card.test.tsx`**

- Event card rendering with image, title, date
- Click handling and navigation
- State synchronization

### 3. API/Data Tests

Test utility functions and hooks:

- Data fetching (SWR hooks)
- Blockchain interactions (Stellar SDK)
- Data transformations

**Example: `gift-tickets.test.ts`**

```typescript
describe("Gift Tickets", () => {
  it("validates recipient wallet", () => {
    expect(() => validateRecipient(invalidWallet)).toThrow();
    expect(() => validateRecipient(validWallet)).not.toThrow();
  });

  it("calculates correct ticket quantities", () => {
    const qty = calculateTicketQty(eventPrice, userBalance);
    expect(qty).toBe(expectedQuantity);
  });
});
```

## Configuration Files

### `vitest.config.ts`

```typescript
// Environment: jsdom (browser-like)
// Globals: enabled (describe, it, expect available without imports)
// Setup: vitest.setup.ts runs before each test
// Coverage:
//   - Provider: v8 (native JavaScript engine coverage)
//   - Reporters: text, json, html
//   - Include: components/** only
//   - Exclude: node_modules, tests, types, configs
```

### `vitest.setup.ts`

```typescript
// Configures @testing-library/jest-dom matchers
// Provides: toBeInTheDocument(), toBeVisible(), etc.
// Cleans up DOM after each test
// Sets up jsdom environment
```

## Writing Tests

### Best Practices

1. **Use descriptive test names**

   ```typescript
   // ❌ Bad
   it('works', () => { ... });

   // ✅ Good
   it('displays ticket availability count when loading event', () => { ... });
   ```

2. **Test user behavior, not implementation**

   ```typescript
   // ❌ Bad - tests internal state
   expect(component.state.isOpen).toBe(true);

   // ✅ Good - tests user-visible behavior
   expect(screen.getByRole("dialog")).toBeInTheDocument();
   ```

3. **Use semantic queries**

   ```typescript
   // ❌ Bad - fragile to style changes
   expect(container.querySelector(".btn-primary")).toBeInTheDocument();

   // ✅ Good - queries by role/label
   expect(
     screen.getByRole("button", { name: /buy ticket/i }),
   ).toBeInTheDocument();
   ```

4. **Isolate tests from each other**

   ```typescript
   // ✅ Each test is independent
   it("test 1", () => {
     /* ... */
   });
   it("test 2", () => {
     /* ... */
   }); // not affected by test 1
   ```

5. **Mock external dependencies**

   ```typescript
   import { vi } from "vitest";

   // Mock API calls
   global.fetch = vi.fn(() => Promise.resolve({ json: () => ({}) }));

   // Mock modules
   vi.mock("@/lib/stellar", () => ({
     mintTicket: vi.fn(() => Promise.resolve({ ticketId: "test" })),
   }));
   ```

### Common Testing Patterns

#### Testing Click Handlers

```typescript
import userEvent from '@testing-library/user-event';

it('opens ticket modal on register click', async () => {
  const user = userEvent.setup();
  render(<RegistrationBox event={mockEvent} />);

  await user.click(screen.getByRole('button', { name: /register/i }));
  expect(screen.getByRole('dialog')).toBeInTheDocument();
});
```

#### Testing Form Inputs

```typescript
it('updates quantity on input change', async () => {
  const user = userEvent.setup();
  render(<QuantitySelector />);

  const input = screen.getByRole('spinbutton');
  await user.clear(input);
  await user.type(input, '5');

  expect(input).toHaveValue(5);
});
```

#### Testing Async Operations

```typescript
it('fetches event details', async () => {
  render(<EventDetail eventId="123" />);

  await waitFor(() => {
    expect(screen.getByText('Event Title')).toBeInTheDocument();
  });
});
```

#### Testing Conditions

```typescript
it('shows ticket availability badge when tickets available', () => {
  const { rerender } = render(<EventCard event={eventWithTickets} />);
  expect(screen.getByText(/tickets available/i)).toBeInTheDocument();

  rerender(<EventCard event={soldOutEvent} />);
  expect(screen.queryByText(/tickets available/i)).not.toBeInTheDocument();
});
```

## Running Tests in CI

The CI pipeline runs:

```bash
pnpm test:ci
```

This:

1. Runs all tests once
2. Generates coverage report
3. Enforces 80% coverage threshold
4. Fails if threshold not met

**Coverage includes:** All component files in `components/**/*.{ts,tsx}`

## Debugging Tests

### Option 1: Console Logging

```typescript
import { screen, debug } from '@testing-library/react';

it('debug test', () => {
  const { container } = render(<MyComponent />);

  // Print entire DOM
  debug();

  // Print specific element
  debug(screen.getByRole('button'));
});
```

### Option 2: Run Single Test

```bash
pnpm test -- button.test.tsx
```

### Option 3: Run Tests in Watch Mode

```bash
pnpm test -- --watch
```

Press `p` to filter by file, `t` to filter by test name.

### Option 4: VS Code Debugger

Add to `.vscode/launch.json`:

```json
{
  "type": "node",
  "request": "launch",
  "name": "Vitest",
  "runtimeExecutable": "pnpm",
  "runtimeArgs": ["test", "--inspect-brk", "--run"],
  "console": "integratedTerminal"
}
```

## Test File Checklist

When creating a new test file, include:

- [ ] Descriptive file name: `component-name.test.tsx`
- [ ] Proper imports: React, component, testing-library, utils
- [ ] `describe` block for grouping related tests
- [ ] Mocks for external dependencies
- [ ] Setup/teardown if needed
- [ ] Multiple test cases (happy path, error cases)
- [ ] Accessibility queries (getByRole, getByLabelText)
- [ ] Async handling with `waitFor` or `userEvent`

## Metrics & Goals

- **Coverage Target**: 80% (enforced in CI)
- **Test Execution Time**: < 30s for full suite
- **Test-to-Code Ratio**: Aim for 0.5-1.0 (at least 50% test code)

## Resources

- [Vitest Documentation](https://vitest.dev)
- [Testing Library Documentation](https://testing-library.com)
- [Best Practices Guide](https://kentcdodds.com/blog/common-mistakes-with-react-testing-library)
- [Accessibility Testing](https://www.w3.org/WAI/test-evaluate/)

## Troubleshooting

### Issue: Tests pass locally but fail in CI

**Solution:** Ensure you're using consistent Node.js version. Check CI logs for exact error.

### Issue: jsdom environment issues

**Solution:** Some DOM APIs aren't available in jsdom. Use `vi.mock()` to mock them.

### Issue: Timeout errors in async tests

**Solution:** Increase timeout in test or use `waitFor` with explicit checks:

```typescript
await waitFor(() => expect(element).toBeInTheDocument(), { timeout: 5000 });
```

### Issue: Mock not being applied

**Solution:** Mocks must be at top level of test file, not inside test:

```typescript
// ✅ Correct
vi.mock('@/lib/stellar');
it('test', () => { ... });

// ❌ Wrong
it('test', () => {
  vi.mock('@/lib/stellar'); // Too late!
});
```
