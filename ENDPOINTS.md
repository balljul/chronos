# API Endpoints Documentation

This document describes all available API endpoints for the Chronos application.

## Authentication Endpoints

### Register User
- **URL**: `POST /api/auth/register`
- **Description**: Register a new user account
- **Request Body**:
  ```json
  {
    "name": "string (optional)",
    "email": "string (required, valid email)",
    "password": "string (required, strong password)"
  }
  ```
- **Response**: `201 Created`
  ```json
  {
    "message": "string",
    "user": {
      "id": "uuid",
      "name": "string|null",
      "email": "string",
      "created_at": "timestamp"
    }
  }
  ```
- **Error Responses**:
  - `400 Bad Request`: Validation errors
  - `409 Conflict`: Email already registered
  - `429 Too Many Requests`: Rate limit exceeded
  - `500 Internal Server Error`: Server error

### Login
- **URL**: `POST /api/auth/login`
- **Description**: Authenticate user and receive JWT tokens
- **Request Body**:
  ```json
  {
    "email": "string (required, valid email)",
    "password": "string (required)"
  }
  ```
- **Response**: `200 OK`
  ```json
  {
    "message": "string",
    "user": {
      "id": "uuid",
      "name": "string|null",
      "email": "string",
      "created_at": "timestamp"
    },
    "tokens": {
      "access_token": "string",
      "refresh_token": "string",
      "token_type": "Bearer",
      "expires_in": 900,
      "refresh_expires_in": 604800
    }
  }
  ```
- **Error Responses**:
  - `401 Unauthorized`: Invalid credentials
  - `423 Locked`: Account temporarily locked
  - `429 Too Many Requests`: Too many failed attempts
  - `500 Internal Server Error`: Server error

### Forgot Password
- **URL**: `POST /api/auth/forgot-password`
- **Description**: Request password reset token
- **Request Body**:
  ```json
  {
    "email": "string (required, valid email)"
  }
  ```
- **Response**: `200 OK`
  ```json
  {
    "message": "string"
  }
  ```
- **Error Responses**:
  - `400 Bad Request`: Validation errors
  - `429 Too Many Requests`: Rate limit exceeded
  - `500 Internal Server Error`: Server error

### Reset Password
- **URL**: `POST /api/auth/reset-password`
- **Description**: Reset password using token from email
- **Request Body**:
  ```json
  {
    "token": "string (required)",
    "password": "string (required, strong password)"
  }
  ```
- **Response**: `200 OK`
  ```json
  {
    "message": "string"
  }
  ```
- **Error Responses**:
  - `400 Bad Request`: Invalid/expired token or validation errors
  - `500 Internal Server Error`: Server error

### Refresh Token
- **URL**: `POST /api/auth/refresh`
- **Description**: Refresh access token using refresh token
- **Request Body**:
  ```json
  {
    "refresh_token": "string (required)"
  }
  ```
- **Response**: `200 OK`
  ```json
  {
    "access_token": "string",
    "refresh_token": "string",
    "token_type": "Bearer",
    "expires_in": 900,
    "refresh_expires_in": 604800
  }
  ```
- **Error Responses**:
  - `401 Unauthorized`: Invalid/expired refresh token
  - `429 Too Many Requests`: Rate limit exceeded
  - `500 Internal Server Error`: Server error

## Protected Endpoints (Require Authentication)

All protected endpoints require a valid JWT token in the Authorization header:
```
Authorization: Bearer <access_token>
```

### Logout
- **URL**: `POST /api/auth/logout`
- **Description**: Logout user and invalidate tokens
- **Headers**: `Authorization: Bearer <access_token>`
- **Request Body**:
  ```json
  {
    "refresh_token": "string (optional)",
    "logout_all_devices": "boolean (optional, default: false)"
  }
  ```
- **Response**: `200 OK`
  ```json
  {
    "message": "string",
    "logged_out_devices": "number|null"
  }
  ```
- **Error Responses**:
  - `401 Unauthorized`: Invalid token
  - `500 Internal Server Error`: Server error

### Get Profile
- **URL**: `GET /api/auth/profile`
- **Description**: Get current user's profile information
- **Headers**: `Authorization: Bearer <access_token>`
- **Response**: `200 OK`
  ```json
  {
    "id": "uuid",
    "name": "string|null",
    "email": "string",
    "created_at": "timestamp",
    "updated_at": "timestamp"
  }
  ```
- **Error Responses**:
  - `401 Unauthorized`: Invalid token
  - `404 Not Found`: User not found
  - `500 Internal Server Error`: Server error

### Update Profile
- **URL**: `PUT /api/auth/profile`
- **Description**: Update current user's profile information
- **Headers**: `Authorization: Bearer <access_token>`
- **Request Body**:
  ```json
  {
    "name": "string (optional)",
    "email": "string (optional, valid email)",
    "current_password": "string (required when changing email)"
  }
  ```
- **Response**: `200 OK`
  ```json
  {
    "id": "uuid",
    "name": "string|null",
    "email": "string",
    "created_at": "timestamp",
    "updated_at": "timestamp"
  }
  ```
- **Error Responses**:
  - `400 Bad Request`: Validation errors or missing current password
  - `401 Unauthorized`: Invalid token or incorrect current password
  - `404 Not Found`: User not found
  - `409 Conflict`: Email already in use
  - `500 Internal Server Error`: Server error

### Change Password
- **URL**: `POST /api/auth/change-password`
- **Description**: Change user's password
- **Headers**: `Authorization: Bearer <access_token>`
- **Request Body**:
  ```json
  {
    "current_password": "string (required)",
    "new_password": "string (required, strong password)"
  }
  ```
- **Response**: `200 OK`
  ```json
  {
    "message": "string"
  }
  ```
- **Error Responses**:
  - `400 Bad Request`: Validation errors or same password
  - `401 Unauthorized`: Invalid token or incorrect current password
  - `404 Not Found`: User not found
  - `500 Internal Server Error`: Server error

## User Management Endpoints

### List All Users
- **URL**: `GET /api/users`
- **Description**: Get list of all users
- **Response**: `200 OK`
  ```json
  [
    {
      "id": "uuid",
      "name": "string|null",
      "email": "string",
      "password_hash": "string",
      "created_at": "timestamp",
      "updated_at": "timestamp"
    }
  ]
  ```
- **Error Responses**:
  - `500 Internal Server Error`: Server error

### Get User by ID
- **URL**: `GET /api/users/{id}`
- **Description**: Get user information by UUID
- **Path Parameters**: `id` - User UUID
- **Response**: `200 OK`
  ```json
  {
    "id": "uuid",
    "name": "string|null",
    "email": "string",
    "password_hash": "string",
    "created_at": "timestamp",
    "updated_at": "timestamp"
  }
  ```
- **Error Responses**:
  - `404 Not Found`: User not found
  - `500 Internal Server Error`: Server error

### Get User by Email
- **URL**: `GET /api/users/email/{email}`
- **Description**: Get user information by email address
- **Path Parameters**: `email` - User email address
- **Response**: `200 OK`
  ```json
  {
    "id": "uuid",
    "name": "string|null",
    "email": "string",
    "password_hash": "string",
    "created_at": "timestamp",
    "updated_at": "timestamp"
  }
  ```
- **Error Responses**:
  - `404 Not Found`: User not found
  - `500 Internal Server Error`: Server error

## Password Requirements

Strong passwords must meet the following criteria:
- At least 8 characters long
- Contains at least one uppercase letter
- Contains at least one lowercase letter
- Contains at least one number
- Contains at least one special character

## Rate Limiting

The following endpoints have rate limiting applied:
- Registration: Limited per IP address
- Password reset: Limited per email address
- Token refresh: Limited per user
- Login attempts: Account lockout after multiple failed attempts

## Security Features

- JWT-based authentication with access and refresh tokens
- Token rotation on refresh
- Token blacklisting on logout
- Account lockout protection
- Rate limiting
- Input validation and sanitization
- Password hashing with Argon2
- Security event logging