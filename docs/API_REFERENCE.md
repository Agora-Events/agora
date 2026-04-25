# API Reference - Axum Backend

## Base URL

```
http://localhost:3000/api/v1
```

## Overview

This API follows RESTful conventions and returns JSON responses with a standardized format. All endpoints are prefixed with `/api/v1` and require specific headers for proper request tracking and authentication.

## Headers

### Mandatory Headers

| Header | Description | Example |
|--------|-------------|---------|
| `X-Request-ID` | Unique identifier for request tracking | `550e8400-e29b-41d4-a716-446655440000` |
| `Content-Type` | Media type of the request body | `application/json` |
| `Authorization` | Bearer token for authenticated endpoints | `Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...` |

### Optional Headers

| Header | Description | Example |
|--------|-------------|---------|
| `Accept` | Expected response format | `application/json` |

## Response Format

### Success Response

```json
{
  "success": true,
  "data": {
    // Response payload
  },
  "message": "Operation completed successfully"
}
```

### Empty Success Response

```json
{
  "success": true,
  "data": null,
  "message": "Operation completed successfully"
}
```

### Error Response

```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error description"
  }
}
```

## HTTP Status Codes

### 2xx Success Codes

| Code | Meaning | Description |
|------|---------|-------------|
| 200 | OK | Request completed successfully |
| 201 | Created | Resource created successfully |
| 204 | No Content | Request completed with no response body |

### 4xx Client Error Codes

| Code | Meaning | Description |
|------|---------|-------------|
| 400 | Bad Request | Invalid request data or validation failed |
| 401 | Unauthorized | Authentication required or failed |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | Resource does not exist |

### 5xx Server Error Codes

| Code | Meaning | Description |
|------|---------|-------------|
| 500 | Internal Server Error | Unexpected server error |
| 503 | Service Unavailable | External service or database unavailable |

## Endpoints

### Health Checks

#### GET /health

Combined health check for API and Database connectivity.

**Response:**
```json
{
  "success": true,
  "data": {
    "status": "ok",
    "timestamp": "2024-01-15T10:30:00Z"
  },
  "message": "API is healthy"
}
```

**Error Response (503):**
```json
{
  "success": false,
  "error": {
    "code": "EXTERNAL_SERVICE_ERROR",
    "message": "API is not ready: database is unreachable (connection timeout)"
  }
}
```

#### GET /health/db

Database connectivity check.

**Response:**
```json
{
  "success": true,
  "data": {
    "status": "ok",
    "database": "connected",
    "timestamp": "2024-01-15T10:30:00Z"
  },
  "message": "Database is healthy"
}
```

#### GET /health/blockchain

Soroban RPC connectivity check.

**Response:**
```json
{
  "success": true,
  "data": {
    "status": "ok",
    "blockchain": "soroban",
    "soroban_rpc": "https://soroban-testnet.stellar.org",
    "timestamp": "2024-01-15T10:30:00Z"
  },
  "message": "Soroban RPC is reachable"
}
```

**Error Response (503):**
```json
{
  "success": false,
  "error": {
    "code": "EXTERNAL_SERVICE_ERROR",
    "message": "Soroban RPC health check failed"
  }
}
```

#### GET /health/ready

Readiness check for both API and Database.

**Response:**
```json
{
  "success": true,
  "data": {
    "status": "ready",
    "api": "ok",
    "database": "ok"
  },
  "message": "Service is ready"
}
```

### Examples

#### GET /examples/validation-error

Returns a validation error for testing error handling.

**Response (400):**
```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "The provided input is invalid"
  }
}
```

#### GET /examples/empty-success

Returns an empty success response for testing success handling.

**Response:**
```json
{
  "success": true,
  "data": null,
  "message": "Operation completed successfully"
}
```

#### GET /examples/not-found/:id

Returns a not found error for testing 404 handling.

**Path Parameters:**
- `id` (string): Resource identifier

**Response (404):**
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Resource with id '123' was not found"
  }
}
```

## Error Codes Reference

| Error Code | HTTP Status | Description |
|------------|-------------|-------------|
| `VALIDATION_ERROR` | 400 | Request data failed validation |
| `AUTH_ERROR` | 401 | Authentication failed or missing |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `DATABASE_ERROR` | 500 | Database operation failed |
| `EXTERNAL_SERVICE_ERROR` | 503 | External service unavailable |
| `INTERNAL_SERVER_ERROR` | 500 | Unexpected server error |

## Request Examples

### cURL Examples

#### Health Check
```bash
curl -X GET \
  http://localhost:3000/api/v1/health \
  -H 'X-Request-ID: 550e8400-e29b-41d4-a716-446655440000' \
  -H 'Accept: application/json'
```

#### Validation Error Example
```bash
curl -X GET \
  http://localhost:3000/api/v1/examples/validation-error \
  -H 'X-Request-ID: 550e8400-e29b-41d4-a716-446655440000' \
  -H 'Accept: application/json'
```

#### Not Found Example
```bash
curl -X GET \
  http://localhost:3000/api/v1/examples/not-found/123 \
  -H 'X-Request-ID: 550e8400-e29b-41d4-a716-446655440000' \
  -H 'Accept: application/json'
```

## Rate Limiting

Currently, no rate limiting is implemented. All endpoints are available without rate restrictions.

## Authentication

Authentication is not yet implemented for the current endpoints. Future endpoints will require Bearer token authentication via the `Authorization` header.

## Versioning

The API is versioned using URL path versioning. The current version is `v1`. All endpoints are prefixed with `/api/v1`.

## Future Endpoints

The following endpoints are planned for future implementation:

### Authentication & Users
- `POST /auth/register` - User registration
- `POST /auth/login` - User login
- `POST /auth/logout` - User logout
- `GET /users/profile` - Get user profile
- `PUT /users/profile` - Update user profile

### Events
- `GET /events` - List events
- `POST /events` - Create event
- `GET /events/:id` - Get event details
- `PUT /events/:id` - Update event
- `DELETE /events/:id` - Delete event

## Development Notes

- All timestamps are returned in ISO 8601 format (UTC)
- The `X-Request-ID` header is propagated through the system for distributed tracing
- Error responses are standardized across all endpoints
- The API uses PostgreSQL as the primary database
- CORS is configured for cross-origin requests
