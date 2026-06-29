# Frontend Environment Variables Reference

This document provides a reference for all environment variables used in the frontend application.

| Variable Name | Description | Required/Optional | Example Value | Where to Obtain |
|---------------|-------------|-------------------|---------------|-----------------|
| `BACKEND_URL` | The base URL for the backend API. | Optional | `http://localhost:3001` | Developer setup or deployment environment |
| `DATABASE_URL` | Database connection string for Prisma. | Required | `postgresql://user:password@localhost:5432/agora` | Database provider (e.g., Supabase, Heroku, local Postgres) |
| `NODE_ENV` | Defines the environment the application is running in (development, production, test). | Optional | `development` | Automatically set by framework or deployment platform |
| `JWT_SECRET` | Secret key used to sign JSON Web Tokens. | Optional | `super_secret_string` | Generated securely by developer |
| `GOOGLE_CLIENT_ID` | Client ID for Google OAuth. | Optional | `123456789-abc.apps.googleusercontent.com` | Google Cloud Console > Credentials |
| `GOOGLE_CLIENT_SECRET` | Client Secret for Google OAuth. | Optional | `GOCSPX-abcdefg123456` | Google Cloud Console > Credentials |
| `GOOGLE_REDIRECT_URI` | Redirect URI for Google OAuth callback. | Optional | `http://localhost:3000/api/auth/google` | Google Cloud Console > Credentials |
| `APPLE_CLIENT_ID` | Client ID for Apple OAuth. | Optional | `com.example.agora.web` | Apple Developer Portal > Identifiers |
| `APPLE_CLIENT_SECRET` | Client Secret for Apple OAuth. | Optional | `secret_string` | Apple Developer Portal |
| `APPLE_REDIRECT_URI` | Redirect URI for Apple OAuth callback. | Optional | `http://localhost:3000/api/auth/apple` | Apple Developer Portal |
| `STELLAR_CONTRACT_ADDRESS` | The address of the deployed Stellar smart contract. | Optional | `CA...XYZ` | Provided after deploying the Stellar contract |
| `STELLAR_SOURCE_SECRET` | Secret key for the Stellar source account. | Optional | `SA...XYZ` | Stellar Laboratory or account generation tool |
| `STELLAR_RPC_URL` | RPC URL for the Stellar network. | Optional | `https://soroban-testnet.stellar.org` | Stellar network provider |
| `STELLAR_NETWORK_PASSPHRASE` | Passphrase for the Stellar network being used. | Optional | `Test SDF Network ; September 2015` | Stellar network documentation |
| `NEXT_PUBLIC_POSTHOG_KEY` | Public key for PostHog analytics. | Optional | `phc_abc123` | PostHog dashboard |
| `NEXT_PUBLIC_POSTHOG_HOST` | Host URL for PostHog analytics. | Optional | `https://app.posthog.com` | PostHog dashboard |
| `NEXT_PUBLIC_SITE_URL` | The public URL of the frontend site. | Optional | `http://localhost:3000` | Deployment environment or local setup |

> Note: Variables marked as Optional often have default fallback values in the codebase for local development environments, but should be explicitly configured in production.
