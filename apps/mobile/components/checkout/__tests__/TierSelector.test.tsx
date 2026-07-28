import React from 'react';
import { render, fireEvent } from '@testing-library/react-native';
import TierSelector from '../TierSelector';
import type { TicketTierOption } from '@/types/checkout';

const tiers: TicketTierOption[] = [
  { id: 'ga', name: 'General Admission', priceUsdc: 25, remaining: 40 },
  { id: 'vip', name: 'VIP', description: 'Front row', priceUsdc: 100, remaining: 3 },
  { id: 'early', name: 'Early Bird', priceUsdc: 10, remaining: 0 },
];

describe('TierSelector', () => {
  it('renders a card for every tier with its formatted USDC price', () => {
    const { getByText } = render(
      <TierSelector tiers={tiers} selectedTierId={null} onSelect={jest.fn()} />
    );
    expect(getByText('General Admission')).toBeTruthy();
    expect(getByText('25.00 USDC')).toBeTruthy();
    expect(getByText('100.00 USDC')).toBeTruthy();
  });

  it('shows "Free" for a zero-price tier instead of "0.00 USDC"', () => {
    const freeTiers: TicketTierOption[] = [{ id: 'free', name: 'Community Pass', priceUsdc: 0 }];
    const { getByText } = render(
      <TierSelector tiers={freeTiers} selectedTierId={null} onSelect={jest.fn()} />
    );
    expect(getByText('Free')).toBeTruthy();
  });

  it('calls onSelect with the tapped tier id', () => {
    const onSelect = jest.fn();
    const { getByTestId } = render(
      <TierSelector tiers={tiers} selectedTierId={null} onSelect={onSelect} />
    );
    fireEvent.press(getByTestId('tier-card-vip'));
    expect(onSelect).toHaveBeenCalledWith('vip');
  });

  it('marks a sold-out tier disabled and does not fire onSelect for it', () => {
    const onSelect = jest.fn();
    const { getByTestId, getByText } = render(
      <TierSelector tiers={tiers} selectedTierId={null} onSelect={onSelect} />
    );
    expect(getByText('Sold out')).toBeTruthy();
    fireEvent.press(getByTestId('tier-card-early'));
    expect(onSelect).not.toHaveBeenCalled();
  });

  it('shows remaining inventory for tiers that report it', () => {
    const { getByText } = render(
      <TierSelector tiers={tiers} selectedTierId={null} onSelect={jest.fn()} />
    );
    expect(getByText('40 remaining')).toBeTruthy();
    expect(getByText('3 remaining')).toBeTruthy();
  });
});
