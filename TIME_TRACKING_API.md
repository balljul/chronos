# Time Tracking API Implementation

## Overview

This document describes the implementation of CRUD backend endpoints for the time tracking application. The implementation follows the existing codebase patterns and provides comprehensive time tracking functionality similar to Toggl or Harvest.

## Database Schema

### Tables Created

1. **projects** - User's projects for organizing time entries
2. **tasks** - Optional tasks within projects  
3. **time_entries** - Main time tracking entries

### Key Features

- UUID primary keys for all entities
- Proper foreign key relationships with cascade deletes
- Automatic timestamp management with triggers
- Duration calculation via database triggers
- Unique constraint preventing multiple running timers per user
- Comprehensive indexing for performance

## API Endpoints

### Base Path: `/api/time-entries`

All endpoints require JWT authentication via the `Authorization: Bearer <token>` header.

#### 1. Create Time Entry
- **POST** `/api/time-entries/`
- **Body**: `CreateTimeEntryRequest`
- **Response**: `TimeEntryResponse` (201 Created)
- Supports both completed entries and starting timers

#### 2. Start Timer
- **POST** `/api/time-entries/start`
- **Body**: `StartTimerRequest` (simplified create request)
- **Response**: `TimeEntryResponse` (201 Created)
- Automatically sets start time to current UTC time

#### 3. List Time Entries
- **GET** `/api/time-entries/`
- **Query Parameters**: `TimeEntryFilters`
- **Response**: `TimeEntriesListResponse` (200 OK)
- Supports pagination, filtering, and sorting

#### 4. Get Single Time Entry
- **GET** `/api/time-entries/{id}`
- **Response**: `TimeEntryResponse` (200 OK)
- **Error**: 404 if not found, 403 if belongs to another user

#### 5. Update Time Entry
- **PATCH** `/api/time-entries/{id}`
- **Body**: `UpdateTimeEntryRequest`
- **Response**: `TimeEntryResponse` (200 OK)
- Partial updates supported

#### 6. Delete Time Entry
- **DELETE** `/api/time-entries/{id}`
- **Response**: 204 No Content
- **Error**: 404 if not found, 403 if belongs to another user

#### 7. Stop Running Timer
- **PATCH** `/api/time-entries/{id}/stop`
- **Response**: `TimeEntryResponse` (200 OK)
- Sets end_time to current UTC time and calculates duration

#### 8. Get Current Running Timer
- **GET** `/api/time-entries/current`
- **Response**: `TimeEntryResponse` (200 OK)
- **Error**: 404 if no timer is running

## Request/Response Models

### CreateTimeEntryRequest
```json
{
  "description": "Optional description (max 1000 chars)",
  "project_id": "UUID or null",
  "task_id": "UUID or null", 
  "start_time": "ISO 8601 DateTime",
  "end_time": "ISO 8601 DateTime or null for running timer"
}
```

### UpdateTimeEntryRequest
```json
{
  "description": "Optional string",
  "project_id": "Optional UUID",
  "task_id": "Optional UUID",
  "start_time": "Optional DateTime",
  "end_time": "Optional DateTime"
}
```

### TimeEntryResponse
```json
{
  "id": "UUID",
  "description": "string or null",
  "project_id": "UUID or null",
  "task_id": "UUID or null", 
  "start_time": "ISO 8601 DateTime",
  "end_time": "ISO 8601 DateTime or null",
  "duration": "integer seconds or null",
  "is_running": "boolean",
  "created_at": "ISO 8601 DateTime",
  "updated_at": "ISO 8601 DateTime"
}
```

### TimeEntriesListResponse
```json
{
  "entries": "Array of TimeEntryResponse",
  "total_count": "integer",
  "total_duration": "integer seconds", 
  "page": "integer",
  "per_page": "integer"
}
```

### Filtering Options
- `start_date` / `end_date`: Date range filtering
- `project_id`: Filter by specific project
- `task_id`: Filter by specific task
- `is_running`: Filter running vs completed entries
- `page`: Page number (1-based, default 1)
- `limit`: Items per page (1-100, default 20)
- `sort_by`: "start_time" or "duration" (default start_time DESC)

## Error Handling

### HTTP Status Codes
- **200 OK**: Successful GET/PATCH requests
- **201 Created**: Successful POST requests
- **204 No Content**: Successful DELETE requests
- **400 Bad Request**: Validation errors, invalid time ranges
- **401 Unauthorized**: Missing or invalid JWT token
- **403 Forbidden**: Attempting to access another user's data
- **404 Not Found**: Resource not found or no running timer
- **409 Conflict**: Attempting to start timer when one is already running
- **500 Internal Server Error**: Database or unexpected errors

### Error Response Format
```json
{
  "error": "Human readable error message"
}
```

## Business Logic & Validation

### Time Entry Constraints
- End time must be after start time when both are provided
- Users cannot have multiple running timers simultaneously
- Descriptions limited to 1000 characters
- Duration automatically calculated by database triggers

### Authorization
- Users can only access their own time entries
- All endpoints verify ownership via user_id from JWT token
- Proper 403 Forbidden responses for unauthorized access

### Data Integrity
- Proper cascading deletes when users/projects/tasks are removed
- Database constraints prevent invalid time ranges
- Automatic timestamp management with updated_at triggers

## Architecture

The implementation follows the existing codebase patterns:

### Layers
1. **Models** (`src/app/models/`) - Data structures and validation
2. **Repositories** (`src/app/repositories/`) - Database access layer
3. **Services** (`src/app/services/`) - Business logic layer  
4. **Controllers** (`src/routes/`) - HTTP request/response handling

### Key Components
- **TimeEntry Model**: Core data structure with validation
- **TimeEntryRepository**: Database operations with complex filtering
- **TimeEntryService**: Business logic and validation orchestration
- **TimeEntriesController**: HTTP endpoint handlers with error handling

### Security Features
- JWT authentication middleware on all endpoints
- User isolation - users can only access their own data
- Input validation using the `validator` crate
- SQL injection protection via parameterized queries

## Testing

Comprehensive test coverage includes:

### Unit Tests
- Model validation and error handling
- Time calculation methods
- JSON serialization/deserialization
- Business logic validation

### Integration Tests  
- API request/response structure validation
- HTTP status code verification
- Error response format testing
- Endpoint path documentation

### Test Files
- `tests/time_entry_tests.rs` - Unit tests
- `tests/time_entry_integration_tests.rs` - Integration tests

## Usage Examples

### Start a Timer
```bash
curl -X POST http://localhost:8080/api/time-entries/start \
  -H "Authorization: Bearer your-jwt-token" \
  -H "Content-Type: application/json" \
  -d '{"description": "Working on new feature", "project_id": "project-uuid"}'
```

### List Time Entries with Filtering
```bash
curl "http://localhost:8080/api/time-entries/?page=1&limit=20&is_running=false&sort_by=duration" \
  -H "Authorization: Bearer your-jwt-token"
```

### Stop a Running Timer
```bash
curl -X PATCH http://localhost:8080/api/time-entries/{entry-id}/stop \
  -H "Authorization: Bearer your-jwt-token"
```

## Future Enhancements

Potential improvements for future iterations:

1. **Projects/Tasks API**: Full CRUD for projects and tasks
2. **Reporting**: Time summaries, reports by date/project
3. **Time Tracking Rules**: Minimum duration, automatic breaks
4. **Export**: CSV/JSON export of time data
5. **Webhooks**: Integration with external systems
6. **Mobile API**: Optimized endpoints for mobile apps

## Migration and Deployment

### Database Migrations
The implementation includes 4 migration files:
1. `20251103000001_create_projects_table.sql`
2. `20251103000002_create_tasks_table.sql` 
3. `20251103000003_create_time_entries_table.sql`
4. `20251103000004_create_time_entries_triggers.sql`

Migrations will run automatically on application startup via SQLx.

### Environment Requirements
- PostgreSQL database
- Rust environment with required dependencies
- JWT secret configuration
- Database URL configuration

The implementation is production-ready and follows security best practices.