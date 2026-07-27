import * as React from 'react';
import renderer from 'react-test-renderer';

import { ThemedText } from '../ThemedText';
import { ThemeProvider } from '../../context/ThemeContext';

/**
 * ThemedText reads its colours through `useThemeColor`, which calls
 * `useThemeContext`. That hook throws when no provider is mounted, so
 * rendering the component bare fails outright — this test has been red since
 * ThemeContext was introduced. Wrapping it in the provider matches how the
 * component is actually used at runtime.
 */
it(`renders correctly`, () => {
  const tree = renderer
    .create(
      <ThemeProvider>
        <ThemedText>Snapshot test!</ThemedText>
      </ThemeProvider>,
    )
    .toJSON();

  expect(tree).toMatchSnapshot();
});
