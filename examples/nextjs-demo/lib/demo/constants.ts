/**
 * Constants and configuration for the demo.
 */

/**
 * Mock delay durations (in milliseconds).
 * Used to simulate API response times.
 */
export const MOCK_DELAYS = {
  FAST: 500,      // Quick UI updates
  NORMAL: 1000,   // Normal API responses
  SLOW: 2000,     // Slow operations like OAuth
  SCAN: 1500,     // Signal scanning
  GROUND: 1200,   // Signal grounding
} as const;

/**
 * Signal configuration by provider.
 */
export const SIGNAL_CONFIGS = {
  github: {
    // Average number of signals to generate per connection
    signalsPerConnection: 8,
    // Signal kinds and their relative weights
    signalKinds: [
      { kind: 'commit', weight: 40 },
      { kind: 'pull_request_opened', weight: 20 },
      { kind: 'pull_request_closed', weight: 10 },
      { kind: 'pull_request_merged', weight: 15 },
      { kind: 'issue_opened', weight: 10 },
      { kind: 'issue_closed', weight: 3 },
      { kind: 'release_published', weight: 2 },
    ],
  },
  'zoho-cliq': {
    signalsPerConnection: 12,
    signalKinds: [
      { kind: 'message_sent', weight: 35 },
      { kind: 'message_received', weight: 35 },
      { kind: 'mention', weight: 15 },
      { kind: 'thread_started', weight: 10 },
      { kind: 'thread_replied', weight: 5 },
    ],
  },
  'slack': {
    signalsPerConnection: 15,
    signalKinds: [
      { kind: 'message_sent', weight: 30 },
      { kind: 'message_received', weight: 30 },
      { kind: 'mention', weight: 15 },
      { kind: 'reaction_added', weight: 10 },
      { kind: 'file_shared', weight: 8 },
      { kind: 'channel_created', weight: 5 },
      { kind: 'user_added', weight: 2 },
    ],
  },
  'google-workspace': {
    signalsPerConnection: 10,
    signalKinds: [
      { kind: 'email_sent', weight: 25 },
      { kind: 'email_received', weight: 25 },
      { kind: 'document_created', weight: 15 },
      { kind: 'document_shared', weight: 12 },
      { kind: 'calendar_event_created', weight: 10 },
      { kind: 'drive_file_modified', weight: 8 },
      { kind: 'spreadsheet_updated', weight: 5 },
    ],
  },
  'jira': {
    signalsPerConnection: 12,
    signalKinds: [
      { kind: 'issue_created', weight: 20 },
      { kind: 'issue_updated', weight: 25 },
      { kind: 'issue_assigned', weight: 15 },
      { kind: 'sprint_started', weight: 10 },
      { kind: 'workflow_transition', weight: 15 },
      { kind: 'comment_added', weight: 10 },
      { kind: 'resolution_set', weight: 5 },
    ],
  },
} as const;

/**
 * Mock user data for demo purposes.
 */
export const MOCK_USERS = [
  {
    email: 'alice@demo.com',
    name: 'Alice Johnson',
    avatarUrl: '/avatars/alice.jpg',
    roles: ['admin'],
  },
  {
    email: 'bob@demo.com',
    name: 'Bob Smith',
    avatarUrl: '/avatars/bob.jpg',
    roles: ['member'],
  },
  {
    email: 'carol@demo.com',
    name: 'Carol Davis',
    avatarUrl: '/avatars/carol.jpg',
    roles: ['admin', 'member'],
  },
  {
    email: 'david@demo.com',
    name: 'David Wilson',
    avatarUrl: '/avatars/david.jpg',
    roles: ['member'],
  },
  {
    email: 'eve@demo.com',
    name: 'Eve Martinez',
    avatarUrl: '/avatars/eve.jpg',
    roles: ['member'],
  },
] as const;

/**
 * Mock organization names.
 */
export const MOCK_ORGANIZATIONS = [
  'Acme Corporation',
  'Tech Innovators Inc',
  'Digital Solutions Ltd',
  'Global Systems LLC',
  'Future Ventures',
  'Creative Agency',
  'Data Analytics Co',
  'Cloud Services Inc',
] as const;

/**
 * Mock repository names for GitHub.
 */
export const MOCK_REPOSITORIES = [
  'frontend-app',
  'backend-api',
  'mobile-client',
  'data-pipeline',
  'infrastructure',
  'design-system',
  'documentation',
  'testing-suite',
] as const;

/**
 * Mock channel names for Zoho Cliq and Slack.
 */
export const MOCK_CHANNELS = [
  'general',
  'development',
  'design-team',
  'product-discussions',
  'random',
  'announcements',
  'support',
  'engineering',
  'marketing',
  'sales',
  'hr',
  'finance',
] as const;

/**
 * Mock Slack-specific configuration.
 */
export const MOCK_SLACK_USERS = [
  ...MOCK_USERS,
  {
    email: 'grace@demo.com',
    name: 'Grace Chen',
    avatar: '/avatars/grace.jpg',
  },
  {
    email: 'henry@demo.com',
    name: 'Henry Williams',
    avatar: '/avatars/henry.jpg',
  },
] as const;

/**
 * Mock reaction emojis for Slack.
 */
export const MOCK_REACTION_EMOJIS = [
  { emoji: '👍', name: 'thumbsup', weight: 30 },
  { emoji: '❤️', name: 'heart', weight: 25 },
  { emoji: '😄', name: 'smile', weight: 20 },
  { emoji: '🎉', name: 'tada', weight: 10 },
  { emoji: '🤔', name: 'thinking', weight: 8 },
  { emoji: '👀', name: 'eyes', weight: 7 },
] as const;

/**
 * Mock file types for Slack file sharing.
 */
export const MOCK_FILE_TYPES = [
  { type: 'pdf', extension: '.pdf', weight: 25 },
  { type: 'doc', extension: '.docx', weight: 20 },
  { type: 'image', extension: '.png', weight: 30 },
  { type: 'spreadsheet', extension: '.xlsx', weight: 15 },
  { type: 'code', extension: '.js', weight: 10 },
] as const;

/**
 * Grounding dimension configuration.
 */
export const GROUNDING_DIMENSIONS = [
  {
    label: 'Relevance',
    description: 'How relevant is this signal to the current context?',
    weight: 0.3,
  },
  {
    label: 'Impact',
    description: 'What is the potential impact of this signal?',
    weight: 0.25,
  },
  {
    label: 'Timeliness',
    description: 'How recent and time-sensitive is this signal?',
    weight: 0.2,
  },
  {
    label: 'Authority',
    description: 'How authoritative is the source of this signal?',
    weight: 0.15,
  },
  {
    label: 'Corroboration',
    description: 'How much evidence supports this signal?',
    weight: 0.1,
  },
] as const;

/**
 * Evidence type configuration.
 */
export const EVIDENCE_TYPES = [
  {
    type: 'reference' as const,
    description: 'Direct reference to related content',
    baseStrength: 80,
  },
  {
    type: 'mention' as const,
    description: 'Mention of related concepts or entities',
    baseStrength: 60,
  },
  {
    type: 'related_activity' as const,
    description: 'Related activity in the same timeframe',
    baseStrength: 70,
  },
  {
    type: 'cross_reference' as const,
    description: 'Cross-reference between different providers',
    baseStrength: 90,
  },
] as const;