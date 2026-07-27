# Agora Project - Progress & Implementation Summary

**Last Updated:** April 28, 2026

This document provides a comprehensive overview of everything that has been implemented in the Agora event and ticketing platform.

---

## Table of Contents

1. [Project Overview](#project-overview)
2. [Architecture](#architecture)
3. [Tech Stack](#tech-stack)
4. [Implemented Features](#implemented-features)
5. [Project Structure](#project-structure)
6. [Frontend Implementation](#frontend-implementation)
7. [Backend Services](#backend-services)
8. [Smart Contracts](#smart-contracts)
9. [Database & Migrations](#database--migrations)
10. [Design & UI](#design--ui)
11. [Getting Started](#getting-started)

---

## Project Overview

**Agora** is a comprehensive event and ticketing platform designed for organizers, creators, and communities to create events, sell tickets, and manage attendees with ease. The platform is built on the **Stellar blockchain** and enables fast, low-cost, borderless payments using USDC.

### Core Value Proposition

- **0% Platform Fees** on Pro plan
- **Instant Payouts** via Stellar USDC
- **Community Features** - Follow organizers, discover events
- **Seamless Ticketing** - Create and sell tickets
- **Blockchain-Powered** - Transparent, secure transactions

### Live Demo

**[https://agora-web-eta.vercel.app/](https://agora-web-eta.vercel.app/)**

---

## Architecture

Agora follows a **monorepo architecture** with three interconnected pillars:

```
┌─────────────────────────────────────────────────────┐
│              Frontend (Next.js/React)                │
│         apps/web - User Interface Layer              │
└────────────────────┬────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────┐
│        Backend API (Rust/Axum + PostgreSQL)          │
│    server - Business Logic & Data Persistence        │
└────────────────────┬────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────┐
│      Smart Contracts (Soroban - Rust)               │
│  contract - On-chain Event & Payment Management      │
└─────────────────────────────────────────────────────┘
                     │
                     ▼
              Stellar Network
         (USDC, Payments, Storage)
```

### Key Components

| Component | Technology | Purpose |
|-----------|-----------|---------|
| **apps/web** | Next.js 14+, React, Tailwind CSS, Framer Motion | User-facing application |
| **server** | Rust, Axum, SQLx, PostgreSQL | REST API & Business Logic |
| **contract** | Rust, Soroban, Cargo | Blockchain smart contracts |
| **Package Manager** | pnpm | Dependency management |

---

## Tech Stack

### Frontend
- **Framework:** Next.js 14+ (App Router)
- **UI Library:** React 19+
- **Styling:** Tailwind CSS 4.1+
- **Animations:** Framer Motion
- **Language:** TypeScript 5.9+
- **Linting:** ESLint 9+
- **Formatting:** Prettier 3.8+
- **Package Manager:** pnpm 10.28+

### Backend
- **Language:** Rust (stable)
- **Framework:** Axum (modern async web framework)
- **Database:** PostgreSQL
- **ORM/Query:** SQLx (compile-time query verification)
- **Migrations:** SQLx CLI with SQL migrations

### Blockchain
- **Platform:** Stellar Network
- **Smart Contracts:** Soroban (Rust)
- **Build Tool:** Cargo (Rust package manager)
- **Payment Token:** USDC (Stellar-native stablecoin)

### Infrastructure
- **Containerization:** Docker
- **Orchestration:** Docker Compose
- **Environment:** PostgreSQL in container for local dev

---

## Implemented Features

### User-Facing Features

#### 1. **Authentication & Authorization**
- ✅ User signup/login system
- ✅ Profile management
- ✅ Account settings
- ✅ Role-based access (Organizer, Attendee)

#### 2. **Event Management**
- ✅ Event creation and customization
- ✅ Event discovery and browsing
- ✅ Event details pages
- ✅ Event status management (Active, Cancelled, Archived)
- ✅ Event metadata management
- ✅ Multi-tier event pricing
- ✅ Event ratings and reviews

#### 3. **Ticketing System**
- ✅ Ticket creation with multiple tiers
- ✅ Ticket purchasing flow
- ✅ Ticket inventory management
- ✅ Promo codes and discounts
- ✅ Refund policies
- ✅ Series passes and season tickets
- ✅ QR code generation for tickets
- ✅ Ticket transfer between users

#### 4. **Payments & Payouts**
- ✅ Stellar USDC payment integration
- ✅ Platform fee configuration
- ✅ Escrow and settlement system
- ✅ Instant payouts to organizers
- ✅ Payment tracking and history
- ✅ Multi-currency support (via USDC)

#### 5. **Community Features**
- ✅ Follow organizers
- ✅ Organizer profiles and discovery
- ✅ Event recommendations
- ✅ Trending events
- ✅ Community engagement

#### 6. **Organizer Dashboard**
- ✅ Event analytics and metrics
- ✅ Attendee management
- ✅ Sales reports
- ✅ Revenue tracking
- ✅ QR code scanner integration
- ✅ Event settings management

#### 7. **Admin Features**
- ✅ Multi-admin governance
- ✅ Organizer blacklisting
- ✅ Platform fee management
- ✅ Token whitelisting
- ✅ Global policy management
- ✅ Audit logging

---

## Project Structure

### Root Directory

```
agora/
├── ARCHITECTURE.md                    # High-level architecture guide
├── DEVELOPMENT_SETUP.md              # Setup instructions
├── README.md                         # Project overview
├── PROGRESS.md                       # This file
├── TROUBLESHOOTING.md                # Common issues
├── LICENSE.md                        # MIT License
├── package.json                      # Root workspace config
├── pnpm-workspace.yaml               # pnpm monorepo config
├── pnpm-lock.yaml                    # Dependency lock file
├── Cargo.toml                        # Rust workspace
├── clippy.toml                       # Rust linting config
├── contributor.md                   # Contribution guidelines
│
├── apps/web/                         # Next.js Frontend
├── server/                           # Rust Backend API
├── contract/                         # Soroban Smart Contracts
├── docs/                             # API documentation
├── design/                           # Design resources & mockups
└── src/                              # Shared Rust utilities
```

---

## Frontend Implementation

### Location: `apps/web/`

#### **Pages & Routes**

```
app/
├── page.tsx                    # Home page
├── auth/                       # Authentication pages
│   ├── login                   # Login
│   └── signup                  # Registration
├── create-event/               # Event creation
├── events/                     # Event listings
│   └── [id]                    # Event details
├── discover/                   # Event discovery & search
├── organizers/                 # Organizer profiles
│   └── [id]                    # Organizer details
├── profile/                    # User profiles
│   └── [id]                    # Profile view
├── pricing/                    # Pricing plans
├── home/                       # Home/landing area
├── faqs/                       # FAQ pages
├── stellar/                    # Stellar integration
└── api/                        # API routes (internal)
```

#### **Components Architecture**

```
components/
├── events/                     # Event-related components
│   └── Event cards, listings, details
├── landing/                    # Landing page components
├── layout/                     # Layout wrappers, headers, footers
├── profile/                    # Profile components
├── recommendations/            # Recommendation engine UI
└── ui/                        # Reusable UI components
```

#### **Utilities & Hooks**

```
lib/
├── auth.ts                     # Authentication logic
├── constants.ts                # App constants
├── events-store.ts             # Event state management
└── DOCS/COMPONENTS.md          # Component documentation

hooks/
├── useRecommendedEvents.ts    # Event recommendations hook

utils/
├── api.ts                      # API client utilities
├── stellar.ts                  # Stellar integration
└── stellar.test.ts             # Stellar tests
```

#### **Styling**

- **Tailwind CSS 4.1+** - Utility-first CSS framework
- **Framer Motion** - Smooth animations and transitions
- **Custom CSS** - `globals.css` for global styles
- **PostCSS** - CSS processing pipeline

#### **Public Assets**

```
public/
├── logo/                       # Agora branding
├── icons/                      # Icon library
├── images/                     # Product images
├── backgrounds/                # Background images
└── design.txt, *design.txt     # Design specs
```

---

## Backend Services

### Location: `server/`

#### **Technology Stack**
- **Framework:** Axum (async Rust web framework)
- **Database:** PostgreSQL via SQLx
- **Architecture:** Clean separation of concerns

#### **Directory Structure**

```
src/
├── main.rs                     # Server entry point
├── lib.rs                      # Module exports
├── config/                     # Configuration management
├── handlers/                   # Request handlers (business logic)
├── routes/                     # API endpoints
├── models/                     # Data models
├── middleware/                 # Custom middleware
├── utils/                      # Shared utilities
└── notifications/              # Notification system

migrations/                     # Database migrations
├── 20260121174426_initial_schema.sql
├── 20260425000001_categories_and_audit_logs.sql
├── 20260425000002_qr_payloads.sql
└── 20260428000003_event_ratings.sql

docker-compose.yml              # PostgreSQL container config
.env.example                    # Environment variables template
```

#### **API Endpoints** (Versioned)

The backend provides RESTful API endpoints for:
- User authentication and management
- Event CRUD operations
- Ticket creation and purchase
- Payment processing
- Organizer management
- Admin operations
- Event analytics

#### **Database Migrations**

| Migration | Date | Purpose |
|-----------|------|---------|
| `initial_schema.sql` | 2026-01-21 | Core tables for events, users, tickets |
| `categories_and_audit_logs.sql` | 2026-04-25 | Event categories & audit trail |
| `qr_payloads.sql` | 2026-04-25 | QR code data storage |
| `event_ratings.sql` | 2026-04-28 | Event rating system |

#### **Key Features**

- ✅ PostgreSQL database with schema migrations
- ✅ User authentication & JWT tokens
- ✅ Event management (CRUD)
- ✅ Ticket lifecycle (create, purchase, transfer, refund)
- ✅ Payment coordination with Stellar
- ✅ QR code generation & validation
- ✅ Event ratings system
- ✅ Audit logging for compliance
- ✅ CORS configuration for frontend
- ✅ Health check endpoints

---

## Smart Contracts

### Location: `contract/`

#### **Purpose**

Smart contracts handle on-chain event state, ticket payments, and governance on the Stellar blockchain.

#### **Contracts**

##### **1. Event Registry Contract**
**Location:** `contracts/event_registry/`

**Responsibilities:**
- Event lifecycle management (create, update, cancel, archive)
- Event metadata storage (CID-based)
- Tier configuration and inventory
- Organizer ownership & permissions
- Scanner authorization
- Loyalty program management
- Staking & verification system
- Multi-admin governance
- Series/season passes
- Organizer blacklisting
- Audit trail

**Key Storage Keys:**
- `Event(event_id)` - Full event information
- `OrganizerEvent*` - Event-to-organizer mappings
- `EventReceipt` - Archived event history
- `MultiSigConfig` - Admin governance
- `GuestProfile` - Loyalty tracking
- `OrganizerStake` - Staking records
- `TokenWhitelist` - Accepted payment tokens
- `GlobalEventCount` - Platform statistics

**Main Functions:**
```rust
initialize()              // One-time setup
register_event()          // Create event
get_event()              // Retrieve event
update_event_status()    // Activate/deactivate
cancel_event()           // Cancel event
archive_event()          // Archive & cleanup
update_metadata()        // Update event info
get_organizer_address()  // Event ownership
get_total_tickets_sold() // Inventory tracking
```

##### **2. Ticket Payment Contract**
**Location:** `contracts/ticket_payment/`

**Responsibilities:**
- Ticket purchase transactions
- Payment escrow & settlement
- Refund processing
- Fund transfers
- Auction support
- Payment governance
- Token swap capabilities
- Security & validation

**Key Features:**
- Escrow mechanism for buyer protection
- Instant settlement to organizers
- Multiple payment token support
- Refund policies enforcement
- Transaction history

#### **Build & Testing**

```
contracts/
├── Cargo.toml              # Workspace definition
├── scripts/
│   ├── deploy_devnet.sh    # Deploy to testnet
│   └── generate_coverage.sh # Code coverage
└── README.md               # Contract documentation
```

**Available Commands:**
- `cargo build` - Compile contracts
- `cargo test` - Run contract tests
- `./scripts/deploy_devnet.sh` - Deploy to testnet
- `./scripts/generate_coverage.sh` - Generate coverage

---

## Database & Migrations

### PostgreSQL Schema

The application uses PostgreSQL for persistent data storage with SQLx for type-safe queries.

#### **Implemented Migrations**

**1. Initial Schema** (2026-01-21)
- Users table
- Events table
- Tickets table
- Transactions table
- Core relationships

**2. Categories & Audit Logs** (2026-04-25)
- Event categories
- Audit log tracking
- Compliance records

**3. QR Payloads** (2026-04-25)
- QR code metadata
- Scanner integration

**4. Event Ratings** (2026-04-28)
- Rating system
- Review tracking
- Attendee feedback

#### **Local Development**

PostgreSQL runs in Docker via `docker-compose.yml`:

```yaml
version: '3.8'
services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: agora
      POSTGRES_USER: user
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
```

**Connection String (Local):**
```
postgres://user:password@localhost:5432/agora
```

---

## Design & UI

### Design Resources

**Location:** `design/`

#### **Design Files & Mockups**

```
design/
├── Account Settings Page Design.md
├── chat-empty-state.md
├── chat-filled-state.md
├── issue-246.md           # Design specs for feature
├── issue-247.md           # Design specs for feature
├── issue-250.md           # Design specs for feature
├── issue-251.md           # Design specs for feature
└── README.md              # Design guide
```

#### **Figma Design File**

**[Agora Event - Figma Design](https://www.figma.com/design/cpRUhrSlBVxGElm18Fa2Uh/Agora-event?node-id=0-1&t=qBlO0jnjQHQaHn2Z-1)**

This is the source of truth for UI/UX design. All new features should reference the Figma file for design consistency.

#### **Component Documentation**

**Location:** `apps/web/DOCS/COMPONENTS.md`

Comprehensive documentation of all reusable UI components including:
- Usage examples
- Props documentation
- Styling guidelines
- State variations

#### **Design System**

- **Color Palette:** Defined in Tailwind config
- **Typography:** Responsive font scales
- **Spacing:** 8px base unit system
- **Components:** Button, Card, Modal, Form inputs, etc.
- **Animations:** Framer Motion presets

---

## Getting Started

### Prerequisites

Before starting development, ensure you have installed:

```bash
# Check Node.js and pnpm
node --version
pnpm --version

# Check Rust toolchain
rustc --version
cargo --version

# Check Docker
docker --version
docker compose version

# Check database tools
sqlx --version

# Check Soroban CLI
soroban --version
```

### Installation Steps

#### 1. Clone & Install

```bash
git clone https://github.com/Agora-Events/agora.git
cd agora
pnpm install
```

#### 2. Configure Environment

```bash
# Backend configuration
cp server/.env.example server/.env

# Optional: Contract deployment (testnet only)
cp contract/.env.devnet.example contract/.env.devnet
```

#### 3. Start Infrastructure

```bash
# Start PostgreSQL in Docker
docker compose -f server/docker-compose.yml up -d

# Run database migrations
cd server
sqlx migrate run
```

#### 4. Start Development Servers

**Terminal 1 - Backend API:**
```bash
cd server
cargo run
```

**Terminal 2 - Frontend:**
```bash
cd apps/web
pnpm dev
```

**Terminal 3 - Contract Development (Optional):**
```bash
cd contract
cargo build
cargo test
```

### Access Points

- **Frontend:** http://localhost:3000
- **Backend API:** http://localhost:3001
- **PostgreSQL:** localhost:5432

---

## Project Status

### ✅ Completed

- [x] User authentication system
- [x] Event creation & management
- [x] Ticket system with multiple tiers
- [x] Stellar/USDC payment integration
- [x] Event discovery & recommendations
- [x] Organizer profiles & management
- [x] Event ratings & reviews
- [x] QR code generation
- [x] Refund system
- [x] Admin governance
- [x] Audit logging
- [x] Database migrations
- [x] API documentation
- [x] Component documentation
- [x] Docker development environment

### 🔄 In Progress / Planned

- [ ] Enhanced analytics dashboard
- [ ] Real-time notifications
- [ ] Series/Season passes refinement
- [ ] Advanced search filters
- [ ] Mobile app optimization
- [ ] Mainnet deployment
- [ ] Performance optimization

---

## Contributing

We welcome contributions from the community! Please see [contributor.md](contributor.md) for detailed guidelines.

### Quick Start for Contributors

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Make your changes
4. Ensure linting passes: `pnpm lint`
5. Commit: `git commit -m 'Add feature: description'`
6. Push: `git push origin feature/your-feature`
7. Open a Pull Request

### Code Standards

- **Frontend:** Follow Tailwind CSS conventions and Next.js best practices
- **Backend:** Follow Rust style guide and Axum patterns
- **Contracts:** Follow Soroban conventions and security best practices

---

## Documentation

- [README.md](README.md) - Project overview
- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture
- [DEVELOPMENT_SETUP.md](DEVELOPMENT_SETUP.md) - Detailed setup guide
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Common issues
- [apps/web/README.md](apps/web/README.md) - Frontend guidelines
- [server/README.md](server/README.md) - Backend documentation
- [contract/README.md](contract/README.md) - Smart contract documentation
- [docs/API_REFERENCE.md](docs/API_REFERENCE.md) - API endpoints
- [apps/web/DOCS/COMPONENTS.md](apps/web/DOCS/COMPONENTS.md) - UI components

---

## License

Distributed under the MIT License. See [LICENSE.md](LICENSE.md) for details.

---

## Contact & Support

For questions or support:
- 📧 Email: support@agora.events
- 💬 Discord: [Join Community](https://discord.gg/agora)
- 🐛 Issues: [GitHub Issues](https://github.com/Agora-Events/agora/issues)

---

**© 2026 Agora. All rights reserved.**

*Last Updated: April 28, 2026*
