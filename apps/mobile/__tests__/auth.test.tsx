import React from 'react';
import { render, fireEvent, waitFor } from '@testing-library/react-native';
import AuthScreen from '../app/auth';
import { useAuthStore } from '../hooks/useAuth';

// Mock expo-router
jest.mock('expo-router', () => ({
  useRouter: () => ({
    replace: jest.fn(),
    push: jest.fn(),
  }),
  useSegments: () => ['auth'],
}));

describe('AuthScreen Validation and Login', () => {
  beforeEach(() => {
    // Reset Zustand store state before each test
    useAuthStore.setState({
      token: null,
      user: null,
      isAuthenticated: false,
    });
  });

  it('renders correctly', () => {
    const { getByPlaceholderText, getByText } = render(<AuthScreen />);
    expect(getByPlaceholderText('Enter your email')).toBeTruthy();
    expect(getByPlaceholderText('Enter your password')).toBeTruthy();
    expect(getByText('Log In')).toBeTruthy();
  });

  it('shows error boundaries when fields are empty', async () => {
    const { getByText, findByText } = render(<AuthScreen />);
    const loginButton = getByText('Log In');

    fireEvent.press(loginButton);

    expect(await findByText('Email is required')).toBeTruthy();
    expect(await findByText('Password is required')).toBeTruthy();
  });

  it('shows validation errors for invalid email format', async () => {
    const { getByPlaceholderText, getByText, findByText } = render(<AuthScreen />);
    const emailInput = getByPlaceholderText('Enter your email');
    const passwordInput = getByPlaceholderText('Enter your password');
    const loginButton = getByText('Log In');

    fireEvent.changeText(emailInput, 'invalidemail');
    fireEvent.changeText(passwordInput, 'password123');
    fireEvent.press(loginButton);

    expect(await findByText('Invalid email format')).toBeTruthy();
  });

  it('shows validation errors for short password', async () => {
    const { getByPlaceholderText, getByText, findByText } = render(<AuthScreen />);
    const emailInput = getByPlaceholderText('Enter your email');
    const passwordInput = getByPlaceholderText('Enter your password');
    const loginButton = getByText('Log In');

    fireEvent.changeText(emailInput, 'user@example.com');
    fireEvent.changeText(passwordInput, '12345');
    fireEvent.press(loginButton);

    expect(await findByText('Password must be at least 6 characters')).toBeTruthy();
  });

  it('triggers mock successful login and updates Zustand store on valid input', async () => {
    const { getByPlaceholderText, getByText } = render(<AuthScreen />);
    const emailInput = getByPlaceholderText('Enter your email');
    const passwordInput = getByPlaceholderText('Enter your password');
    const loginButton = getByText('Log In');

    fireEvent.changeText(emailInput, 'user@example.com');
    fireEvent.changeText(passwordInput, 'password123');
    fireEvent.press(loginButton);

    await waitFor(() => {
      const state = useAuthStore.getState();
      expect(state.isAuthenticated).toBe(true);
      expect(state.user?.email).toBe('user@example.com');
      expect(state.token).toBe('mock-jwt-token-agora');
    }, { timeout: 1500 });
  });

  it('handles Google/Apple mock sign-in correctly', async () => {
    const { getByText } = render(<AuthScreen />);
    const googleButton = getByText('Sign In with Google');

    fireEvent.press(googleButton);

    await waitFor(() => {
      const state = useAuthStore.getState();
      expect(state.isAuthenticated).toBe(true);
      expect(state.user?.name).toBe('Google User');
    }, { timeout: 1500 });
  });
});
