# <img src="apps/web/public/logo/agora%20logo.svg" alt="Agora Logo" width="50" /> Agora

[![Chromatic](https://chromatic.com/badge?appCode=)](https://www.chromatic.com/)

**Plan Events. Bring People Together. Grow Communities.**

Agora is an event and ticketing platform built for organizers, creators, and communities to create events, sell tickets, and manage attendees with ease. Built on [Stellar](https://stellar.org), it enables fast, low-cost, borderless payments using USDC.

**Live Demo:** [https://agora-web-eta.vercel.app/](https://agora-web-eta.vercel.app/)

## Features

- **Event Management**: Create and customize event pages.
- **Ticketing**: Sell tickets seamlessly with 0% platform fees on the Pro plan.
- **Payments**: Instant payouts via Stellar USDC.
- **Community**: Follow organizers and discover events.

## Tech Stack

- **Frontend**: Next.js, React, Tailwind CSS, Framer Motion.
- **Blockchain**: Stellar Smart Contracts (Soroban).
- **Package Manager**: pnpm.

## Repository Structure

This project is organized as a monorepo:

- [`apps/web`](apps/web/README.md): The main frontend application (Next.js). **Please read the [Frontend Guidelines](apps/web/README.md) regarding styles and components before contributing.**
- [`apps/mobile`](apps/mobile/README.md): The mobile application (Expo).
- [`contract`](contract/README.md): Smart contracts and blockchain logic.
- [`docs`](docs/): Detailed platform documentation and guides.

## 📚 Documentation

- [**Ticket Purchase & Stellar Integration Guide**](docs/payments/ticket-purchase-guide.md): Deep dive into the ticketing technical flow.
- [**Stellar Testnet Setup Guide**](docs/stellar/testnet-setup-guide.md): Install Freighter, fund via Friendbot, add a USDC trustline, and run your first Testnet purchase.
- [**Stellar Smart Contracts**](docs/contracts/stellar-contract.md): Overview of our on-chain logic.
- [**Database Schema**](docs/DATABASE_SCHEMA.md): Detailed view of the system's data model.

## Backend Architecture (Axum)

The backend is built using **Rust** and the **Axum** web framework, following a clean and modular architecture designed for scalability and maintainability.

### Directory Overview

- `routes/` – API route definitions and versioned endpoints
- `handlers/` – Request handlers (controllers/business logic)
- `models/` – Data models and domain structures
- `utils/` – Shared helpers and utilities
- `config/` – Environment and configuration management
- `main.rs` – Application entry point and server bootstrap
- `lib.rs` – Central module exports

This separation of concerns allows the backend to scale independently, supports clean testing, and keeps business logic isolated from routing and infrastructure code.

## Getting Started

### 1. Clone the repository

```bash
git clone https://github.com/Agora-Events/agora.git
cd agora
```

### 2. Install dependencies

Install workspace dependencies from the repository root:

```bash
pnpm install
```

### 3. Run the development server

Start the Next.js web application from the repository root:

```bash
pnpm dev
# or explicitly target the web workspace:
pnpm --filter web dev
```

Alternatively, navigate into the web application directory:

```bash
cd apps/web
pnpm dev
```

The frontend will be available at [http://localhost:3000](http://localhost:3000).

### 4. Full-Stack Setup (Docker)

To run the complete local stack (PostgreSQL, Redis, Stellar RPC, Axum backend, and Next.js frontend) with a single command:

```bash
make up
# or directly:
docker compose up --build
```

- **Frontend:** [http://localhost:3000](http://localhost:3000)
- **Backend API:** [http://localhost:3001](http://localhost:3001)

For detailed instructions on environment variables, database migrations (`sqlx migrate run`), backend manual runs, and Soroban smart contract testing, refer to the [Development Setup Guide](docs/DEVELOPMENT_SETUP.md).

## Design Resources

- [**Figma Design File**](https://www.figma.com/design/cpRUhrSlBVxGElm18Fa2Uh/Agora-event?node-id=0-1&t=qBlO0jnjQHQaHn2Z-1)

## Contributing

We welcome contributions from the community! To contribute:

1. **Fork the Project**.
2. **Clone your fork**:
   ```bash
   git clone https://github.com/<your-username>/agora.git
   cd agora
   ```
3. **Create your Feature Branch** (`git checkout -b feature/AmazingFeature`).
4. **Commit your Changes** (`git commit -m 'Add some AmazingFeature'`).
5. **Push to the Branch** (`git push origin feature/AmazingFeature`).
6. **Open a Pull Request**.

Please ensure your code follows the existing style guidelines (see [Frontend Guidelines](apps/web/README.md) for web) and passes all linting checks (`pnpm lint`).

**If you find this project useful, please give it a star! ⭐️**

## License

Distributed under the MIT License. See [`LICENSE.md`](LICENSE.md) for more information.

---

© 2026 Agora. All rights reserved.

## Handsoff notes

<!-- handsoff-issue-1397 -->

- #1397: [FRONTEND] Remove debug console.log from create-event form submit
