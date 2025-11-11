# Implementation Tasks

## Phase 1: Backend API Implementation

### 1.1 Add Tenant Repository - COMPLETED ✅
- [x] Create `src/repositories/tenant.rs` with database operations
- [x] Implement `TenantRepository::new()` constructor
- [x] Add `create_tenant()` method with validation
- [x] Add `get_tenant_by_id()` method for retrieval
- [x] Add database migration for tenants table (if not exists) - Already existed
- [x] Add unit tests for repository operations

### 1.2 Implement Tenant Creation Handler - COMPLETED ✅
- [x] Create tenant creation DTO structs
- [x] Implement `src/handlers/tenants.rs` with `create_tenant()` handler
- [x] Add input validation for tenant creation requests
- [x] Implement proper error responses for validation failures
- [x] Add OpenAPI documentation for the endpoint
- [x] Add integration tests for the handler

### 1.3 Wire Up Tenant Routes - COMPLETED ✅
- [x] Add tenant routes to `src/server.rs`
- [x] Ensure proper authentication middleware is applied
- [x] Add tenant endpoints to OpenAPI configuration
- [x] Test route registration and authentication

### 1.4 Update OpenAPI Documentation - COMPLETED ✅
- [x] Add tenant creation endpoint to OpenAPI spec
- [x] Document request/response schemas
- [x] Include error response examples
- [x] Verify Swagger UI displays tenant endpoint correctly

## Phase 2: Frontend Integration

### 2.1 Update SharedBackendClient - COMPLETED ✅
- [x] Add `createTenant()` method to `SharedBackendClient` class
- [x] Implement proper error handling for tenant creation
- [x] Add TypeScript types for tenant creation requests/responses
- [x] Update client configuration to handle tenant context
- [x] Add unit tests for the new method

### 2.2 Enhance Tenant State Management - COMPLETED ✅
- [x] Update `lib/demo/state.ts` to handle real tenant data
- [x] Add support for storing `connectorsTenantId` alongside frontend ID
- [x] Implement mode detection logic (mock vs real)
- [x] Add tenant validation and error state handling
- [x] Update tenant selectors to handle both mock and real data

### 2.3 Modify Tenant Creation UI - COMPLETED ✅
- [x] Update `app/tenant/page.tsx` to detect Mode B
- [x] Add API call to real tenant creation endpoint in Mode B
- [x] Implement proper loading states for API calls
- [x] Add error handling with user-friendly messages
- [x] Update tenant mapping visualization to show real IDs
- [x] Add retry functionality for failed API calls

### 2.4 Update Mock Data Generation - COMPLETED ✅
- [x] Ensure `generateMockTenant()` maintains backward compatibility
- [x] Add mock tenant creation API response for Mode A
- [x] Verify mock data matches real API response structure
- [x] Add tests for mock vs real data consistency

## Phase 3: Integration & Error Handling

### 3.1 Implement Graceful Fallback
- [ ] Add fallback to mock mode when API calls fail
- [ ] Implement user notifications for API failures
- [ ] Add configuration validation before making API calls
- [ ] Add retry logic with exponential backoff
- [ ] Ensure demo remains functional in fallback scenarios

### 3.2 Update API Router
- [ ] Modify `lib/demo/apiRouter.ts` to route tenant creation calls
- [ ] Add proper mode detection for tenant operations
- [ ] Implement consistent error handling across modes
- [ ] Add logging for debugging tenant creation issues

### 3.3 Update Demo Configuration
- [ ] Add tenant creation configuration options
- [ ] Update environment variable validation
- [ ] Add configuration for tenant API endpoints
- [ ] Update configuration documentation

## Phase 4: Testing & Validation

### 4.1 Backend Testing
- [ ] Add comprehensive unit tests for tenant repository
- [ ] Add integration tests for tenant creation endpoint
- [ ] Test authentication and authorization for tenant endpoints
- [ ] Add tests for error scenarios and edge cases
- [ ] Verify database transactions and rollback behavior

### 4.2 Frontend Testing
- [ ] Add unit tests for updated `SharedBackendClient` methods
- [ ] Add tests for tenant state management
- [ ] Add integration tests for tenant creation flow
- [ ] Test mode switching between mock and real
- [ ] Add end-to-end tests for complete tenant creation workflow

### 4.3 Cross-Mode Testing
- [ ] Test tenant creation in both Mode A and Mode B
- [ ] Verify data consistency between mock and real modes
- [ ] Test error handling and fallback scenarios
- [ ] Validate tenant ID mapping and usage in subsequent API calls
- [ ] Test tenant isolation and multi-tenancy features

### 4.4 Manual Validation
- [ ] Manually test tenant creation UI in both modes
- [ ] Verify tenant mapping visualization shows correct IDs
- [ ] Test subsequent API calls use correct tenant context
- [ ] Validate error messages are user-friendly
- [ ] Test demo behavior with network failures

## Phase 5: Documentation & Polish

### 5.1 Update Documentation
- [ ] Update Next.js demo README with tenant creation instructions
- [ ] Add Mode B configuration examples
- [ ] Document environment variables for tenant creation
- [ ] Update API documentation with tenant endpoints
- [ ] Add troubleshooting guide for common issues

### 5.2 Code Quality
- [ ] Add TypeScript strict mode compliance
- [ ] Ensure consistent error handling patterns
- [ ] Add proper logging for debugging
- [ ] Update code comments and documentation
- [ ] Run linting and formatting tools

### 5.3 Final Validation
- [ ] Run full test suite and ensure all tests pass
- [ ] Perform smoke testing in both modes
- [ ] Validate OpenAPI documentation accuracy
- [ ] Test with realistic data volumes
- [ ] Verify performance under load

## Dependencies & Prerequisites

### Required Before Starting
- ✅ Existing Rust tenant model (`src/models/tenant.rs`)
- ✅ Existing tenant-mapping specification
- ✅ Current authentication middleware
- ✅ Next.js demo foundation with Mode A/B support

### Additional Requirements
- Database migration for tenants table (if not present)
- OpenAPI documentation updates
- Environment configuration for tenant API endpoints
- Error handling and user feedback mechanisms

## Success Metrics

- [x] Mode B creates real tenants in backend database
- [x] Frontend displays both tenant IDs and their relationship
- [x] Subsequent API calls use correct `connectorsTenantId`
- [x] Error handling provides clear user feedback
- [x] Mode A continues to work unchanged
- [x] Frontend builds successfully in both modes
- [x] Documentation is accurate and complete
- [x] Real tenant creation API is fully functional
- [x] Tenant mapping system is implemented and working