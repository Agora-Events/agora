import React from 'react';
import { render, fireEvent, waitFor, act } from '@testing-library/react-native';
import * as Clipboard from 'expo-clipboard';
import ProfileHeader, { truncatePublicKey } from '../ProfileHeader';

jest.mock('expo-image-picker', () => ({
  requestMediaLibraryPermissionsAsync: jest.fn().mockResolvedValue({ status: 'granted' }),
  launchImageLibraryAsync: jest.fn().mockResolvedValue({ canceled: true }),
  MediaTypeOptions: { Images: 'Images' },
}));

jest.mock('expo-clipboard', () => ({
  setStringAsync: jest.fn().mockResolvedValue(true),
}));

describe('truncatePublicKey', () => {
  it('truncates public key correctly with default 4 start and 4 end chars', () => {
    expect(truncatePublicKey('GD47ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGH7STN')).toBe('GD47...7STN');
  });

  it('handles null, undefined or empty string', () => {
    expect(truncatePublicKey('')).toBe('');
    expect(truncatePublicKey(null)).toBe('');
    expect(truncatePublicKey(undefined)).toBe('');
  });

  it('returns original string if length is less than or equal to start + end chars', () => {
    expect(truncatePublicKey('GD477STN')).toBe('GD477STN');
    expect(truncatePublicKey('SHORT')).toBe('SHORT');
  });
});

describe('ProfileHeader', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  it('renders username and email correctly', () => {
    const { getByTestId, getByText } = render(
      <ProfileHeader
        username="Jane Doe"
        email="jane@example.com"
        publicKey="GD47ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGH7STN"
      />
    );

    expect(getByTestId('profile-username')).toHaveTextContent('Jane Doe');
    expect(getByTestId('profile-email')).toHaveTextContent('jane@example.com');
    expect(getByText('GD47...7STN')).toBeTruthy();
  });

  it('renders default fallback username and email when not provided', () => {
    const { getByTestId } = render(<ProfileHeader />);

    expect(getByTestId('profile-username')).toHaveTextContent('Agora User');
    expect(getByTestId('profile-email')).toHaveTextContent('user@agora.events');
    expect(getByTestId('truncated-public-key')).toHaveTextContent('No address');
  });

  it('renders avatar placeholder with initial letter of user name', () => {
    const { getByText } = render(<ProfileHeader username="Alice" />);
    expect(getByText('A')).toBeTruthy();
  });

  it('copies the full public key to clipboard and shows toast message', async () => {
    const fullKey = 'GD47ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGH7STN';
    const onCopy = jest.fn();

    const { getByTestId, queryByTestId } = render(
      <ProfileHeader
        username="Jane Doe"
        email="jane@example.com"
        publicKey={fullKey}
        onCopy={onCopy}
        toastDuration={2000}
      />
    );

    expect(queryByTestId('toast-message')).toBeNull();

    const copyBtn = getByTestId('copy-public-key-button');
    fireEvent.press(copyBtn);

    await waitFor(() => {
      expect(Clipboard.setStringAsync).toHaveBeenCalledWith(fullKey);
      expect(onCopy).toHaveBeenCalledWith(fullKey);
    });

    expect(getByTestId('toast-message')).toBeTruthy();
    expect(getByTestId('toast-message')).toHaveTextContent('Public key copied to clipboard!');

    // Fast-forward toast duration
    act(() => {
      jest.advanceTimersByTime(2500);
    });

    await waitFor(() => {
      expect(queryByTestId('toast-message')).toBeNull();
    });
  });

  it('does not trigger clipboard copy when publicKey is absent', async () => {
    const { getByTestId } = render(<ProfileHeader />);
    const copyBtn = getByTestId('copy-public-key-button');

    fireEvent.press(copyBtn);

    expect(Clipboard.setStringAsync).not.toHaveBeenCalled();
  });
});
