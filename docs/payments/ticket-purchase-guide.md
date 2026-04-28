# Ticket Purchase & Stellar Integration Guide

This guide explains the end-to-end technical flow of purchasing a ticket on Agora, from the frontend interaction to the Stellar smart contract integration.

## Purchase Flow Overview

The ticket purchase flow is a multi-step process involving the frontend UI, a backend API route, and the Stellar network.

1.  **User Selection**: In the `TicketModal`, the user selects the desired quantity of tickets for an event.
2.  **API Request**: Upon clicking "Confirm Purchase", the frontend sends a `POST` request to `/api/payments/ticket` with the `eventId`, `quantity`, and `buyerWallet`.
3.  **Availability Check**: The API route validates the request and checks the internal `events-store` to ensure sufficient tickets are available.
4.  **Stellar Minting**: If available, the API calls the `mintTicket` function in `utils/stellar.ts`.
5.  **Smart Contract Interaction**:
    *   `mintTicket` connects to the Stellar network using the `stellar-sdk`.
    *   It builds a transaction that calls the `mint_ticket` function on the deployed Soroban smart contract.
    *   The transaction is signed by a server-side "source" account which pays the network fees.
    *   The transaction is submitted to the Soroban RPC server.
6.  **Response**:
    *   On success, the contract updates its internal state (incrementing inventory).
    *   The API returns a `ticketId` and the `transactionXdr`.
7.  **Confirmation & QR Code**:
    *   The frontend displays a success message.
    *   A QR code is generated using the `ticketId`. This QR code serves as the attendee's proof of purchase for check-in.

## Technical Integration

### `utils/stellar.ts`

This utility file is the bridge between the application and the Stellar network.

*   **Contract Connection**: It uses the `@stellar/stellar-sdk` `Contract` class initialized with the `STELLAR_CONTRACT_ADDRESS`.
*   **Transaction Building**: Uses `TransactionBuilder` to construct the operation. It specifically calls the `mint_ticket` method of the contract.
*   **RPC Server**: It communicates with the network via an `rpc.Server` instance.

### Environment Variables

The following environment variables must be configured in your `.env` file for the integration to work:

| Variable | Description | Example |
| :--- | :--- | :--- |
| `STELLAR_CONTRACT_ADDRESS` | The ID of the deployed ticket payment contract. | `CC...` |
| `STELLAR_SOURCE_SECRET` | The private key of the account used to sign transactions and pay fees. | `S...` |
| `STELLAR_RPC_URL` | The URL of the Soroban RPC server. | `https://soroban-testnet.stellar.org` |
| `STELLAR_NETWORK_PASSPHRASE` | The passphrase for the specific Stellar network. | `Test SDF Network ; September 2015` |

### Switching Networks

To switch between Testnet and Mainnet:
1.  Update `STELLAR_RPC_URL` to the appropriate RPC endpoint.
2.  Update `STELLAR_NETWORK_PASSPHRASE` to match the target network.
3.  Ensure `STELLAR_CONTRACT_ADDRESS` refers to a contract deployed on that network.
4.  Ensure `STELLAR_SOURCE_SECRET` belongs to an account with sufficient funds on that network.

### `mintTicket()` Deep Dive

```typescript
export async function mintTicket(eventId: string, buyer: string, qty: number)
```

*   **Parameters**:
    *   `eventId`: The unique identifier of the event.
    *   `buyer`: The Stellar public key of the ticket buyer.
    *   `qty`: The number of tickets to mint.
*   **Return Value**:
    *   An object containing:
        *   `transactionXdr`: The base64 encoded transaction envelope.
        *   `ticketId`: A unique identifier for the ticket (generated from the transaction hash).
*   **Error Handling**:
    *   Throws an error if required environment variables are missing.
    *   Throws an error if parameters are invalid.
    *   Propagates RPC errors if the transaction fails to prepare or submit.

### What is `transactionXdr`?

`transactionXdr` is the External Data Representation of the Stellar transaction. It is a base64 encoded string that contains the complete transaction details, including the operations, source account, sequence number, and signatures. In the Agora platform, it provides a cryptographic proof of the on-chain minting process and can be used for debugging or manual verification on a block explorer.

## QR Code Generation

QR codes are generated on the frontend using the `qrcode.react` library.

```tsx
<QRCodeSVG
  value={purchasedTicket.id}
  size={200}
  level="H"
  includeMargin={true}
/>
```

The `value` encoded in the QR code is the `ticketId` returned by the API. During event check-in, scanners read this ID to verify the ticket's validity against the blockchain state.

## Common Errors & Fixes

| Error | Cause | Fix |
| :--- | :--- | :--- |
| `Missing required environment variable` | One or more STELLAR_* vars are missing from `.env`. | Add the missing variables to your environment configuration. |
| `Not enough tickets available` | The event has reached its capacity. | Increase event capacity in the registry or select a smaller quantity. |
| `Failed to mint ticket` (Status 502) | RPC server is down, or the transaction failed on-chain. | Check `STELLAR_RPC_URL` status or verify source account balance. |
| `Insufficient Funds` | The `STELLAR_SOURCE_SECRET` account does not have enough XLM. | Top up the source account on the respective network (e.g., using Friendbot on Testnet). |
| `Network Timeout` | The RPC server took too long to respond. | Retry the transaction or check network congestion. |
