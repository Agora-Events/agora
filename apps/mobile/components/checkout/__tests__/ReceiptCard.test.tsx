import React from 'react';
import { render, fireEvent, waitFor } from '@testing-library/react-native';
import * as Clipboard from 'expo-clipboard';
import ReceiptCard from '../ReceiptCard';
import type { CheckoutReceipt } from '@/types/checkout';

jest.mock('expo-clipboard', () => ({
  setStringAsync: jest.fn().mockResolvedValue(true),
}));

const receipt: CheckoutReceipt = {
  ticketId: 'ticket-abc123',
  paymentId: 'pay-evt-1-abc123',
  eventId: 'evt-1',
  eventTitle: 'Stellar Meridian 2026',
  tierName: 'General Admission',
  quantity: 2,
  unitPriceUsdc: 25,
  platformFeeUsdc: 1.25,
  totalPaidUsdc: 50,
  approvalTxHash: 'a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2',
  paymentTxHash: 'f6e5d4c3b2a1f6e5d4c3b2a1f6e5d4c3b2a1f6e5',
  buyerPublicKey: 'GA7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVSGZ',
  completedAt: '2026-07-28T12:00:00.000Z',
};

describe('ReceiptCard', () => {
  it('renders the event, tier, quantity, ids, and totals', () => {
    const { getByText } = render(<ReceiptCard receipt={receipt} />);

    expect(getByText('Stellar Meridian 2026')).toBeTruthy();
    expect(getByText('General Admission × 2')).toBeTruthy();
    expect(getByText('ticket-abc123')).toBeTruthy();
    expect(getByText('pay-evt-1-abc123')).toBeTruthy();
    expect(getByText('50.00 USDC')).toBeTruthy(); // total paid
  });

  it('truncates the buyer wallet address for display', () => {
    const { getByText } = render(<ReceiptCard receipt={receipt} />);
    expect(getByText('GA7QYN...UJVSGZ')).toBeTruthy();
  });

  it('copies the approval tx hash to the clipboard when tapped, and shows confirmation', async () => {
    const { getByTestId, queryAllByText } = render(<ReceiptCard receipt={receipt} />);

    fireEvent.press(getByTestId('tx-row-approval-tx'));

    expect(Clipboard.setStringAsync).toHaveBeenCalledWith(receipt.approvalTxHash);
    await waitFor(() => expect(queryAllByText('Copied!').length).toBeGreaterThan(0));
  });

  it('links out to the Stellar Expert explorer for both transactions', () => {
    const { getByText } = render(<ReceiptCard receipt={receipt} />);
    expect(
      getByText(`https://stellar.expert/explorer/testnet/tx/${receipt.approvalTxHash}`)
    ).toBeTruthy();
    expect(
      getByText(`https://stellar.expert/explorer/testnet/tx/${receipt.paymentTxHash}`)
    ).toBeTruthy();
  });
});
