import React from 'react';
import { render, fireEvent } from '@testing-library/react-native';
import { SettingsScreen } from '../SettingsScreen';

jest.mock('../../hooks/useAuth', () => ({
  useAuth: () => ({
    user: {
      id: '1',
      name: 'Test User',
      email: 'test@example.com',
      walletAddress: 'GA7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVSGZ',
    },
    logout: jest.fn(),
  }),
}));

describe('SettingsScreen', () => {
  it('renders settings and invokes logout when destructive button is pressed', () => {
    const handleLogout = jest.fn();
    const { getByTestId, getByText } = render(
      <SettingsScreen onLogout={handleLogout} />
    );

    expect(getByText('Settings')).toBeTruthy();
    expect(getByText('Test User')).toBeTruthy();

    const logoutButton = getByTestId('logout-button');
    expect(logoutButton).toBeTruthy();

    fireEvent.press(logoutButton);
    expect(handleLogout).toHaveBeenCalledTimes(1);
  });
});
