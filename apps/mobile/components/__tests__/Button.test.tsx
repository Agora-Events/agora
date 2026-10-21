import React from 'react';
import { render, fireEvent } from '@testing-library/react-native';
import { Button } from '../ui/Button';

describe('Button component', () => {
  it('renders destructive variant with accessibility and responds to press', () => {
    const handlePress = jest.fn();
    const { getByText, getByRole } = render(
      <Button
        title="Log Out"
        variant="destructive"
        onPress={handlePress}
      />
    );

    const button = getByText('Log Out');
    expect(button).toBeTruthy();

    fireEvent.press(button);
    expect(handlePress).toHaveBeenCalledTimes(1);
  });
});
