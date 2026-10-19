# Payment & Ticket Minting Flow

This document outlines the architecture for purchasing a ticket and minting it on the Stellar blockchain.

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant UI as Frontend
    participant Modal as TicketModal
    participant API as /api/payments/ticket
    participant Utils as utils/stellar.ts
    participant Stellar as Stellar Smart Contract
    
    User->>UI: Clicks "Buy Ticket"
    UI->>Modal: Opens TicketModal
    User->>Modal: Confirms Purchase Details
    Modal->>API: POST /api/payments/ticket
    
    rect rgb(240, 248, 255)
        Note over API, Stellar: Blockchain Transaction
        API->>Utils: Calls mintTicket()
        Utils->>Stellar: Executes Contract Mint
        Stellar-->>Utils: Returns transactionXdr + ticketId
        Utils-->>API: Returns Transaction Data
    end
    
    API-->>Modal: Returns Success Data
    Modal->>UI: Renders QR Code
    UI-->>User: Displays Ticket & QR