import React from 'react';
import { render, fireEvent } from '@testing-library/react-native';
import CheckoutProgressModal from '../CheckoutProgressModal';
import { CHECKOUT_STEP_LABELS, CHECKOUT_STEP_ORDER, type CheckoutStep } from '@/types/checkout';

function buildSteps(overrides: Partial<Record<string, CheckoutStep['status']>> = {}): CheckoutStep[] {
  return CHECKOUT_STEP_ORDER.map((id) => ({
    id,
    label: CHECKOUT_STEP_LABELS[id],
    status: overrides[id] ?? 'pending',
  }));
}

describe('CheckoutProgressModal', () => {
  it('renders every step label so the buyer can see what stage they are at', () => {
    const steps = buildSteps({ 'build-approval': 'done', 'sign-approval': 'active' });
    const { getByText } = render(
      <CheckoutProgressModal
        visible
        steps={steps}
        phase="in-progress"
        errorMessage={null}
        onRetry={jest.fn()}
        onDismiss={jest.fn()}
      />
    );

    expect(getByText(CHECKOUT_STEP_LABELS['build-approval'])).toBeTruthy();
    expect(getByText(CHECKOUT_STEP_LABELS['confirm-payment'])).toBeTruthy();
    expect(getByText('Processing Your Purchase')).toBeTruthy();
  });

  it('shows the poll attempt detail text under the active step', () => {
    const steps = buildSteps({ 'confirm-approval': 'active' });
    steps.find((s) => s.id === 'confirm-approval')!.detail = 'Attempt 4 of 30...';

    const { getByText } = render(
      <CheckoutProgressModal
        visible
        steps={steps}
        phase="in-progress"
        errorMessage={null}
        onRetry={jest.fn()}
        onDismiss={jest.fn()}
      />
    );

    expect(getByText('Attempt 4 of 30...')).toBeTruthy();
  });

  it('shows the failure title, readable error message, and Retry/Dismiss actions on error', () => {
    const steps = buildSteps({ 'submit-payment': 'error' });
    const onRetry = jest.fn();
    const onDismiss = jest.fn();

    const { getByText, getByTestId } = render(
      <CheckoutProgressModal
        visible
        steps={steps}
        phase="error"
        errorMessage="Your USDC spending approval has not gone through yet. Please try again in a moment."
        onRetry={onRetry}
        onDismiss={onDismiss}
      />
    );

    expect(getByText('Checkout Failed')).toBeTruthy();
    expect(
      getByText('Your USDC spending approval has not gone through yet. Please try again in a moment.')
    ).toBeTruthy();

    fireEvent.press(getByTestId('checkout-progress-retry'));
    expect(onRetry).toHaveBeenCalledTimes(1);

    fireEvent.press(getByTestId('checkout-progress-dismiss'));
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });

  it('renders without crashing when visible is false', () => {
    // Note: RN's <Modal> keeps its children mounted (just not natively
    // presented) when `visible` is false, so this only guards against a
    // render-time crash rather than asserting the content is hidden.
    expect(() =>
      render(
        <CheckoutProgressModal
          visible={false}
          steps={buildSteps()}
          phase="idle"
          errorMessage={null}
          onRetry={jest.fn()}
          onDismiss={jest.fn()}
        />
      )
    ).not.toThrow();
  });
});
