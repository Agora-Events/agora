import React from 'react';
import { fireEvent, render } from '@testing-library/react-native';
import TermsAgreementModal from '../TermsAgreementModal';

const termsText = 'These terms explain how Agora services may be used.'.repeat(20);

function renderModal(overrides = {}) {
  const props = {
    visible: true,
    termsText,
    onAgree: jest.fn(),
    onRequestClose: jest.fn(),
    ...overrides,
  };

  return { ...render(<TermsAgreementModal {...props} />), props };
}

describe('TermsAgreementModal', () => {
  it('renders the terms and keeps the agreement action disabled initially', () => {
    const { getByText, getByTestId, props } = renderModal();

    expect(getByText(termsText)).toBeTruthy();
    expect(getByTestId('terms-agreement-confirm')).toBeDisabled();

    fireEvent.press(getByTestId('terms-agreement-confirm'));
    expect(props.onAgree).not.toHaveBeenCalled();
  });

  it('enables the agreement action only after the user reaches the bottom', () => {
    const { getByTestId, props } = renderModal();
    const scrollView = getByTestId('terms-agreement-scroll');
    const confirmButton = getByTestId('terms-agreement-confirm');

    fireEvent.scroll(scrollView, {
      nativeEvent: {
        contentOffset: { y: 150 },
        layoutMeasurement: { height: 300 },
        contentSize: { height: 600 },
      },
    });
    expect(confirmButton).toBeDisabled();

    fireEvent.scroll(scrollView, {
      nativeEvent: {
        contentOffset: { y: 300 },
        layoutMeasurement: { height: 300 },
        contentSize: { height: 600 },
      },
    });
    expect(confirmButton).toBeEnabled();

    fireEvent.press(confirmButton);
    expect(props.onAgree).toHaveBeenCalledTimes(1);
  });

  it('recalculates the end position when the viewport is resized', () => {
    const { getByTestId } = renderModal();
    const scrollView = getByTestId('terms-agreement-scroll');
    const confirmButton = getByTestId('terms-agreement-confirm');

    fireEvent.scroll(scrollView, {
      nativeEvent: {
        contentOffset: { y: 100 },
        layoutMeasurement: { height: 200 },
        contentSize: { height: 500 },
      },
    });
    expect(confirmButton).toBeDisabled();

    fireEvent(scrollView, 'layout', { nativeEvent: { layout: { height: 400 } } });
    expect(confirmButton).toBeEnabled();
  });

  it('enables the action when all terms already fit inside the viewport', () => {
    const { getByTestId } = renderModal();
    const scrollView = getByTestId('terms-agreement-scroll');

    fireEvent(scrollView, 'layout', { nativeEvent: { layout: { height: 400 } } });
    fireEvent(scrollView, 'contentSizeChange', 300, 250);

    expect(getByTestId('terms-agreement-confirm')).toBeEnabled();
  });

  it('resets the agreement state when the modal is reopened', () => {
    const props = {
      visible: true,
      termsText,
      onAgree: jest.fn(),
      onRequestClose: jest.fn(),
    };
    const { getByTestId, rerender } = render(<TermsAgreementModal {...props} />);

    fireEvent.scroll(getByTestId('terms-agreement-scroll'), {
      nativeEvent: {
        contentOffset: { y: 300 },
        layoutMeasurement: { height: 300 },
        contentSize: { height: 600 },
      },
    });
    expect(getByTestId('terms-agreement-confirm')).toBeEnabled();

    rerender(<TermsAgreementModal {...props} visible={false} />);
    rerender(<TermsAgreementModal {...props} visible />);

    expect(getByTestId('terms-agreement-confirm')).toBeDisabled();
  });

  it('allows the user to dismiss the modal without accepting the terms', () => {
    const { getByLabelText, props } = renderModal();

    fireEvent.press(getByLabelText('Close terms of service'));
    expect(props.onRequestClose).toHaveBeenCalledTimes(1);
    expect(props.onAgree).not.toHaveBeenCalled();
  });
});
