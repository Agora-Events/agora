import { expect, afterEach } from "vitest";
import "@testing-library/jest-dom";

// This file is used to set up Vitest tests
// It imports the Jest DOM matchers so they're available in all test files

// Make sure jest-dom matchers are available globally
// This is the recommended way to use jest-dom with Vitest
// See: https://testing-library.com/docs/jest-dom/install#vitest

// Configure expect to use jest-dom matchers
expect.extend({
  toBeInTheDocument: (received) => {
    const pass = received && received.ownerDocument && received.ownerDocument.body.contains(received);
    if (pass) {
      return {
        message: () => `expected ${received} not to be in the document`,
        pass: true,
      };
    } else {
      return {
        message: () => `expected ${received} to be in the document`,
        pass: false,
      };
    }
  },
});
