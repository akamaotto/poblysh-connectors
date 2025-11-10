## MODIFIED Requirements

### Requirement: Mock UX Demo Application
The system SHALL provide a complete Next.js demo application that operates in either mock mode (Mode A) or real API mode (Mode B) while maintaining mock functionality as the default behavior.

#### Scenario: Complete mock Connectors integration flow (Mode A)
Given a developer runs the Next.js demo application with default configuration
When they navigate through the login → tenant → integrations → signals → grounding flow
Then they should experience a realistic, educational simulation of the Connectors integration story using only mock data
And all functionality should work without external dependencies or credentials

#### Scenario: Real API integration flow (Mode B)
Given a developer configures the demo with `NEXT_PUBLIC_DEMO_MODE=real` and `CONNECTORS_API_BASE_URL`
When they navigate through the flows
Then the application should route calls to the real Connectors API endpoints
And use actual authentication tokens and tenant IDs for API communication

#### Scenario: Mode configuration detection and routing
Given the demo application starts up
When it reads the `NEXT_PUBLIC_DEMO_MODE` environment variable
Then it should configure the demo to operate in either mock or real mode based on the configuration
And provide appropriate routing for data operations accordingly

#### Scenario: Deterministic mock data generation
Given a user creates a tenant and connects GitHub in mock mode
When they trigger a signal scan
Then the application should generate consistent mock signals across page reloads
And the signals should be realistic and properly associated with the tenant and connection

#### Scenario: Educational annotations and mapping
Given a user interacts with any major demo feature
When they view the UI elements
Then they should see clear explanations of what would happen in production
And the annotations should map mock behavior to real Connectors API concepts

## ADDED Requirements

### Requirement: Demo Mode Configuration System
The system SHALL provide a configuration system that supports environment variable-based mode switching between mock and real API behaviors.

#### Scenario: Environment variable configuration
Given a developer wants to configure the demo mode
When they set `NEXT_PUBLIC_DEMO_MODE` environment variable to "mock" (default) or "real"
Then the demo should read this configuration at startup and apply the appropriate mode
And maintain the current mock behavior as the default when no configuration is provided

#### Scenario: Environment variable validation
Given the demo application starts with configuration
When it validates the `NEXT_PUBLIC_DEMO_MODE` environment variable
Then it should accept only "mock" or "real" as valid values
And fallback to mock mode with a console warning for any invalid value
And log the invalid value that was provided for debugging purposes

#### Scenario: API base URL configuration
Given a developer configures the demo for real API mode
When they set the `CONNECTORS_API_BASE_URL` environment variable
Then the demo should validate that it's a properly formatted HTTPS URL
And use this URL for all real API calls when in real mode
And handle missing or invalid URL configurations gracefully

#### Scenario: API base URL validation
Given the demo is configured for real API mode
When it validates the `CONNECTORS_API_BASE_URL` environment variable
Then it should accept only valid HTTPS URLs (starting with "https://")
And fallback to mock mode with a console error if the URL is invalid or missing in real mode
And provide a clear error message explaining the URL format requirements

#### Scenario: Demo configuration helper
Given the demo application needs to route calls based on mode
When it uses the `demoConfig` helper module
Then it should provide methods like `isMockMode()`, `isRealMode()`, `getApiBaseUrl()`, and `makeApiCall()`
And automatically route data operations to mock functions or real API endpoints based on the current mode
And provide a consistent interface regardless of the underlying implementation
And include type-safe error handling for configuration-related failures

#### Scenario: Demo configuration logging
Given the demo application starts and validates configuration
When it encounters configuration issues or mode switches
Then it should log appropriate messages: warnings for invalid demo mode values, errors for invalid API URLs in real mode
And include the current operating mode in startup logs for developer visibility
And provide clear guidance on how to fix configuration issues

#### Scenario: Configuration validation and fallbacks
Given the demo starts with missing or invalid configuration
When it validates environment variables
Then it should fall back to mock mode with appropriate logging level based on severity
And provide specific error messages: "Invalid NEXT_PUBLIC_DEMO_MODE value. Expected 'mock' or 'real', falling back to mock mode." or "Invalid CONNECTORS_API_BASE_URL for real mode. Expected valid HTTPS URL, falling back to mock mode."
And continue operation in mock mode to ensure the demo always remains functional
And document all fallback behavior in the console for developer awareness

#### Scenario: Mode-aware educational annotations
Given a user interacts with the demo in either mode
When they view educational annotations
Then the annotations should accurately reflect whether they're seeing mock data or real API responses
And provide appropriate guidance for the current mode