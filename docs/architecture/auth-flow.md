# Authentication Flows

This document outlines the three primary authentication flows used in the Agora application.

## 1. Email / JWT Flow
Standard email and password authentication resulting in a stateless JWT cookie.

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant UI as Frontend
    participant API as /api/auth/email
    
    User->>UI: Enters Email & Password
    UI->>API: POST credentials
    API-->>UI: Return JWT cookie
    UI->>UI: Redirect to Dashboard / Home
```

## 2. Google OAuth Flow

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant UI as Frontend
    participant Google as Google Provider
    participant API as /api/auth/google/callback
    participant Session as Session Manager
    
    User->>UI: Clicks "Sign in with Google"
    UI->>Google: Redirect to Google Auth
    Google-->>API: Returns Auth Code / Token
    API->>Session: Create User Session
    Session-->>UI: Session Created
    UI->>UI: Redirect to Dashboard / Home
```

## 3. Apple OAuth Flow

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant UI as Frontend
    participant Apple as Apple Provider
    participant API as /api/auth/apple/callback
    participant Session as Session Manager
    
    User->>UI: Clicks "Sign in with Apple"
    UI->>Apple: Redirect to Apple Auth
    Apple-->>API: Returns Auth Code / Token
    API->>Session: Create User Session
    Session-->>UI: Session Created
    UI->>UI: Redirect to Dashboard / Home
```