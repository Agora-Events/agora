# Authentication Guide

This guide provides a comprehensive overview of how authentication is implemented and managed within the Agora platform. Agora uses a centralized authentication system based on JSON Web Tokens (JWT) stored in secure, HttpOnly cookies.

## Overview of Auth Strategy

Agora utilizes a stateless authentication strategy using **JWT (JSON Web Tokens)**.

- **Cookie Name**: `auth_token`
- **Storage**: HttpOnly, Secure (in production), SameSite: Lax.
- **Expiration**: 7 days.
- **Signing**: Tokens are signed using a server-side `JWT_SECRET`.

When a user authenticates via any of the supported flows, a JWT is generated containing the user's identification (email and/or subject ID) and set as a cookie. Subsequent requests to the frontend API routes or the backend (via proxy) use this cookie to validate the user's session.

---

## Authentication Flows

### 1. Email Authentication Flow

The email flow is currently implemented as a passwordless login system.

**Step-by-step:**
1. **Initiation**: The user enters their email address on the `/auth` page.
2. **Request**: The frontend sends a `POST` request to `/api/auth/email` with the email in the request body.
3. **Validation**: The API validates the email format.
4. **Token Generation**: A JWT is signed containing the email address.
5. **Cookie Injection**: The JWT is set as an `HttpOnly` cookie named `auth_token`.
6. **Completion**: The API returns a success response, and the frontend redirects the user to the `/home` page.

> [!NOTE]
> This flow currently serves as a foundation for a future "Magic Link" implementation. In the current development state, it provides immediate access upon entering a valid email.

---

### 2. Google OAuth Flow

Agora supports authentication via Google OAuth 2.0.

**Step-by-step:**
1. **Initiation**: The user clicks the "Sign in with Google" button.
2. **Redirect to Provider**: The application redirects the user to Google's OAuth consent screen (via `/api/auth/google`).
3. **User Authorization**: The user grants permission to Agora.
4. **Callback**: Google redirects the user back to Agora at `/api/auth/google?code=...`.
5. **Code Exchange**: The server-side API route exchanges the `code` for an `id_token` using the Google OAuth API.
6. **User Identification**: The `id_token` is decoded to extract the user's email and unique subject ID (`sub`).
7. **Session Creation**: A JWT is signed for the Agora session and set as the `auth_token` cookie.
8. **Redirection**: The user is redirected to the `/home` page.

---

### 3. Apple OAuth Flow

The Apple OAuth flow follows the "Sign in with Apple" protocol.

**Step-by-step:**
1. **Initiation**: The user clicks the "Sign in with Apple" button.
2. **Redirect to Provider**: The application redirects the user to Apple's authorization server.
3. **User Authorization**: The user authorizes the request (potentially using FaceID/TouchID).
4. **Callback (GET/POST)**: Apple redirects back to `/api/auth/apple`.
    - If `response_mode` is `query`, it's a `GET` request.
    - If `response_mode` is `form_post` (used when requesting scopes like email/name), it's a `POST` request.
5. **Code Exchange**: Agora exchanges the authorization `code` for an `id_token` from Apple.
6. **User Identification**: The `id_token` is decoded. If the email is missing (common in subsequent logins), the subject ID (`sub`) is used to identify the user.
7. **Session Creation**: The session JWT is generated and set as the `auth_token` cookie.
8. **Redirection**: The user is redirected to the `/home` page.

---

## Session Storage and Validation

### Validation Logic
Session validation is performed on the server-side by checking the presence and validity of the `auth_token` cookie.

- **Frontend (Next.js)**: The utility function `getAuthFromRequest` in `apps/web/lib/auth.ts` extracts and verifies the JWT from the request cookies.
- **Token Payload**:
  ```typescript
  {
    "email": "user@example.com",
    "sub": "unique-provider-id",
    "iat": 1234567890,
    "exp": 1234567890
  }
  ```

---

## Local Testing

To test authentication locally, ensure you have the following environment variables configured in your `apps/web/.env.local` (or equivalent):

### Required Environment Variables
```text
JWT_SECRET=your_jwt_secret_here
GOOGLE_CLIENT_ID=your_google_client_id
GOOGLE_CLIENT_SECRET=your_google_client_secret
GOOGLE_REDIRECT_URI=http://localhost:3000/api/auth/google
APPLE_CLIENT_ID=your_apple_client_id
APPLE_CLIENT_SECRET=your_apple_client_secret
APPLE_REDIRECT_URI=http://localhost:3000/api/auth/apple
```

### Testing the Email Flow
1. Start the development server: `pnpm dev`.
2. Navigate to `http://localhost:3000/auth`.
3. Enter any valid email (e.g., `test@example.com`).
4. You should be redirected to `/home` and see the `auth_token` cookie set in your browser's dev tools.

### Mocking OAuth
For local development without real OAuth credentials:
- You can manually set an `auth_token` cookie with a valid JWT signed by your local `JWT_SECRET`.
- Alternatively, use the Email flow which provides a similar session token.

---

## Common Auth Errors

| Error Message | Cause | Resolution |
| :--- | :--- | :--- |
| `Invalid email format` | The provided email does not match the standard email regex. | Enter a valid email address. |
| `Missing code` | The OAuth callback was triggered without an authorization code. | Ensure the OAuth flow is initiated from the official buttons. |
| `Google/Apple OAuth failed` | The provider rejected the code exchange request. | Check your Client ID, Secret, and Redirect URI configuration. |
| `Invalid ID token` | The token returned by the provider could not be decoded or verified. | Ensure the provider's public keys are reachable or the token hasn't expired. |
| `Internal server error` | An unexpected error occurred during JWT signing or cookie setting. | Check server logs for stack traces and environment variable status. |
