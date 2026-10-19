# Mobile app (Expo)

## Prerequisites

- `Node.js` 18+ and `pnpm`
- `Expo Go` app (for testing on a real phone) **or** `Xcode` (for iOS simulator) / `Android Studio` (for Android emulator)

## Install and Run

From the repo root:

```bash
pnpm install
cd apps/mobile
pnpm install
```

## Environment Setup

Create a `.env` file in `apps/mobile` with your backend API URL:

```bash
cd apps/mobile
echo "EXPO_PUBLIC_API_URL=http://localhost:8080" > .env
```

The mobile app defaults to `http://localhost:8080` if no environment variable is set. Point this to your local server (e.g., `http://10.0.2.2:3001` for Android emulator, `http://localhost:3001` for iOS simulator).

## Start the App

From `apps/mobile`:

```bash
pnpm start
```

This opens the Metro bundler. To run on:

- **iOS Simulator**: Press `i` or select "Run on iOS Simulator"
- **Android Emulator**: Press `a` or select "Run on Android Emulator"
- **Expo Go**: Scan the QR code with the Expo Go app on your phone

## Pointing to Your Local Server

When running the Expo app:

- **iOS Simulator**: Use `http://localhost:3001` (or your backend port)
- **Android Emulator**: Use `http://10.0.2.2:3001` (special alias for host loopback)
- **Physical device with Expo Go**: Use your computer's local IP address (e.g., `http://192.168.1.100:3001`) - make sure your phone and computer are on the same WiFi network

## Running Tests and Lint

```bash
cd apps/mobile
pnpm test        # Run Jest tests
pnpm lint        # Run ESLint
pnpm typecheck   # Run TypeScript type checking
```

## Common Errors and Fixes

### Metro cache issues
If you see strange bundling behavior or errors about modules not found:

```bash
pnpm start -- -c
# or
npx expo start --clear
```

### Port already in use
If port `19000` or `19001` is already in use:

```bash
pnpm start -- --port 19002
```

### Metro doesn't find the app
If Metro can't find the entry point:

```bash
pnpm start -- --reset-cache
```