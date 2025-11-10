# nextjs-demo Specification

## Purpose
TBD - created by archiving change establish-nextjs-demo-skeleton. Update Purpose after archive.
## Requirements
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

### Requirement: Mock User Authentication
The system SHALL provide a simulated authentication flow that creates a mock user session using only an email address.

#### Scenario: Demo login flow
Given a user visits the demo landing page
When they enter an email address and click "Continue"
Then the application should create a mock user session
And redirect them to the tenant creation step
And display appropriate educational annotations about mock authentication

#### Scenario: Session persistence handling
Given a user has completed the login flow
When they refresh the page or navigate directly to a deep link
Then the application should handle lost state gracefully
And provide options to restart the demo flow

#### Scenario: Root route session handling
Given a user with an existing demo session visits the root route `/`
When the application loads
Then it should detect the existing session and redirect to the appropriate step:
- If no tenant exists, redirect to `/tenant`
- If tenant exists but no connections, redirect to `/integrations`
- If connections exist but no signals, redirect to `/signals` with scan prompt
- If signals exist, redirect to `/signals` to show the signal list
And always provide an option to reset the demo flow

### Requirement: Tenant Creation and Mapping Visualization
The system SHALL provide tenant creation functionality that demonstrates the mapping between Poblysh tenant IDs and Connectors tenant IDs.

#### Scenario: Tenant creation with dual ID mapping
Given a logged-in mock user with no tenant
When they enter a company name and submit
Then the application should generate both a Poblysh tenant ID and a Connectors tenant ID
And display both IDs with clear explanations of their roles
And show how the mapping would work in production

#### Scenario: Tenant mapping education
Given a user views the tenant summary
When they examine the UI
Then they should see explanations of how `X-Tenant-Id: <connectorsTenantId>` would be used in production
And understand the relationship between Poblysh Core and Connectors service

### Requirement: Mock Integration Management
The system SHALL provide integration management functionality that simulates connecting GitHub and Zoho Cliq providers.

#### Scenario: GitHub connection simulation
Given a user with an active tenant
When they click "Connect GitHub"
Then the application should show a mock OAuth consent interface
And create a GitHub connection with status "connected"
And display educational notes about real OAuth flows

#### Scenario: Integration status management
Given a user views the integrations page
When they examine the available connectors
Then they should see current connection status for each provider
And have options to connect or mock-disconnect integrations
And see appropriate calls-to-action based on connection state

#### Scenario: Zoho Cliq integration (multi-provider demo)
Given a user has connected GitHub
When they view available integrations
Then they should also see Zoho Cliq as an option
And be able to connect it to demonstrate multi-provider scenarios

### Requirement: Mock Signal Discovery and Listing
The system SHALL provide signal discovery and listing functionality that simulates scanning connected integrations for activity signals.

#### Scenario: Signal scan simulation
Given a user with at least one connected integration
When they trigger "Scan for signals"
Then the application should generate realistic mock signals for connected providers
And show appropriate loading states during the "scan"
And display generated signals in a list view

#### Scenario: Signal filtering and searching
Given a user has generated signals
When they use the filter controls
Then they should be able to filter signals by provider
And search through signal titles and summaries
And see the results update immediately

#### Scenario: Signal list pagination simulation
Given a user views the signals list
When they navigate through the results
Then the application should provide a mock pagination experience
And maintain consistent mock data across page changes

### Requirement: Signal Detail and Grounding Demo
The system SHALL provide signal detail and grounding functionality that demonstrates transforming weak signals into grounded signals with cross-connector evidence.

#### Scenario: Signal detail view
Given a user clicks on a signal from the list
When they view the signal detail page
Then they should see comprehensive signal information
Including metadata fields and structured data
And educational notes about real signal structure

#### Scenario: Mock signal grounding
Given a user views a signal detail
When they click "Ground this signal"
Then the application should generate a grounded signal with:
- Overall relevance score (0-100, inclusive)
- Dimensional scores (Relevance, Impact, Recency, etc.) each ranging 0-100 inclusive
- Evidence items from across connected providers
And display results with clear explanations

#### Scenario: Cross-connector evidence demonstration
Given a user has connected both GitHub and Zoho Cliq
When they ground a signal
Then the evidence should include items from both providers
And demonstrate how cross-connector analysis works
And show realistic relationships between different signal types

### Requirement: Reference Implementation Quality
The system SHALL provide reference-quality code that demonstrates best practices for Next.js integration with Connectors concepts.

#### Scenario: Clean component architecture
Given a developer examines the demo code
When they review the component structure
Then they should see well-organized, reusable components
With clear separation of concerns
And patterns that can be applied to real implementations

#### Scenario: Type safety and developer experience
Given a developer works with the demo codebase
When they use TypeScript features
Then they should have full type coverage for domain models
And helpful autocompletion for mock data functions
And clear interfaces that map to real API concepts

#### Scenario: Educational code comments
Given a developer reads through the implementation
When they examine mock functions and components
Then they should see comments explaining real-world equivalents
And references to relevant API endpoints and OpenSpec changes
And guidance on adapting patterns for production use

### Requirement: Demo Accessibility and Responsiveness
The system SHALL provide a responsive and accessible interface that works across different devices and user needs.

#### Scenario: Mobile-friendly interface
Given a user accesses the demo on a mobile device
When they navigate through the flows
Then all interfaces should be usable on smaller screens
With appropriate responsive design patterns

#### Scenario: Accessibility compliance
Given a user with accessibility needs uses the demo
When they navigate with keyboard or screen reader
Then all interactive elements should be accessible
With proper ARIA labels and semantic HTML

### Requirement: Demo Discovery and Documentation
The system SHALL be discoverable from the main project and include comprehensive inline documentation.

#### Scenario: Demo discoverability
Given a developer explores the main project
When they look for examples and documentation
Then they should find clear references to the Next.js demo
With setup instructions and explanations of its purpose

#### Scenario: Inline documentation
Given a user runs the demo application
When they navigate through the flows
Then they should have access to contextual help
And understand what each step represents in the real system
And know where to find more detailed information

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

