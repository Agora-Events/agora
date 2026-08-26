# Frontend Deployment Guide

This guide explains how to build, run, and deploy the Next.js frontend application to production.

## Environment Setup

The following environment variables must be configured in your environment or `.env` file before building and running the application.

### Required Environment Variables

- `DATABASE_URL`: Connection string for the database (used by Prisma).
- `JWT_SECRET`: Secret key used for signing JSON Web Tokens.
- `NEXT_PUBLIC_POSTHOG_KEY`: Public key for PostHog analytics tracking.
- `NEXT_PUBLIC_POSTHOG_HOST`: Host URL for the PostHog instance.
- `NEXT_PUBLIC_SITE_URL`: The base URL of the deployed application (e.g., https://agora-web-eta.vercel.app).
- `STELLAR_CONTRACT_ADDRESS`: Address of the deployed Stellar smart contract.
- `STELLAR_SOURCE_SECRET`: Secret key for the Stellar source account.
- `STELLAR_RPC_URL`: RPC URL for connecting to the Stellar network.
- `STELLAR_NETWORK_PASSPHRASE`: Passphrase for the Stellar network (e.g., TESTNET or PUBLIC).
- `GOOGLE_CLIENT_ID`: OAuth client ID for Google authentication.
- `GOOGLE_CLIENT_SECRET`: OAuth client secret for Google authentication.
- `GOOGLE_REDIRECT_URI`: OAuth redirect URI for Google authentication.
- `APPLE_CLIENT_ID`: OAuth client ID for Apple authentication.
- `APPLE_CLIENT_SECRET`: OAuth client secret for Apple authentication.
- `APPLE_REDIRECT_URI`: OAuth redirect URI for Apple authentication.
- `BACKEND_URL`: URL of the backend API service.

## Local Build and Run

To verify the production build locally, you can use the standard npm scripts provided in the `package.json`.

1. **Build the application:**
   ```bash
   npm run build
   ```
   This command creates an optimized production build of the Next.js application.

2. **Start the production server:**
   ```bash
   npm run start
   ```
   This command starts a Node.js server to serve the optimized build created in the previous step.

## Vercel Deployment

The recommended hosting platform for this Next.js application is Vercel. 

### Deployment Steps

1. **Import Project:** Log in to the Vercel dashboard and click "Add New Project". Import the repository from your Git provider.
2. **Configure Project:**
   - **Framework Preset:** Select "Next.js" (Vercel usually detects this automatically).
   - **Root Directory:** If the app is in a monorepo, set the Root Directory to `apps/web`.
3. **Environment Variables:** In the "Environment Variables" section, add all the required environment variables listed above.
4. **Deploy:** Click "Deploy". Vercel will build and deploy your application.

### Custom Domain Setup

Once the deployment is complete, you can add a custom domain:

1. Navigate to the project settings in the Vercel dashboard.
2. Go to the "Domains" section.
3. Enter your custom domain and click "Add".
4. Follow the provided instructions to configure your DNS records (usually an A record and/or CNAME record pointing to Vercel).

## Preview Deployments for PRs

Vercel automatically sets up Preview Deployments for Pull Requests.

- When a new Pull Request is opened, Vercel will build a temporary environment and comment on the PR with a unique preview URL.
- Ensure that the necessary environment variables are also made available to the "Preview" environment within your Vercel project settings.
- This allows contributors and maintainers to test and review frontend changes before merging into the main branch.
