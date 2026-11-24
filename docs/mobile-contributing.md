# Mobile Development Contributing Guide

Welcome to the Agora Mobile contributing guide! This guide covers everything you need to know to set up your local development environment, run the mobile app using Expo, connect to the backend server, and execute tests.

---

## 1. Overview & Architecture

The Agora mobile app is built with:
- **Framework:** React Native with [Expo](https://expo.dev/) (managed workflow)
- **Routing:** [Expo Router](https://docs.expo.dev/router/introduction/) (file-based routing located in `apps/mobile/app/`)
- **Language:** TypeScript
- **State & Hooks:** Custom hooks for Stellar wallet integration, authentication, offline ticket vault, and seating
- **Styling:** React Native `StyleSheet` with shared design tokens from `constants/Colors.ts`

---

## 2. Prerequisites

Before getting started, make sure you have installed:

- **Node.js:** v18.x or later (v20+ recommended)
- **Package Manager:** `pnpm` (version 10+)
- **Mobile Environment (choose one or more):**
  - **Physical Device:** Install the **Expo Go** app from the App Store (iOS) or Google Play Store (Android).
  - **iOS Simulator:** Requires macOS with Xcode and Xcode Command Line Tools installed.
  - **Android Emulator:** Requires Android Studio with Android SDK and an Android Virtual Device (AVD) configured.

---

## 3. Installation & Setup

1. **Clone the repository and install root dependencies:**
   ```bash
   git clone https://github.com/Agora-Events/agora.git
   cd agora
   pnpm install
   ```

2. **Navigate to the mobile app workspace and install packages:**
   ```bash
   cd apps/mobile
   pnpm install
   ```

3. **Configure Environment Variables:**
   Copy the example environment configuration into `.env`:
   ```bash
   cp .env.example .env
   ```
   Set your backend API endpoint in `apps/mobile/.env`:
   ```ini
   EXPO_PUBLIC_API_URL=http://localhost:8080
   ```

---

## 4. Connecting to the Local Backend Server

The mobile app communicates with the Agora backend server. When running locally, configure `EXPO_PUBLIC_API_URL` based on your platform:

| Target Platform | `EXPO_PUBLIC_API_URL` Value | Notes |
|---|---|---|
| **iOS Simulator** | `http://localhost:8080` | Shares localhost directly with macOS host |
| **Android Emulator** | `http://10.0.2.2:8080` | `10.0.2.2` maps to the host machine's loopback interface |
| **Physical Device (Expo Go)** | `http://<YOUR_LAN_IP>:8080` (e.g. `http://192.168.1.150:8080`) | Device and development computer must be connected to the same Wi-Fi network |

To start the backend server locally (from repository root):
```bash
cargo run --manifest-path server/Cargo.toml
```

---

## 5. Starting the Expo Development Server

To launch Metro bundler and start the mobile application:

```bash
cd apps/mobile
pnpm start
```

Once Metro is running, press one of the following interactive shortcuts in your terminal:
- `a` — Open in connected **Android Emulator**
- `i` — Open in **iOS Simulator**
- `w` — Open in **Web Browser**
- `r` — Reload the connected app
- `c` — Clear Metro bundler cache

To scan the QR code on a physical device, open the **Expo Go** app and scan the terminal QR code.

---

## 6. Running Tests & Quality Checks

Run all verification scripts from `apps/mobile/`:

### Run Jest Unit Tests
```bash
pnpm test
```
To run tests in watch mode:
```bash
pnpm test -- --watch
```

### Run ESLint
```bash
pnpm lint
```

### Run TypeScript Type Check
```bash
pnpm typecheck
```

---

## 7. Project Structure

```
apps/mobile/
├── app/                  # Expo Router file-based pages and screens
│   ├── (tabs)/           # Bottom tab navigation (Discover, Tickets, Profile)
│   ├── checkout/         # Ticket checkout and waiting room flow
│   ├── create-event/     # Multi-step event creation wizard
│   ├── event/            # Event details screen
│   └── organizer/        # Ticket scanner & gate verification
├── components/           # Reusable UI widgets and feature components
│   └── ui/               # Core atomic primitives (Button, Input, ErrorBoundary)
├── constants/            # Design tokens, theme palettes, and Colors
├── hooks/                # Custom React hooks (useAuth, useTicketCheckout, etc.)
├── services/             # Blockchain, API client, and offline vault services
└── lib/                  # Utilities, CRDT sync, and crypto helpers
```

---

## 8. Troubleshooting Common Issues

### Metro Cache Issues
If you encounter module resolution errors or stale code:
```bash
npx expo start --clear
# or
pnpm start -- --reset-cache
```

### Port Conflicts
If port 8081 is already in use by another process:
```bash
pnpm start -- --port 8082
```

### Network Request Failures on Physical Device
1. Confirm your computer and phone are connected to the exact same Wi-Fi network.
2. Check firewall settings to ensure incoming connections on your backend port (e.g. 8080) are permitted.
3. Test your computer's LAN IP in your phone's browser: `http://<YOUR_IP>:8080/health`.

---

## 9. Submitting Pull Requests

1. Create a descriptive feature/fix branch off `main`:
   ```bash
   git checkout -b feat/my-mobile-feature
   ```
2. Verify that type checking, linting, and unit tests pass:
   ```bash
   pnpm typecheck
   pnpm lint
   pnpm test
   ```
3. Commit with concise, descriptive commit messages following the Conventional Commits specification.
4. Open a pull request against `Agora-Events/agora`.
