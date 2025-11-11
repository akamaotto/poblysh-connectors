# tenant-api Specification

## Purpose
Defines the tenant creation API endpoints and integration requirements for wiring the Next.js demo to the real Connectors service.

## ADDED Requirements

### Requirement: Tenant Creation Endpoint

The Connectors Service MUST provide a tenant creation endpoint for registering new tenants.

#### Scenarios:

1) #### Scenario: Successful tenant creation
   - When Poblysh Core sends a valid tenant creation request to `POST /api/v1/tenants`
   - And the request includes valid tenant data with required fields
   - Then the Connectors Service MUST:
     - Validate the request data according to the tenant schema
     - Create a new tenant record in the database with a generated UUID
     - Return a `201 Created` response with the complete tenant data
     - Include the generated `id` (Connectors tenant UUID) and any provided `name`
     - Set appropriate response headers including `Location` header pointing to the new tenant resource
   - And the response MUST match the tenant response schema exactly.

2) #### Scenario: Tenant creation with validation errors
   - When Poblysh Core sends a tenant creation request with invalid data
   - Such as missing required fields, invalid formats, or constraint violations
   - Then the Connectors Service MUST:
     - Reject the request with a `400 Bad Request` status code
     - Return a structured error response using the unified `ApiError` envelope
     - Use the `VALIDATION_FAILED` code with specific field-level validation details
     - Include all validation errors in the response details field
     - Not create any tenant record in the database
   - **Example validation errors:**
     - Missing `name` field returns `400 VALIDATION_FAILED` with field details
     - Invalid `name` format returns `400 VALIDATION_FAILED` with format requirements
     - Duplicate tenant identifier returns `409 CONFLICT` with conflict details.

3) #### Scenario: Tenant creation with authentication errors
   - When an unauthenticated request is made to `POST /api/v1/tenants`
   - Or when the request lacks proper operator authentication
   - Then the Connectors Service MUST:
     - Reject the request with a `401 Unauthorized` or `403 Forbidden` status code
     - Return a structured error response using the unified `ApiError` envelope
     - Use appropriate authentication error codes (`UNAUTHORIZED` or `FORBIDDEN`)
     - Include details about the authentication failure in the error details field
     - Not proceed with any tenant creation logic.

#### Authentication Requirements:
Tenant creation requires authentication using the existing Connectors service authentication patterns:

- **Required Headers**:
  - `Authorization: Bearer <operator_token>` - JWT token from Poblysh Core
  - `Content-Type: application/json` - Standard JSON content type

- **Authentication Context**:
  - Requests must include valid operator credentials from Poblysh Core
  - Tenant creation is a system operation and does not require tenant-scoped authentication
  - The operator must have sufficient permissions to create tenants in the system

- **Token Validation**:
  - JWT tokens are validated against the Poblysh Core public key
  - Token expiration and signature must be valid
  - Token must contain required operator permissions for tenant management

4) #### Scenario: Tenant creation with server errors
   - When the Connectors Service experiences internal errors during tenant creation
   - Such as database connectivity issues, service unavailability, or unexpected failures
   - Then the Connectors Service MUST:
     - Return a `500 Internal Server Error` status code
     - Use the `INTERNAL_SERVER_ERROR` code in the unified `ApiError` envelope
     - Include a trace ID for debugging purposes
     - Not leak sensitive internal error details to the client
     - Ensure no partial tenant data is persisted in case of failures.

---

### Requirement: Tenant Creation Request Schema

The tenant creation endpoint MUST accept a standardized request schema.

#### Request Schema:
```json
{
  "name": "string (required, max 255 characters)",
  "metadata": {
    "poblysh_tenant_id": "string (optional, UUID format)",
    "organization": "string (optional)",
    "created_by": "string (optional)",
    "environment": "string (optional, enum: local, test, staging, prod)"
  }
}
```

#### Validation Rules:
- `name`: Required string, maximum 255 characters, trimmed, cannot be empty or whitespace only
- `metadata.poblysh_tenant_id`: Optional UUID format for tracking Poblysh tenant association
- `metadata.organization`: Optional string for organization name
- `metadata.created_by`: Optional string identifying the creator
- `metadata.environment`: Optional enum value for environment context
- Request body must be valid JSON
- Additional metadata fields are allowed but ignored

#### Scenarios:

1) #### Scenario: Minimal valid request
   - When Poblysh Core sends a tenant creation request with `{"name": "Test Org"}`
   - Then the request MUST be accepted as valid
   - And the tenant MUST be created with the provided name
   - And all optional metadata fields MUST be null in the response
   - And the tenant name MUST be stored exactly as provided (after trimming whitespace).

2) #### Scenario: Invalid name handling
   - When Poblysh Core sends a tenant creation request with missing or invalid name
   - Such as `{"name": ""}`, `{"name": "   "}`, or missing name field entirely
   - Then the request MUST be rejected with `400 VALIDATION_FAILED`
   - And the error details MUST specify the name validation failure
   - And no tenant record MUST be created.

3) #### Scenario: Complete valid request with metadata
   - When Poblysh Core sends a request with all fields populated
   - And all data matches the required formats and constraints
   - Then the request MUST be accepted as valid
   - And the tenant MUST be created with all provided metadata stored
   - And the response MUST include all stored metadata fields.

4) #### Scenario: Invalid request formats
   - When Poblysh Core sends a request with invalid data formats
   - Such as non-JSON body, oversized fields, or invalid enum values
   - Then the request MUST be rejected with `400 VALIDATION_FAILED`
   - And the error details MUST specify the exact validation failures
   - **Examples:**
     - Name longer than 255 characters returns validation error
     - Invalid UUID format for `poblysh_tenant_id` returns validation error
     - Invalid enum value for `environment` returns validation error

5) #### Scenario: Duplicate tenant name handling
   - When Poblysh Core sends a request with a tenant name that already exists
   - And the system enforces unique tenant names
   - Then the request MUST be rejected with `409 CONFLICT`
   - And the error details MUST indicate the duplicate name conflict
   - And no tenant record MUST be created

6) #### Scenario: Concurrent tenant creation conflicts
   - When multiple concurrent requests attempt to create tenants with the same identifying data
   - Such as identical names or metadata that should be unique
   - Then only one request MUST succeed
   - And other requests MUST return `409 CONFLICT`
   - And the error response MUST indicate the source of the conflict

7) #### Scenario: Invalid metadata field validation
   - When Poblysh Core sends requests with invalid metadata
   - Such as oversized string fields, invalid UUIDs, or malformed data
   - Then the request MUST be rejected with `400 VALIDATION_FAILED`
   - And the error details MUST specify which metadata fields failed validation
   - **Examples:**
     - `organization` field longer than 1000 characters returns validation error
     - `created_by` field with invalid characters returns validation error
     - Invalid UUID format in `poblysh_tenant_id` returns validation error

---

### Requirement: Tenant Creation Response Schema

The tenant creation endpoint MUST return a standardized response schema on success.

#### Response Schema:
```json
{
  "data": {
    "id": "string (UUID, Connectors tenant identifier)",
    "name": "string (or null)",
    "created_at": "string (ISO 8601 datetime)",
    "updated_at": "string (ISO 8601 datetime)",
    "metadata": {
      "poblysh_tenant_id": "string (or null, UUID format)",
      "organization": "string (or null)",
      "created_by": "string (or null)",
      "environment": "string (or null)"
    }
  },
  "meta": {
    "request_id": "string (UUID)",
    "timestamp": "string (ISO 8601 datetime)"
  }
}
```

#### Response Headers:
- `Content-Type`: `application/json`
- `Location`: `/api/v1/tenants/{tenant_id}`
- `X-Trace-Id`: `string (UUID for request tracing)`

#### HTTP Status Code Reference:
- **201 Created**: Tenant successfully created
- **400 Bad Request**: Validation errors with `VALIDATION_FAILED` code
- **401 Unauthorized**: Missing or invalid authentication with `UNAUTHORIZED` code
- **403 Forbidden**: Insufficient permissions with `FORBIDDEN` code
- **409 Conflict**: Duplicate resource with `CONFLICT` code
- **500 Internal Server Error**: System errors with `INTERNAL_SERVER_ERROR` code

#### Scenarios:

1) #### Scenario: Successful creation response
   - When a tenant is successfully created
   - Then the response MUST have `201 Created` status code
   - And the response body MUST match the response schema exactly
   - And the `data.id` field MUST be the generated Connectors tenant UUID
   - And the `data.created_at` and `data.updated_at` MUST be set to the current timestamp
   - And the response headers MUST include the `Location` header pointing to the new resource.

2) #### Scenario: Response with metadata preservation
   - When a tenant is created with metadata in the request
   - Then the response MUST include all provided metadata fields
   - And the metadata values MUST match the request exactly
   - And any additional server-generated metadata MUST be clearly documented
   - And metadata field types MUST match the schema (string or null).

3) #### Scenario: Response consistency
   - When multiple tenant creation requests are made
   - Then all successful responses MUST follow the same schema structure
   - And all timestamp formats MUST be consistent ISO 8601
   - And all UUID formats MUST be consistent RFC 4122
   - And all field names and types MUST match the documented schema exactly.

---

### Requirement: Tenant API Integration for Next.js Demo

The Next.js demo MUST integrate with the tenant creation API when in Mode B (real API mode).

#### Integration Requirements:
- Detect demo mode and route tenant creation accordingly
- Handle API authentication using existing demo configuration
- Implement proper error handling and user feedback
- Store and display both frontend and backend tenant IDs
- Maintain backward compatibility with Mode A (mock mode)

#### Scenarios:

1) #### Scenario: Mode B tenant creation flow
   - When the Next.js demo is configured for Mode B (real API)
   - And a user submits the tenant creation form
   - Then the demo MUST:
     - Detect Mode B configuration and bypass mock tenant generation
     - Call the real Connectors tenant creation API endpoint with proper authentication
     - Handle API responses and errors appropriately using the `SharedBackendClient.createTenant()` method
     - Store the returned tenant data in the application state
     - Display both the frontend tenant context and the returned `connectorsTenantId`
   - And subsequent API calls MUST use the returned `connectorsTenantId` in the `X-Tenant-Id` header.

2) #### Scenario: Mode A backward compatibility
   - When the Next.js demo is configured for Mode A (mock mode)
   - And a user submits the tenant creation form
   - Then the demo MUST:
     - Use the existing mock tenant generation logic
     - Maintain all current mock data structures and behaviors
     - Not attempt any real API calls
     - Provide the same user experience as before
   - And no changes to Mode A behavior MUST be observable.

3) #### Scenario: API error handling in demo
   - When the tenant creation API call fails in Mode B
   - Due to network errors, validation failures, or server errors
   - Then the demo MUST:
     - Display clear, user-friendly error messages
     - Provide options to retry or fall back to mock mode
     - Maintain application state and prevent data corruption
     - Log errors appropriately for debugging
     - Allow the user to continue using the demo functionality
   - And the error handling MUST not break the demo experience.

4) #### Scenario: Tenant ID mapping display
   - When a tenant is successfully created (either mode)
   - Then the demo MUST display the tenant mapping information
   - Including both the frontend tenant identifier and the backend `connectorsTenantId`
   - And the display MUST clearly explain the relationship between the IDs
   - And the visualization MUST be consistent between Mode A and Mode B
   - And real API responses MUST show actual generated UUIDs instead of mock placeholders.

---

## MODIFIED Requirements

### Requirement: SharedBackendClient Enhancement

The `SharedBackendClient` class MUST be enhanced to support tenant creation operations.

#### Required Methods:
```typescript
interface SharedBackendClient {
  // Existing methods...

  /**
   * Creates a new tenant in the Connectors service
   *
   * @param tenant - Tenant data without system-generated fields
   * @returns Promise resolving to API response containing the created tenant
   */
  createTenant(tenant: {
    name: string;
    metadata?: {
      poblysh_tenant_id?: string;
      organization?: string;
      created_by?: string;
      environment?: "local" | "test" | "staging" | "prod";
    };
  }): Promise<DemoApiResponse<{
    id: string;           // Connectors tenant UUID
    name: string | null;  // Tenant name
    created_at: string;   // ISO 8601 timestamp
    updated_at: string;   // ISO 8601 timestamp
    metadata: {
      poblysh_tenant_id: string | null;
      organization: string | null;
      created_by: string | null;
      environment: string | null;
    };
  }>>;
}
```

#### Type Mapping Details:
The frontend `DemoApiResponse<T>` maps to the backend JSON response as follows:

- **Request Transformation**: The frontend `tenant` parameter maps directly to the backend request JSON
- **Response Transformation**: The backend JSON response body maps to the generic type parameter of `DemoApiResponse`
- **Field Mapping**: All field names use snake_case in the backend API and camelCase in the frontend interface
- **Error Handling**: The `DemoApiResponse` wrapper includes standard error fields that map to the `ApiError` envelope

#### Integration Requirements:
- Follow existing client patterns for authentication and error handling
- Support both mock and real API modes transparently
- Implement proper TypeScript typing for tenant operations
- Include request/response validation
- Handle rate limiting and retry logic consistently

#### Scenarios:

1) #### Scenario: Method implementation
   - When the `createTenant()` method is called
   - Then it MUST:
     - Accept tenant creation data matching the API schema
     - Apply existing authentication headers and configuration
     - Handle both Mode A (mock) and Mode B (real) appropriately
     - Return responses in the standard `DemoApiResponse<T>` format
     - Follow existing error handling patterns for consistency.

2) #### Scenario: Mode delegation
   - When `createTenant()` is called in Mode A
   - Then it MUST generate mock tenant data using existing mock generators
   - And when called in Mode B
   - Then it MUST make a real API call to the tenant creation endpoint
   - And the mode detection MUST be transparent to the calling code.

3) #### Scenario: Error handling consistency
   - When tenant creation encounters errors
   - Then the error handling MUST follow existing client patterns
   - And error types and messages MUST be consistent with other API operations
   - And retry logic MUST be applied appropriately based on error types
   - And rate limiting MUST be handled consistently with other operations.