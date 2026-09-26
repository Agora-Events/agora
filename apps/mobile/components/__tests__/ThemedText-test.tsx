import * as React from 'react';
import renderer, { act } from 'react-test-renderer';

import { ThemedText } from '../ThemedText';
import { ThemeProvider } from '../../context/ThemeContext';

/**
 * ThemedText reads its colours through `useThemeColor`, which calls
 * `useThemeContext`. That hook throws without a provider, so rendering the
 * component bare fails — this test has been red since ThemeContext landed.
 *
 * `act` is required as well as the provider: ThemeProvider subscribes to
 * `Appearance` in an effect, and a bare `renderer.create` leaves that effect
 * to run after Jest has torn the environment down ("trying to `import` a file
 * after the Jest environment has been torn down"). Flushing inside `act` and
 * unmounting keeps the subscription's whole lifecycle within the test.
 */
it(`renders correctly`, () => {
  let tree: renderer.ReactTestRenderer;

  act(() => {
    tree = renderer.create(
      <ThemeProvider>
        <ThemedText>Snapshot test!</ThemedText>
      </ThemeProvider>,
    );
  });

  expect(tree!.toJSON()).toMatchSnapshot();

  act(() => {
    tree.unmount();
  });
});
