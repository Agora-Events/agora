import React from 'react';
import { render, fireEvent } from '@testing-library/react-native';
import QuantityStepper from '../QuantityStepper';

describe('QuantityStepper', () => {
  it('renders the current value', () => {
    const { getByTestId } = render(<QuantityStepper value={3} onChange={jest.fn()} />);
    expect(getByTestId('quantity-value').props.children).toBe(3);
  });

  it('calls onChange with value + 1 when the increment button is pressed', () => {
    const onChange = jest.fn();
    const { getByTestId } = render(<QuantityStepper value={2} onChange={onChange} max={10} />);
    fireEvent.press(getByTestId('quantity-increment'));
    expect(onChange).toHaveBeenCalledWith(3);
  });

  it('calls onChange with value - 1 when the decrement button is pressed', () => {
    const onChange = jest.fn();
    const { getByTestId } = render(<QuantityStepper value={2} onChange={onChange} min={1} />);
    fireEvent.press(getByTestId('quantity-decrement'));
    expect(onChange).toHaveBeenCalledWith(1);
  });

  it('disables the decrement button at the minimum', () => {
    const onChange = jest.fn();
    const { getByTestId } = render(<QuantityStepper value={1} onChange={onChange} min={1} />);
    fireEvent.press(getByTestId('quantity-decrement'));
    expect(onChange).not.toHaveBeenCalled();
  });

  it('disables the increment button at the maximum (e.g. tier inventory cap)', () => {
    const onChange = jest.fn();
    const { getByTestId } = render(<QuantityStepper value={5} onChange={onChange} max={5} />);
    fireEvent.press(getByTestId('quantity-increment'));
    expect(onChange).not.toHaveBeenCalled();
  });

  it('ignores presses entirely while disabled', () => {
    const onChange = jest.fn();
    const { getByTestId } = render(
      <QuantityStepper value={2} onChange={onChange} min={1} max={5} disabled />
    );
    fireEvent.press(getByTestId('quantity-increment'));
    fireEvent.press(getByTestId('quantity-decrement'));
    expect(onChange).not.toHaveBeenCalled();
  });
});
