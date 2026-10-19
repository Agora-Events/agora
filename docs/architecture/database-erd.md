# Database Schema ERD

## Overview
Agora Events uses a Prisma PostgreSQL schema with two primary entities:

### Event
Represents hosted events on the platform.

**Core responsibilities:**
- Event metadata
- Organizer identity
- Ticket pricing
- Capacity limits
- Follower-only restrictions

**Relationships:**
- One Event can have many Tickets

---

### Ticket
Represents purchased or minted event tickets.

**Core responsibilities:**
- Ticket ownership
- Buyer wallet tracking
- Event linkage
- Quantity management
- Optional Stellar asset linkage

**Relationships:**
- Each Ticket belongs to one Event

---

## Relationship Summary
- Event → Ticket (One-to-Many)

---

## Maintenance Workflow
Whenever schema changes:
1. Update `apps/web/prisma/schema.prisma`
2. Run:
   `npx prisma generate --schema=apps/web/prisma/schema.prisma`
3. Export updated ERD PNG
4. Update this documentation