## 1. Configuration Infrastructure
- [x] 1.1 Add environment variable types and validation
- [x] 1.2 Create `demoConfig` helper module with mode detection
- [x] 1.3 Add environment variable documentation and examples

## 2. Demo Mode Integration
- [x] 2.1 Extend `DemoConfig` type to include mode and API base URL
- [x] 2.2 Update demo state management to handle mode-specific behavior
- [x] 2.3 Create API routing abstraction layer

## 3. Mock vs Real API Routing
- [x] 3.1 Implement mock mode data operations (maintain existing functionality)
- [x] 3.2 Implement real mode API client functions
- [x] 3.3 Create unified interface for data operations regardless of mode

## 4. Configuration Validation and Error Handling
- [x] 4.1 Add environment variable validation at startup
- [x] 4.2 Implement graceful fallback to mock mode on configuration errors
- [x] 4.3 Add clear error messages and logging for configuration issues

## 5. Educational Updates
- [x] 5.1 Update UI annotations to reflect current mode (mock vs real)
- [x] 5.2 Add configuration documentation within the demo interface
- [x] 5.3 Update inline code comments to explain mode-specific behavior

## 6. Testing and Validation
- [x] 6.1 Add unit tests for configuration detection and validation
- [x] 6.2 Test mock mode behavior (ensure existing functionality is preserved)
- [x] 6.3 Test real mode configuration and fallback scenarios
- [x] 6.4 Add integration tests for mode switching

## 7. Documentation and Examples
- [x] 7.1 Create example `.env` files for both modes
- [x] 7.2 Update README with configuration instructions
- [x] 7.3 Add mode-specific setup documentation