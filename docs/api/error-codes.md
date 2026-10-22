# API Error Codes

This document provides a centralized reference for all machine-readable API error codes used by the Agora backend.

## Overview

Every error response follows the same JSON shape:

```json
{
  "code": "VALIDATION_FAILED",
  "message": "Human-readable error message",
  "request_id": "b3b3f9b0-1c1e-4c9a-9c1b-2f6a7e9d0c1a"
}
```

- **`code`**: A stable, machine-readable error code in `SCREAMING_SNAKE_CASE`
- **`message`**: A human-readable message safe to display to end users
- **`request_id`**: Optional correlation ID that matches the `x-request-id` response header

The HTTP status code is carried in the response header, not in the JSON body.

---

## Error Code Reference

### VALIDATION_FAILED

- **HTTP Status**: `400 Bad Request` or `422 Unprocessable Entity`
- **Description**: Request body or query parameters failed validation
- **Common causes**:
  - Missing required fields
  - Fields exceeding maximum length
  - Invalid format (e.g., invalid UUID, malformed URL)
  - Invalid enum values

#### Example

```http
POST /api/v1/profile HTTP/1.1
Content-Type: application/json

{
  "displayName": "A very long display name that exceeds fifty characters and should be rejected by the server's validation logic"
}
```

```http
HTTP/1.1 400 Bad Request
Content-Type: application/json

{
  "code": "VALIDATION_FAILED",
  "message": "displayName must not exceed 50 characters",
  "request_id": "b3b3f9b0-1c1e-4c9a-9c1b-2f6a7e9d0c1a"
}
```

#### Remediation

- Review the request body against the API specification
- Ensure all required fields are present
- Check field length constraints
- Validate input format (e.g., UUIDs, URLs, email addresses)

---

### UNAUTHORIZED

- **HTTP Status**: `401 Unauthorized`
- **Description**: Missing or invalid authentication credentials
- **Common causes**:
  - Missing `Authorization` header
  - Invalid JWT token
  - Expired token
  - Malformed token (wrong scheme, invalid signature)

#### Example

```http
GET /api/v1/profile HTTP/1.1
```

```http
HTTP/1.1 401 Unauthorized
Content-Type: application/json

{
  "code": "UNAUTHORIZED",
  "message": "Missing Authorization header",
  "request_id": "b3b3f9b0-1c1e-4c9a-9c1b-2f6a7e9d0c1a"
}
```

#### Remediation

- Include a valid `Authorization: Bearer <jwt>` header
- Ensure the JWT is properly signed with the correct secret
- Refresh expired tokens using your authentication flow
- Verify the token format follows the expected pattern

---

### FORBIDDEN

- **HTTP Status**: `403 Forbidden`
- **Description**: Authenticated caller is not authorized to perform this action
- **Common causes**:
  - Accessing another organizer's private resources
  - Performing admin-only actions without admin privileges
  - Attempting to modify resources outside your scope

#### Example

```http
DELETE /api/v1/events/550e8400-e29b-41d4-a716-446655440000 HTTP/1.1
Authorization: Bearer <token_for_different_organizer>
```

```http
HTTP/1.1 403 Forbidden
Content-Type: application/json

{
  "code": "FORBIDDEN",
  "message": "You are not authorized to delete this event",
  "request_id": "b3b3f9b0-1c1e-4c9a-9c1b-2f6a7e9d0c1a"
}
```

#### Remediation

- Verify you are authenticated as the correct organizer
- Check that the resource belongs to your organization
- Review your permission scope in the application

---

### NOT_FOUND

- **HTTP Status**: `404 Not Found`
- **Description**: The requested resource does not exist
- **Common causes**:
  - Invalid event ID
  - Invalid organizer address
  - Resource was deleted
  - Typo in endpoint path

#### Example

```http
GET /api/v1/events/550e8400-e29b-41d4-a716-446655440000 HTTP/1.1
```

```http
HTTP/1.1 404 Not Found
Content-Type: application/json

{
  "code": "NOT_FOUND",
  "message": "Event with id '550e8400-e29b-41d4-a716-446655440000' was not found",
  "request_id": "b3b3f9b0-1c1e-4c9a-9c1b-2f6a7e9d0c1a"
}
```

#### Remediation

- Verify the resource ID is correct
- Check that the resource still exists
- Use list endpoints to discover valid resource IDs

---

### CONFLICT

- **HTTP Status**: `409 Conflict`
- **Description**: The request conflicts with the current resource state
- **Common causes**:
  - Duplicate resource creation (unique constraint violation)
  - Sold-out event (no inventory remaining)
  - Profile deletion with active events
  - Database foreign key constraint violation

#### Example

```http
POST /api/v1/events HTTP/1.1
Content-Type: application/json

{
  "title": "Duplicate Event Title",
  "location": "Same Location",
  "start_time": "2024-01-01T00:00:00Z"
}
```

```http
HTTP/1.1 409 Conflict
Content-Type: application/json

{
  "code": "CONFLICT",
  "message": "A resource with this identifier already exists",
  "request_id": "b3b3f9b0-1c1e-4c9a-9c1b-2f6a7e9d0c1a"
}
```

#### Remediation

- For unique constraints: check if the resource already exists before creating
- For sold-out events: wait for additional inventory or choose a different event
- For profile deletion: cancel or end all active events before deleting
- For foreign key violations: ensure referenced resources exist first

---

### RATE_LIMITED

- **HTTP Status**: `429 Too Many Requests`
- **Description**: The caller has exceeded the allowed request rate
- **Common causes**:
  - Exceeding API request quota
  - Too many authentication attempts
  - Requesting challenges too frequently

#### Example

```http
POST /api/v1/waiting-room/challenge HTTP/1.1
Content-Type: application/json

{
  "event_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

```http
HTTP/1.1 429 Too Many Requests
Content-Type: application/json
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1704067200

{
  "code": "RATE_LIMITED",
  "message": "Rate limit exceeded. Try again in 60 seconds.",
  "request_id": "b3b3f9b0-1c1e-4c9a-9c1b-2f6a7e9d0c1a"
}
```

#### Remediation

- Implement exponential backoff in your client
- Check `Retry-After` header for when to retry
- Reduce request frequency
- Consider caching responses where appropriate

---

### INTERNAL_ERROR

- **HTTP Status**: `500 Internal Server Error`
- **Description**: An unexpected internal failure occurred
- **Common causes**:
  - Unhandled exceptions
  - Logic errors
  - Unexpected data states
  - Unhandled error paths

#### Example

```http
HTTP/1.1 500 Internal Server Error
Content-Type: application/json

{
  "code": "INTERNAL_ERROR",
  "message": "An unexpected error occurred",
  "request_id": "b3b3f9b0-1c1e-4c9a-9c1b-2f6a7e9d0c1a"
}
```

#### Remediation

- This indicates a server-side issue
- Include the `request_id` when reporting the issue
- Check server logs for the correlated error
- Retry the request after a brief delay

---

### SERVICE_UNAVAILABLE

- **HTTP Status**: `503 Service Unavailable`
- **Description**: A required downstream service is temporarily unavailable
- **Common causes**:
  - Database connection failure
  - Redis connection failure
  - External service timeout
  - Database connection pool exhaustion

#### Example

```http
HTTP/1.1 503 Service Unavailable
Content-Type: application/json

{
  "code": "SERVICE_UNAVAILABLE",
  "message": "Database service is temporarily unavailable",
  "request_id": "b3b3f9b0-1c1e-4c9a-9c1b-2f6a7e9d0c1a"
}
```

#### Remediation

- Implement retry logic with exponential backoff
- Check `Retry-After` header if provided
- Monitor your connection to the API
- This is typically transient - retry after 1-5 seconds

---

## HTTP Status Code Mapping

| HTTP Status | Error Code | Description |
|-------------|------------|-------------|
| `400` | `VALIDATION_FAILED` | Bad request - validation error |
| `401` | `UNAUTHORIZED` | Missing or invalid authentication |
| `403` | `FORBIDDEN` | Not authorized for this action |
| `404` | `NOT_FOUND` | Resource not found |
| `409` | `CONFLICT` | Request conflicts with current state |
| `422` | `VALIDATION_FAILED` | Unprocessable entity - validation error |
| `429` | `RATE_LIMITED` | Too many requests |
| `500` | `INTERNAL_ERROR` | Internal server error |
| `502` | `SERVICE_UNAVAILABLE` | Bad gateway |
| `503` | `SERVICE_UNAVAILABLE` | Service unavailable |
| `504` | `SERVICE_UNAVAILABLE` | Gateway timeout |

---

## Database Error Handling

Database errors are mapped to the above codes based on type:

| Database Error | HTTP Status | Error Code |
|----------------|-------------|------------|
| Connection pool timeout | `503` | `SERVICE_UNAVAILABLE` |
| Connection closed | `503` | `SERVICE_UNAVAILABLE` |
| Unique constraint violation | `409` | `CONFLICT` |
| Foreign key violation | `409` | `CONFLICT` |
| Query failure | `500` | `INTERNAL_ERROR` |

---

## Rate Limiting Headers

When rate limiting is applied, responses include these headers:

- `X-RateLimit-Limit`: Maximum requests allowed in the window
- `X-RateLimit-Remaining`: Requests remaining in the current window
- `X-RateLimit-Reset`: Unix timestamp when the window resets
- `Retry-After`: Seconds to wait before retrying (when `429`)

---

## Request ID Correlation

Every error response includes a `request_id` field that matches the `x-request-id` response header:

```http
HTTP/1.1 404 Not Found
x-request-id: b3b3f9b0-1c1e-4c9a-9c1b-2f6a7e9d0c1a
Content-Type: application/json

{
  "code": "NOT_FOUND",
  "message": "Event not found",
  "request_id": "b3b3f9b0-1c1e-4c9a-9c1b-2f6a7e9d0c1a"
}
```

Use this ID when reporting issues to support or reviewing server logs.

---

## Best Practices for API Consumers

1. **Branch on `code`, not `message`**: The `code` field is stable; the `message` may change

   ```typescript
   // ✅ Good
   if (error.code === 'VALIDATION_FAILED') {
     // Handle validation error
   }
   
   // ❌ Bad
   if (error.message.includes('invalid')) {
     // fragile string matching
   }
   ```

2. **Implement retry logic**: For `SERVICE_UNAVAILABLE` and `RATE_LIMITED`, use exponential backoff

3. **Log `request_id`**: Include it in your error tracking for correlation

4. **Handle all error codes**: Implement a fallback for unexpected codes

5. **Don't expose internal details**: The `message` field is safe for users; never expose internal error details beyond what's provided

---

## Error Code Definition (Rust)

For implementation reference, here's the error code enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    ValidationFailed,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    RateLimited,
    InternalError,
    ServiceUnavailable,
}
```
