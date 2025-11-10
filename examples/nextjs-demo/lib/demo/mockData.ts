/**
 * Mock data generators for the demo.
 * Creates deterministic, realistic mock data for all entities.
 */

import {
  DemoUser,
  DemoTenant,
  DemoConnection,
  DemoSignal,
  DemoGroundedSignal,
  DemoEvidenceItem,
  DemoSyncJob,
  DemoWebhook,
  DemoToken,
  DemoRateLimit,
} from "./types";
import {
  generateId,
  generateUuid,
  generateTenantIds,
  generateSyncJobId,
  generateWebhookId,
  generateTokenId,
  generateRateLimitId,
} from "./id";
import {
  MOCK_DELAYS,
  SIGNAL_CONFIGS,
  MOCK_USERS,
  MOCK_SLACK_USERS,
  MOCK_ORGANIZATIONS,
  MOCK_REPOSITORIES,
  MOCK_CHANNELS,
  MOCK_REACTION_EMOJIS,
  MOCK_FILE_TYPES,
  GROUNDING_DIMENSIONS,
  EVIDENCE_TYPES,
} from "./constants";

/**
 * Seeded random number generator.
 * Ensures consistent mock data across sessions.
 */
class SeededRandom {
  private seed: number;

  constructor(seed: string) {
    this.seed = this.hashSeed(seed);
  }

  private hashSeed(seed: string): number {
    let hash = 0;
    for (let i = 0; i < seed.length; i++) {
      const char = seed.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash;
    }
    return Math.abs(hash);
  }

  // Returns a number between 0 and 1
  next(): number {
    this.seed = (this.seed * 9301 + 49297) % 233280;
    return this.seed / 233280;
  }

  // Returns an integer between min and max (inclusive)
  int(min: number, max: number): number {
    return Math.floor(this.next() * (max - min + 1)) + min;
  }

  // Returns a random element from an array
  pick<T>(array: readonly T[]): T {
    return array[this.int(0, array.length - 1)];
  }

  // Returns a random subset of an array
  sample<T>(array: readonly T[], count: number): T[] {
    const shuffled = [...array].sort(() => this.next() - 0.5);
    return shuffled.slice(0, Math.min(count, array.length));
  }
}

/**
 * Creates a seeded random generator for a given connection.
 */
function createRandomForConnection(connection: DemoConnection): SeededRandom {
  return new SeededRandom(
    `${connection.tenantId}-${connection.providerSlug}-${connection.id}`,
  );
}

/**
 * Generates a mock user from an email address.
 */
export function generateMockUser(email: string): DemoUser {
  const mockUser = MOCK_USERS.find((u) => u.email === email);

  if (mockUser) {
    return {
      id: generateId(email),
      email: mockUser.email,
      name: mockUser.name,
    };
  }

  // Generate user from email if not in mock list
  const name = email
    .split("@")[0]
    .replace(/[._-]/g, " ")
    .replace(/\b\w/g, (l) => l.toUpperCase());

  return {
    id: generateId(email),
    email,
    name,
  };
}

/**
 * Generates a mock tenant from an organization name.
 */
export function generateMockTenant(organizationName: string): DemoTenant {
  const { tenantId, connectorsTenantId } = generateTenantIds(organizationName);

  return {
    id: tenantId,
    name: organizationName,
    connectorsTenantId,
    createdAt: new Date(
      Date.now() - Math.random() * 30 * 24 * 60 * 60 * 1000,
    ).toISOString(),
  };
}

/**
 * Generates a mock connection for a tenant and provider.
 */
export function generateMockConnection(
  tenantId: string,
  providerSlug: string,
  status: "connected" | "disconnected" | "error" = "connected",
): DemoConnection {
  const random = new SeededRandom(`${tenantId}-${providerSlug}`);
  const displayName = `${providerSlug.charAt(0).toUpperCase() + providerSlug.slice(1)} Integration`;

  return {
    id: generateUuid(`connection-${tenantId}-${providerSlug}`),
    tenantId,
    providerSlug,
    displayName,
    status,
    createdAt: new Date(
      Date.now() - random.int(1, 30) * 24 * 60 * 60 * 1000,
    ).toISOString(),
    lastSyncAt:
      status === "connected"
        ? new Date(
            Date.now() - random.int(0, 7) * 24 * 60 * 60 * 1000,
          ).toISOString()
        : undefined,
    error:
      status === "error"
        ? "Connection failed due to network timeout"
        : undefined,
  };
}

/**
 * Generates mock signals for a connection.
 */
export function generateMockSignals(connection: DemoConnection): DemoSignal[] {
  const random = createRandomForConnection(connection);
  const config =
    SIGNAL_CONFIGS[connection.providerSlug as keyof typeof SIGNAL_CONFIGS];

  if (!config) {
    return [];
  }

  const signals: DemoSignal[] = [];
  const signalCount = random.int(
    Math.max(3, config.signalsPerConnection - 3),
    config.signalsPerConnection + 3,
  );

  for (let i = 0; i < signalCount; i++) {
    const signalKind = selectSignalKind(random, config.signalKinds);
    const occurredAt = new Date(
      Date.now() - random.int(0, 30) * 24 * 60 * 60 * 1000,
    );

    // Include index in signal generation to ensure unique IDs even with same timestamp
    const signal = generateSignalForKind(
      connection,
      signalKind,
      occurredAt,
      random,
      i,
    );
    signals.push(signal);
  }

  // Sort by occurrence time (newest first)
  return signals.sort(
    (a, b) =>
      new Date(b.occurredAt).getTime() - new Date(a.occurredAt).getTime(),
  );
}

/**
 * Selects a signal kind based on weighted probabilities.
 */
function selectSignalKind(
  random: SeededRandom,
  signalKinds: readonly { kind: string; weight: number }[],
): string {
  const totalWeight = signalKinds.reduce((sum, sk) => sum + sk.weight, 0);
  let randomWeight = random.int(0, totalWeight - 1);

  for (const { kind, weight } of signalKinds) {
    randomWeight -= weight;
    if (randomWeight < 0) {
      return kind;
    }
  }

  return signalKinds[0].kind;
}

/**
 * Generates a signal for a specific kind.
 */
function generateSignalForKind(
  connection: DemoConnection,
  kind: string,
  occurredAt: Date,
  random: SeededRandom,
  index: number = 0,
): DemoSignal {
  const author = random.pick(MOCK_USERS).name;

  switch (connection.providerSlug) {
    case "github":
      return generateGitHubSignal(
        connection,
        kind,
        author,
        occurredAt,
        random,
        index,
      );

    case "zoho-cliq":
      return generateZohoCliqSignal(
        connection,
        kind,
        author,
        occurredAt,
        random,
        index,
      );

    case "slack":
      return generateSlackSignal(
        connection,
        kind,
        author,
        occurredAt,
        random,
        index,
      );

    case "google-workspace":
      return generateGoogleWorkspaceSignal(
        connection,
        kind,
        author,
        occurredAt,
        random,
        index,
      );

    case "jira":
      return generateJiraSignal(
        connection,
        kind,
        author,
        occurredAt,
        random,
        index,
      );

    default:
      return generateGenericSignal(
        connection,
        kind,
        author,
        occurredAt,
        random,
      );
  }
}

/**
 * Generates a GitHub-specific signal.
 */
function generateGitHubSignal(
  connection: DemoConnection,
  kind: string,
  author: string,
  occurredAt: Date,
  random: SeededRandom,
  index: number = 0,
): DemoSignal {
  const repository = random.pick(MOCK_REPOSITORIES);
  const prNumber = random.int(1, 999);
  const issueNumber = random.int(1, 999);
  const commitHash = random.int(1000000, 9999999).toString(16);

  let title = "";
  let summary = "";
  let url = "";
  let metadata = {};

  switch (kind) {
    case "commit":
      title = `Commit: ${random.pick(["Fix bug", "Add feature", "Update docs", "Refactor code", "Add tests"])}`;
      summary = `Committed changes to ${repository}`;
      url = `https://github.com/demo/${repository}/commit/${commitHash}`;
      metadata = { repository, commitHash, filesChanged: random.int(1, 10) };
      break;

    case "pull_request_opened":
      title = `PR: ${random.pick(["Add new feature", "Fix critical bug", "Improve performance", "Update dependencies"])}`;
      summary = `Opened pull request #${prNumber} in ${repository}`;
      url = `https://github.com/demo/${repository}/pull/${prNumber}`;
      metadata = { repository, prNumber, state: "open" };
      break;

    case "pull_request_merged":
      title = `PR Merged: ${random.pick(["Feature complete", "Bug fix merged", "Docs updated"])}`;
      summary = `Merged pull request #${prNumber} in ${repository}`;
      url = `https://github.com/demo/${repository}/pull/${prNumber}`;
      metadata = { repository, prNumber, state: "merged" };
      break;

    case "pull_request_closed":
      title = `PR Closed: ${random.pick(["Feature withdrawn", "Duplicate PR", "Superseded by changes"])}`;
      summary = `Closed pull request #${prNumber} in ${repository}`;
      url = `https://github.com/demo/${repository}/pull/${prNumber}`;
      metadata = { repository, prNumber, state: "closed" };
      break;

    case "issue_opened":
      title = `Issue: ${random.pick(["Bug report", "Feature request", "Documentation issue", "Performance problem"])}`;
      summary = `Opened issue #${issueNumber} in ${repository}`;
      url = `https://github.com/demo/${repository}/issues/${issueNumber}`;
      metadata = { repository, issueNumber, state: "open" };
      break;

    case "issue_closed":
      title = `Issue Closed: ${random.pick(["Bug resolved", "Feature implemented", "Documentation updated", "Performance fixed"])}`;
      summary = `Closed issue #${issueNumber} in ${repository}`;
      url = `https://github.com/demo/${repository}/issues/${issueNumber}`;
      metadata = { repository, issueNumber, state: "closed" };
      break;

    case "release_published":
      title = `Release: ${random.pick(["v1.0.0", "v1.1.0", "v2.0.0", "v1.0.1"])}`;
      summary = `Published release in ${repository}`;
      url = `https://github.com/demo/${repository}/releases/tag/v${random.int(1, 3)}.${random.int(0, 9)}.${random.int(0, 9)}`;
      metadata = {
        repository,
        version: `v${random.int(1, 3)}.${random.int(0, 9)}.${random.int(0, 9)}`,
      };
      break;

    default:
      title = `GitHub: ${kind.replace("_", " ")}`;
      summary = `Activity in ${repository}`;
      metadata = { repository };
  }

  return {
    id: generateUuid(
      `signal-${connection.id}-${kind}-${occurredAt.getTime()}-${index}`,
    ),
    tenantId: connection.tenantId,
    providerSlug: connection.providerSlug,
    connectionId: connection.id,
    kind,
    title,
    summary,
    author,
    occurredAt: occurredAt.toISOString(),
    discoveredAt: new Date().toISOString(),
    metadata,
    url,
    relevanceScore: random.int(40, 95),
    // New required fields with default values
    rawPayload: metadata,
    processingDetails: {
      fetchTime: random.int(100, 1000),
      processingTime: random.int(50, 200),
      retryCount: 0,
    },
    relatedSignals: [],
    childSignalIds: [],
    categories: ["development"],
    sentiment: random.pick(["positive", "negative", "neutral"] as const),
    urgency: random.pick(["low", "medium", "high"] as const),
    impact: {
      scope: random.pick(["team", "project"] as const),
      affectedUsers: random.int(1, 10),
      estimatedCost: random.int(0, 1000),
    },
    environment: random.pick(["production", "staging", "development"] as const),
  };
}

/**
 * Generates a Zoho Cliq-specific signal.
 */
function generateZohoCliqSignal(
  connection: DemoConnection,
  kind: string,
  author: string,
  occurredAt: Date,
  random: SeededRandom,
  index: number = 0,
): DemoSignal {
  const channel = random.pick(MOCK_CHANNELS);
  const messageId = random.int(10000, 99999);

  let title = "";
  let summary = "";
  let metadata = {};

  switch (kind) {
    case "message_sent":
      title = `Message in #${channel}`;
      summary = `Posted a message in the ${channel} channel`;
      metadata = { channel, messageId, direction: "outgoing" };
      break;

    case "message_received":
      title = `Message from ${random.pick(MOCK_USERS).name}`;
      summary = `Received a message in the ${channel} channel`;
      metadata = { channel, messageId, direction: "incoming" };
      break;

    case "mention":
      title = `You were mentioned in #${channel}`;
      summary = `${author} mentioned you in a discussion`;
      metadata = { channel, messageId, mentionedUser: "You" };
      break;

    case "thread_started":
      title = `New thread in #${channel}`;
      summary = `Started a new discussion thread`;
      metadata = {
        channel,
        messageId,
        threadId: generateId(`thread-${messageId}`),
      };
      break;

    case "thread_replied":
      title = `Thread reply in #${channel}`;
      summary = `Replied to a discussion thread`;
      metadata = {
        channel,
        messageId,
        threadId: generateId(`thread-${messageId}`),
        action: "reply",
      };
      break;

    default:
      title = `Cliq: ${kind.replace("_", " ")}`;
      summary = `Activity in ${channel}`;
      metadata = { channel };
  }

  return {
    id: generateUuid(
      `signal-${connection.id}-${kind}-${occurredAt.getTime()}-${index}`,
    ),
    tenantId: connection.tenantId,
    providerSlug: connection.providerSlug,
    connectionId: connection.id,
    kind,
    title,
    summary,
    author,
    occurredAt: occurredAt.toISOString(),
    discoveredAt: new Date().toISOString(),
    metadata,
    relevanceScore: random.int(30, 85),
    // New required fields with default values
    rawPayload: metadata,
    processingDetails: {
      fetchTime: random.int(100, 1000),
      processingTime: random.int(50, 200),
      retryCount: 0,
    },
    relatedSignals: [],
    childSignalIds: [],
    categories: ["communication"],
    sentiment: random.pick(["positive", "negative", "neutral"] as const),
    urgency: random.pick(["low", "medium"] as const),
    impact: {
      scope: "team",
      affectedUsers: random.int(1, 5),
      estimatedCost: 0,
    },
    environment: "production",
  };
}

/**
 * Generates a generic signal for unknown providers.
 */
function generateGenericSignal(
  connection: DemoConnection,
  kind: string,
  author: string,
  occurredAt: Date,
  random: SeededRandom,
  index: number = 0,
): DemoSignal {
  const metadata = { provider: connection.providerSlug };
  return {
    id: generateUuid(
      `signal-${connection.id}-${kind}-${occurredAt.getTime()}-${index}`,
    ),
    tenantId: connection.tenantId,
    providerSlug: connection.providerSlug,
    connectionId: connection.id,
    kind,
    title: `${connection.providerSlug}: ${kind.replace("_", " ")}`,
    summary: `Activity from ${connection.providerSlug}`,
    author,
    occurredAt: occurredAt.toISOString(),
    discoveredAt: new Date().toISOString(),
    metadata,
    relevanceScore: random.int(25, 75),
    // New required fields with default values
    rawPayload: metadata,
    processingDetails: {
      fetchTime: random.int(100, 1000),
      processingTime: random.int(50, 200),
      retryCount: 0,
    },
    relatedSignals: [],
    childSignalIds: [],
    categories: ["general"],
    sentiment: "neutral",
    urgency: "low",
    impact: {
      scope: "team",
      affectedUsers: 1,
      estimatedCost: 0,
    },
    environment: "development",
  };
}

/**
 * Generates a Slack-specific signal.
 */
function generateSlackSignal(
  connection: DemoConnection,
  kind: string,
  author: string,
  occurredAt: Date,
  random: SeededRandom,
  index: number = 0,
): DemoSignal {
  const channel = random.pick(MOCK_CHANNELS);
  const user = random.pick(MOCK_SLACK_USERS);
  const messageId = `msg_${random.int(100000, 999999)}`;

  let title = "";
  let summary = "";
  let url = "";
  let metadata = {};

  switch (kind) {
    case "message_sent":
      title = `Message in #${channel}`;
      summary = `Posted a message in the ${channel} channel`;
      metadata = {
        channel,
        messageId,
        threadId:
          random.int(0, 10) > 7
            ? `thread_${random.int(10000, 99999)}`
            : undefined,
        channelType: random.pick(["public", "private"] as const),
      };
      break;

    case "message_received":
      title = `Message from ${user.name}`;
      summary = `Received a message from ${user.name} in #${channel}`;
      metadata = {
        channel,
        messageId,
        sender: user.name,
        channelType: random.pick(["public", "private"] as const),
      };
      break;

    case "mention":
      title = `You were mentioned in #${channel}`;
      summary = `${author} mentioned you in a discussion about ${random.pick(["project update", "deployment", "meeting", "issue"])}`;
      metadata = {
        channel,
        messageId,
        mentionedUser: "You",
        mentionType: "channel",
        threadId: `thread_${random.int(10000, 99999)}`,
      };
      break;

    case "reaction_added":
      const reaction = random.pick(MOCK_REACTION_EMOJIS);
      title = `Reaction ${reaction.emoji} added`;
      summary = `${author} reacted with ${reaction.emoji} to a message in #${channel}`;
      metadata = {
        channel,
        messageId: `msg_${random.int(100000, 999999)}`,
        reaction: reaction.emoji,
        reactionName: reaction.name,
      };
      break;

    case "file_shared":
      const fileType = random.pick(MOCK_FILE_TYPES);
      const fileName = `${random.pick(["report", "presentation", "document", "image", "data"])}${fileType.extension}`;
      title = `File shared: ${fileName}`;
      summary = `${author} shared ${fileName} in #${channel}`;
      url = `https://slack-demo.com/files/${messageId}/${fileName}`;
      metadata = {
        channel,
        fileName,
        fileType: fileType.type,
        fileSize: random.int(1024, 10485760), // 1KB to 10MB
        messageId: `file_${random.int(100000, 999999)}`,
      };
      break;

    case "channel_created":
      title = `New channel created: #${channel}`;
      summary = `${author} created the #${channel} channel`;
      metadata = {
        channel,
        channelType: random.pick(["public", "private"] as const),
        purpose: random.pick([
          "Project discussions",
          "Team updates",
          "Random chat",
          "Support tickets",
        ]),
        members: random.int(2, 50),
      };
      break;

    case "user_added":
      title = `${user.name} joined the workspace`;
      summary = `${user.name} was added to the workspace`;
      metadata = {
        addedUser: user.name,
        workspaceMembers: random.int(5, 200),
        addedBy: author,
      };
      break;

    default:
      title = `Slack: ${kind.replace("_", " ")}`;
      summary = `Activity in ${channel}`;
      metadata = { channel };
  }

  const signalMetadata = {
    ...metadata,
    provider: connection.providerSlug,
    slackTeam: random.pick(["engineering", "marketing", "sales", "ops"]),
    workspace: random.pick([
      "acme-corp",
      "tech-company",
      "startup-io",
      "enterprise-co",
    ]),
  };

  return {
    id: generateUuid(
      `signal-${connection.id}-${kind}-${occurredAt.getTime()}-${index}`,
    ),
    tenantId: connection.tenantId,
    providerSlug: connection.providerSlug,
    connectionId: connection.id,
    kind,
    title,
    summary,
    author,
    occurredAt: occurredAt.toISOString(),
    discoveredAt: new Date().toISOString(),
    metadata: signalMetadata,
    url,
    relevanceScore: random.int(30, 85),
    // Enhanced metadata
    rawPayload: signalMetadata,
    processingDetails: {
      fetchTime: random.int(100, 1000),
      processingTime: random.int(50, 200),
      retryCount: 0,
    },
    relatedSignals: [],
    childSignalIds: [],
    categories: ["communication", "collaboration"],
    sentiment: random.pick(["positive", "negative", "neutral"] as const),
    urgency: random.pick(["low", "medium", "high"] as const),
    impact: {
      scope: random.pick(["team", "project"] as const),
      affectedUsers: random.int(1, 10),
      estimatedCost: 0,
    },
    environment: random.pick(["production", "development"] as const),
  };
}

/**
 * Generates a Google Workspace-specific signal.
 */
function generateGoogleWorkspaceSignal(
  connection: DemoConnection,
  kind: string,
  author: string,
  occurredAt: Date,
  random: SeededRandom,
  index: number = 0,
): DemoSignal {
  const userEmail = random.pick(MOCK_USERS).email;
  const documentId = `doc_${random.int(100000, 999999)}`;
  const calendarEventId = `event_${random.int(10000, 99999)}`;

  let title = "";
  let summary = "";
  let url = "";
  let metadata = {};

  switch (kind) {
    case "email_sent":
      title = `Email: ${random.pick(["Project Update", "Weekly Report", "Meeting Invitation", "Follow-up"])}`;
      summary = `Sent email to ${random.pick(MOCK_USERS).name}`;
      metadata = {
        messageId: `msg_${random.int(100000, 999999)}`,
        subject: title.split(": ")[1],
        recipients: [random.pick(MOCK_USERS).email],
        from: userEmail,
      };
      break;

    case "email_received":
      title = `Email from ${author}`;
      summary = `Received email from ${random.pick(MOCK_USERS).name}`;
      metadata = {
        messageId: `msg_${random.int(100000, 999999)}`,
        subject: random.pick([
          "Urgent: Server Issue",
          "Meeting Tomorrow",
          "Project Update",
          "Code Review Request",
        ]),
        sender: userEmail,
        recipient: userEmail,
      };
      break;

    case "document_created":
      title = `Document: ${random.pick(["Q4 Report", "Project Proposal", "Meeting Notes", "Design Specification"])}`;
      summary = `Created document in Google Drive`;
      url = `https://docs.google.com/document/d/${documentId}`;
      metadata = {
        documentId,
        documentType: random.pick([
          "document",
          "spreadsheet",
          "presentation",
        ] as const),
        owner: userEmail,
        mimeType: "application/vnd.google-apps.document",
      };
      break;

    case "document_shared":
      title = `Document Shared: ${random.pick(["Q4 Report", "Project Proposal"])}`;
      summary = `Shared document with ${random.int(2, 8)} collaborators`;
      url = `https://docs.google.com/document/d/${documentId}`;
      metadata = {
        documentId,
        documentType: "document",
        owner: userEmail,
        collaborators: random.int(2, 8),
        permissions: ["comment", "view"],
      };
      break;

    case "calendar_event_created":
      title = `Calendar: ${random.pick(["Team Standup", "Sprint Planning", "Client Meeting", "Demo Presentation"])}`;
      summary = `Created calendar event: ${random.pick(["Tomorrow at 2pm", "Next Monday at 10am", "December 15th at 3pm"])}`;
      url = `https://calendar.google.com/calendar/event/${calendarEventId}`;
      metadata = {
        eventId: calendarEventId,
        startTime: new Date(
          occurredAt.getTime() + random.int(1, 24) * 60 * 60 * 1000,
        ).toISOString(),
        attendees: random.int(3, 12),
        duration: random.int(30, 120) * 60 * 1000,
        organizer: userEmail,
      };
      break;

    case "drive_file_modified":
      title = `File Modified: ${random.pick(["Budget Spreadsheet", "Product Requirements", "Technical Documentation"])}`;
      summary = `Modified file in Google Drive`;
      metadata = {
        fileId: `file_${random.int(100000, 999999)}`,
        fileName: random.pick([
          "budget_2024.xlsx",
          "requirements.docx",
          "technical_spec.pdf",
        ]),
        fileType: random.pick(["spreadsheet", "document", "pdf"] as const),
        modifiedBy: userEmail,
        version: random.int(1, 5),
      };
      break;

    case "spreadsheet_updated":
      title = `Spreadsheet Updated: ${random.pick(["Sales Data", "Project Timeline", "Budget Forecast"])}`;
      summary = `Updated spreadsheet with new data`;
      metadata = {
        spreadsheetId: `sheet_${random.int(10000, 99999)}`,
        sheetName: random.pick(["Q4", "Sales", "Marketing", "Budget"] as const),
        updatedBy: userEmail,
        cellUpdates: random.int(5, 50),
      };
      break;

    default:
      title = `Google Workspace: ${kind.replace("_", " ")}`;
      summary = `Activity in Google Workspace`;
      metadata = {
        service: random.pick(["gmail", "drive", "calendar", "docs"] as const),
      };
  }

  const signalMetadata = {
    ...metadata,
    provider: connection.providerSlug,
    workspace: random.pick(["acme-corp.com", "tech-company.io"]),
    service: random.pick(["gmail", "drive", "calendar", "docs"] as const),
  };

  return {
    id: generateUuid(
      `signal-${connection.id}-${kind}-${occurredAt.getTime()}-${index}`,
    ),
    tenantId: connection.tenantId,
    providerSlug: connection.providerSlug,
    connectionId: connection.id,
    kind,
    title,
    summary,
    author,
    occurredAt: occurredAt.toISOString(),
    discoveredAt: new Date().toISOString(),
    metadata: signalMetadata,
    url,
    relevanceScore: random.int(40, 90),
    // Enhanced metadata
    rawPayload: signalMetadata,
    processingDetails: {
      fetchTime: random.int(100, 1000),
      processingTime: random.int(50, 200),
      retryCount: 0,
    },
    relatedSignals: [],
    childSignalIds: [],
    categories: ["productivity", "communication"],
    sentiment: random.pick(["positive", "negative", "neutral"] as const),
    urgency: random.pick(["low", "medium", "high"] as const),
    impact: {
      scope: random.pick(["team", "project", "organization"] as const),
      affectedUsers: random.int(1, 15),
      estimatedCost: 0,
    },
    environment: random.pick(["production", "staging", "development"] as const),
  };
}

/**
 * Generates a Jira-specific signal.
 */
function generateJiraSignal(
  connection: DemoConnection,
  kind: string,
  author: string,
  occurredAt: Date,
  random: SeededRandom,
  index: number = 0,
): DemoSignal {
  const projectKey = random.pick(["PROJ", "DEMO", "ENG", "QA", "OPS"] as const);
  const issueNumber = random.int(100, 9999);
  const issueId = `${projectKey}-${issueNumber}`;

  let title = "";
  let summary = "";
  let url = "";
  let metadata = {};

  switch (kind) {
    case "issue_created":
      title = `Issue Created: ${random.pick(["Bug in User Authentication", "Feature Request: Dark Mode", "Improvement: Search Performance"])}`;
      summary = `Created issue #${issueNumber} in ${projectKey}`;
      url = `https://jira.atlassian.com/browse/${issueId}`;
      metadata = {
        issueId,
        issueNumber,
        projectKey,
        issueType: random.pick(["Bug", "Story", "Task", "Epic"] as const),
        priority: random.pick(["High", "Medium", "Low"] as const),
        status: "Open",
        assignee: random.pick(MOCK_USERS).name,
        reporter: author,
      };
      break;

    case "issue_updated":
      title = `Issue Updated: ${issueId} - ${random.pick(["Description Updated", "Priority Changed", "Status Updated"])}`;
      summary = `Updated issue #${issueNumber} in ${projectKey}`;
      url = `https://jira.atlassian.com/browse/${issueId}`;
      metadata = {
        issueId,
        issueNumber,
        projectKey,
        changedField: random.pick([
          "description",
          "priority",
          "status",
          "assignee",
        ] as const),
        newValue: random.pick([
          "In Progress",
          "Done",
          "To Do",
          "Code Review",
        ] as const),
        updatedBy: author,
      };
      break;

    case "issue_assigned":
      title = `Issue Assigned: ${issueId}`;
      summary = `${issueId} assigned to ${random.pick(MOCK_USERS).name}`;
      url = `https://jira.atlassian.com/browse/${issueId}`;
      metadata = {
        issueId,
        issueNumber,
        projectKey,
        assignedTo: random.pick(MOCK_USERS).name,
        assignedBy: author,
        previousAssignee: "Unassigned",
      };
      break;

    case "sprint_started":
      const sprintName = `${projectKey} Sprint ${random.int(1, 20)}`;
      title = `Sprint Started: ${sprintName}`;
      summary = `Started ${sprintName} with ${random.int(5, 15)} issues`;
      metadata = {
        sprintId: `sprint_${random.int(10000, 99999)}`,
        sprintName,
        projectKey,
        duration: random.int(7, 14) * 24 * 60 * 60 * 1000, // milliseconds
        issuesInSprint: random.int(5, 15),
        sprintGoal: random.pick([
          "Complete User Story 1",
          "Fix Critical Bugs",
          "Release V1.0",
        ] as const),
        sprintLeader: random.pick(MOCK_USERS).name,
      };
      break;

    case "workflow_transition":
      title = `Workflow Transition: ${issueId}`;
      summary = `${issueId} moved from ${random.pick(["Open", "In Progress", "Review"])} to ${random.pick(["In Review", "Testing", "Done"])}`;
      url = `https://jira.atlassian.com/browse/${issueId}`;
      metadata = {
        issueId,
        issueNumber,
        projectKey,
        fromStatus: random.pick(["Open", "In Progress", "Review"] as const),
        toStatus: random.pick(["In Review", "Testing", "Done"] as const),
        transitionReason: random.pick([
          "Code review completed",
          "Testing passed",
          "Ready for deployment",
        ] as const),
        transitionBy: author,
      };
      break;

    case "comment_added":
      title = `Comment Added: ${issueId}`;
      summary = `${author} added a comment to ${issueId}`;
      url = `https://jira.atlassian.com/browse/${issueId}`;
      metadata = {
        issueId,
        issueNumber,
        projectKey,
        commentId: `comment_${random.int(10000, 99999)}`,
        commentAuthor: author,
        commentBody: random.pick([
          "Fixed the issue by updating the authentication flow.",
          "I think we should also consider edge cases.",
          "This looks good to me. Ready for merge.",
          "Let me create a subtask for this.",
        ]),
      };
      break;

    case "resolution_set":
      title = `Issue Resolved: ${issueId} - ${random.pick(["Bug Fixed", "Feature Implemented", "Issue Closed"])}`;
      summary = `Resolved issue #${issueNumber} in ${projectKey}`;
      url = `https://jira.atlassian.com/browse/${issueId}`;
      metadata = {
        issueId,
        issueNumber,
        projectKey,
        resolutionType: random.pick([
          "Fixed",
          "Done",
          "Won't Fix",
          "Duplicate",
        ] as const),
        resolvedBy: author,
        resolution: random.pick([
          "Implemented the requested feature.",
          "Fixed the authentication bug reported by QA.",
          "Completed the documentation update.",
          "Marked as duplicate of issue XYZ.",
        ]),
      };
      break;

    default:
      title = `Jira: ${kind.replace("_", " ")}`;
      summary = `Activity in Jira project ${projectKey}`;
      metadata = { projectKey, service: "jira" };
  }

  const signalMetadata = {
    ...metadata,
    provider: connection.providerSlug,
    jiraInstance: random.pick(["jira.atlassian.net", "jira.company.com"]),
    jiraUrl: `https://${random.pick(["jira.atlassian.net", "jira.company.com"])}/browse/`,
  };

  return {
    id: generateUuid(
      `signal-${connection.id}-${kind}-${occurredAt.getTime()}-${index}`,
    ),
    tenantId: connection.tenantId,
    providerSlug: connection.providerSlug,
    connectionId: connection.id,
    kind,
    title,
    summary,
    author,
    occurredAt: occurredAt.toISOString(),
    discoveredAt: new Date().toISOString(),
    metadata: signalMetadata,
    url,
    relevanceScore: random.int(35, 90),
    // Enhanced metadata
    rawPayload: signalMetadata,
    processingDetails: {
      fetchTime: random.int(100, 1000),
      processingTime: random.int(50, 200),
      retryCount: 0,
    },
    relatedSignals: [],
    childSignalIds: [],
    categories: ["project_management", "development"],
    sentiment: random.pick(["positive", "negative", "neutral"] as const),
    urgency: random.pick(["low", "medium", "high", "critical"] as const),
    impact: {
      scope: random.pick(["team", "project", "organization"] as const),
      affectedUsers: random.int(1, 20),
      estimatedCost: random.int(0, 5000),
    },
    environment: random.pick(["production", "development", "staging"] as const),
  };
}

/**
 * Generates evidence items for signal grounding.
 */
export function generateEvidenceItems(
  sourceSignal: DemoSignal,
  allSignals: DemoSignal[],
): DemoEvidenceItem[] {
  const random = new SeededRandom(`evidence-${sourceSignal.id}`);
  const evidence: DemoEvidenceItem[] = [];

  // Find signals from other providers that could be evidence
  const otherProviderSignals = allSignals.filter(
    (signal) =>
      signal.providerSlug !== sourceSignal.providerSlug &&
      signal.id !== sourceSignal.id,
  );

  if (otherProviderSignals.length === 0) {
    return evidence;
  }

  // Generate 1-3 evidence items
  const evidenceCount = Math.min(random.int(1, 3), otherProviderSignals.length);
  const selectedSignals = random.sample(otherProviderSignals, evidenceCount);

  for (const relatedSignal of selectedSignals) {
    const evidenceType = random.pick(EVIDENCE_TYPES);
    const strengthVariation = random.int(-10, 10);
    const strength = Math.max(
      20,
      Math.min(100, evidenceType.baseStrength + strengthVariation),
    );

    evidence.push({
      id: generateUuid(`evidence-${sourceSignal.id}-${relatedSignal.id}`),
      sourceSignalId: sourceSignal.id,
      providerSlug: relatedSignal.providerSlug,
      type: evidenceType.type,
      description: generateEvidenceDescription(
        sourceSignal,
        relatedSignal,
        evidenceType.type,
      ),
      strength,
      relatedSignalId: relatedSignal.id,
    });
  }

  return evidence.sort((a, b) => b.strength - a.strength);
}

/**
 * Generates evidence description based on signal relationship.
 */
function generateEvidenceDescription(
  sourceSignal: DemoSignal,
  relatedSignal: DemoSignal,
  evidenceType: DemoEvidenceItem["type"],
): string {
  switch (evidenceType) {
    case "reference":
      return `Related activity in ${relatedSignal.providerSlug} around the same time`;

    case "mention":
      return `${relatedSignal.providerSlug} activity mentions related topics`;

    case "related_activity":
      return `Corresponding activity in ${relatedSignal.providerSlug}`;

    case "cross_reference":
      return `Cross-provider correlation with ${relatedSignal.providerSlug}`;

    default:
      return `Related signal from ${relatedSignal.providerSlug}`;
  }
}

/**
 * Generates a grounded signal from a source signal.
 */
export function generateGroundedSignal(
  sourceSignal: DemoSignal,
  allSignals: DemoSignal[],
  connections: DemoConnection[], // eslint-disable-line @typescript-eslint/no-unused-vars
): DemoGroundedSignal {
  const random = new SeededRandom(`grounding-${sourceSignal.id}`);
  const evidence = generateEvidenceItems(sourceSignal, allSignals);

  // Calculate dimensional scores
  const dimensions = GROUNDING_DIMENSIONS.map((dimension) => {
    const baseScore = random.int(60, 95);
    const evidenceBonus = evidence.length > 0 ? evidence.length * 5 : 0;
    const score = Math.min(100, baseScore + evidenceBonus);

    return {
      label: dimension.label,
      score,
      description: dimension.description,
    };
  });

  // Calculate overall score (weighted average)
  const overallScore = Math.round(
    dimensions.reduce((sum, dim) => {
      const weight =
        GROUNDING_DIMENSIONS.find((d) => d.label === dim.label)?.weight || 0.2;
      return sum + dim.score * weight;
    }, 0),
  );

  // Determine confidence level
  const confidence =
    overallScore >= 80 ? "high" : overallScore >= 60 ? "medium" : "low";

  // Generate summary
  const summary = generateGroundingSummary(
    sourceSignal,
    evidence,
    overallScore,
  );

  return {
    id: generateUuid(`grounded-${sourceSignal.id}`),
    sourceSignalId: sourceSignal.id,
    tenantId: sourceSignal.tenantId,
    score: overallScore,
    dimensions,
    evidence,
    createdAt: new Date().toISOString(),
    confidence,
    summary,
  };
}

/**
 * Generates a grounding summary.
 */
function generateGroundingSummary(
  sourceSignal: DemoSignal,
  evidence: DemoEvidenceItem[],
  overallScore: number,
): string {
  const evidenceCount = evidence.length;
  const providers = [...new Set(evidence.map((e) => e.providerSlug))];

  if (evidenceCount === 0) {
    return `Signal analyzed with confidence score of ${overallScore}. No cross-provider evidence found.`;
  }

  const providerText =
    providers.length > 1 ? `${providers.join(" and ")}` : providers[0];

  return `Signal grounded with ${overallScore}% confidence based on ${evidenceCount} evidence items from ${providerText}. ${overallScore >= 80 ? "Strong correlation detected." : overallScore >= 60 ? "Moderate correlation detected." : "Weak correlation detected."}`;
}

/**
 * Generates mock sync jobs for a connection.
 */
export function generateMockSyncJobs(
  connection: DemoConnection,
): DemoSyncJob[] {
  const random = new SeededRandom(
    `${connection.tenantId}-${connection.providerSlug}-syncjobs`,
  );
  const jobCount = random.int(2, 8);
  const jobs: DemoSyncJob[] = [];

  for (let i = 0; i < jobCount; i++) {
    const kind = random.pick([
      "full",
      "incremental",
      "webhook_triggered",
    ] as const);
    const status = random.pick([
      "completed",
      "running",
      "pending",
      "failed",
    ] as const);
    const createdAt = new Date(
      Date.now() - random.int(1, 30) * 24 * 60 * 60 * 1000,
    );
    const lastRunAt =
      status === "completed"
        ? new Date(createdAt.getTime() + random.int(1, 7) * 24 * 60 * 60 * 1000)
        : undefined;

    jobs.push({
      id: generateSyncJobId(connection.id, createdAt.getTime() + i),
      tenantId: connection.tenantId,
      connectionId: connection.id,
      providerSlug: connection.providerSlug,
      status,
      kind,
      cursor:
        kind === "incremental" ? `cursor_${random.int(1000, 9999)}` : undefined,
      errorCount: status === "failed" ? random.int(1, 3) : 0,
      lastRunAt: lastRunAt?.toISOString(),
      nextRunAt:
        status === "pending"
          ? new Date(Date.now() + random.int(1, 60) * 60 * 1000).toISOString()
          : undefined,
      createdAt: createdAt.toISOString(),
      errorMessage:
        status === "failed"
          ? random.pick([
              "Network timeout occurred",
              "API rate limit exceeded",
              "Invalid credentials provided",
              "Temporary provider outage",
            ])
          : undefined,
    });
  }

  return jobs.sort(
    (a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
  );
}

/**
 * Generates mock webhook events for a connection.
 */
export function generateMockWebhooks(
  connection: DemoConnection,
): DemoWebhook[] {
  const random = new SeededRandom(
    `${connection.tenantId}-${connection.providerSlug}-webhooks`,
  );
  const webhookCount = random.int(3, 15);
  const webhooks: DemoWebhook[] = [];

  for (let i = 0; i < webhookCount; i++) {
    const eventTypes =
      connection.providerSlug === "github"
        ? ["push", "pull_request", "issues", "release"]
        : ["message", "mention", "reaction"];

    const eventType = random.pick(eventTypes);
    const createdAt = new Date(
      Date.now() - random.int(1, 7) * 24 * 60 * 60 * 1000,
    );
    const processedAt =
      random.int(0, 10) > 2
        ? new Date(createdAt.getTime() + random.int(1000, 5000))
        : undefined;

    webhooks.push({
      id: generateWebhookId(
        connection.providerSlug,
        eventType,
        createdAt.getTime() + i,
      ),
      tenantId: connection.tenantId,
      connectionId: connection.id,
      providerSlug: connection.providerSlug,
      eventType,
      payload: {
        event: eventType,
        provider: connection.providerSlug,
        timestamp: createdAt.toISOString(),
        data: generateMockWebhookPayload(eventType, random),
      },
      signature: `sha256=${random.int(100000, 999999).toString(16)}`,
      verified: random.int(0, 10) > 1,
      processedAt: processedAt?.toISOString(),
      createdAt: createdAt.toISOString(),
    });
  }

  return webhooks.sort(
    (a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
  );
}

/**
 * Generates mock webhook payload data.
 */
function generateMockWebhookPayload(
  eventType: string,
  random: SeededRandom,
): Record<string, unknown> {
  switch (eventType) {
    case "push":
      return {
        ref: `refs/heads/main`,
        commits: random.int(1, 5),
        repository: random.pick(MOCK_REPOSITORIES),
      };
    case "message":
      return {
        channel: random.pick(MOCK_CHANNELS),
        user: random.pick(MOCK_USERS).name,
        text: `Sample message ${random.int(1, 100)}`,
      };
    default:
      return {
        event_id: generateId(`webhook_payload_${eventType}`),
        data: `Sample payload for ${eventType}`,
      };
  }
}

/**
 * Generates mock tokens for a connection.
 */
export function generateMockTokens(connection: DemoConnection): DemoToken[] {
  const random = new SeededRandom(
    `${connection.tenantId}-${connection.providerSlug}-tokens`,
  );
  const tokenTypes: ("oauth" | "api_key" | "service_account")[] = [
    "oauth",
    "api_key",
    "service_account",
  ];
  const tokenType = random.pick(tokenTypes);
  const createdAt = new Date(
    Date.now() - random.int(1, 90) * 24 * 60 * 60 * 1000,
  );

  const expiresAt =
    tokenType === "oauth"
      ? new Date(createdAt.getTime() + random.int(30, 90) * 24 * 60 * 60 * 1000)
      : undefined;

  return [
    {
      id: generateTokenId(connection.id, tokenType),
      connectionId: connection.id,
      tokenType,
      scopes: tokenType === "oauth" ? ["read", "write"] : [],
      expiresAt: expiresAt?.toISOString(),
      lastRefreshed:
        tokenType === "oauth"
          ? new Date(
              createdAt.getTime() + random.int(1, 30) * 24 * 60 * 60 * 1000,
            ).toISOString()
          : undefined,
      status: expiresAt && new Date() > expiresAt ? "expired" : "active",
    },
  ];
}

/**
 * Generates mock rate limit data for a connection.
 */
export function generateMockRateLimits(
  connection: DemoConnection,
): DemoRateLimit[] {
  const random = new SeededRandom(
    `${connection.tenantId}-${connection.providerSlug}-ratelimits`,
  );
  const endpoints = ["api/v3", "api/v4", "webhooks", "search"];
  const rateLimits: DemoRateLimit[] = [];

  for (const endpoint of endpoints) {
    const currentLimit = random.int(100, 5000);
    const used = random.int(0, currentLimit);
    const resetTime = new Date(Date.now() + random.int(1, 3600) * 1000);

    rateLimits.push({
      id: generateRateLimitId(connection.id, endpoint),
      connectionId: connection.id,
      providerSlug: connection.providerSlug,
      endpoint,
      currentLimit,
      remaining: currentLimit - used,
      resetAt: resetTime.toISOString(),
      retryAfter: used >= currentLimit ? random.int(60, 300) : undefined,
    });
  }

  return rateLimits;
}

/**
 * Simulates API delay for better UX.
 */
export function simulateDelay(key: keyof typeof MOCK_DELAYS): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, MOCK_DELAYS[key]));
}

/**
 * Mock API response wrapper.
 */
export function createMockResponse<T>(
  data: T,
  delay?: keyof typeof MOCK_DELAYS,
): Promise<{ data: T }> {
  const actualDelay = delay || "NORMAL";
  return simulateDelay(actualDelay).then(() => ({ data }));
}

/**
 * Mock API error response.
 */
export function createMockError(
  code: string,
  message: string,
  delay?: keyof typeof MOCK_DELAYS,
): Promise<never> {
  const actualDelay = delay || "NORMAL";
  return simulateDelay(actualDelay).then(() => {
    throw new Error(`${code}: ${message}`);
  });
}

/**
 * Error scenario types for advanced simulation.
 */
interface ErrorScenarioConfig {
  rate: number; // 0-1, probability of error occurrence
  types: (
    | "timeout"
    | "auth_failure"
    | "rate_limit"
    | "provider_error"
    | "network_error"
  )[];
  retryable: boolean;
  recoveryTime?: number; // milliseconds
}

const ERROR_SCENARIOS: Record<string, ErrorScenarioConfig> = {
  timeout: {
    rate: 0.1,
    types: ["timeout"],
    retryable: true,
    recoveryTime: 30000, // 30 seconds
  },
  auth_failure: {
    rate: 0.05,
    types: ["auth_failure"],
    retryable: false,
    recoveryTime: 300000, // 5 minutes for manual intervention
  },
  rate_limit: {
    rate: 0.15,
    types: ["rate_limit"],
    retryable: true,
    recoveryTime: 60000, // 1 minute for rate limit reset
  },
  provider_error: {
    rate: 0.08,
    types: ["provider_error"],
    retryable: true,
    recoveryTime: 120000, // 2 minutes
  },
  network_error: {
    rate: 0.12,
    types: ["network_error"],
    retryable: true,
    recoveryTime: 15000, // 15 seconds
  },
};

/**
 * Enhanced error scenarios for sync jobs with realistic retry logic.
 */
export function generateFailedSyncJobs(
  connection: DemoConnection,
): DemoSyncJob[] {
  const random = new SeededRandom(
    `${connection.tenantId}-${connection.providerSlug}-failed-syncjobs`,
  );
  const failedJobCount = Math.max(1, Math.floor(random.int(1, 4) * 0.3)); // 30% failure rate
  const jobs: DemoSyncJob[] = [];

  for (let i = 0; i < failedJobCount; i++) {
    const errorType = random.pick(Object.keys(ERROR_SCENARIOS));
    const scenario = ERROR_SCENARIOS[errorType];
    const kind = random.pick([
      "full",
      "incremental",
      "webhook_triggered",
    ] as const);
    const createdAt = new Date(
      Date.now() - random.int(1, 7) * 24 * 60 * 60 * 1000,
    );

    let errorMessage: string;
    let nextRunAt: string | undefined;

    switch (errorType) {
      case "timeout":
        errorMessage = `Request timeout after 30s - provider API slow to respond`;
        nextRunAt = new Date(Date.now() + scenario.recoveryTime!).toISOString();
        break;
      case "auth_failure":
        errorMessage = `Authentication failed - OAuth token expired or revoked`;
        nextRunAt = undefined; // Requires manual intervention
        break;
      case "rate_limit":
        errorMessage = `API rate limit exceeded - 5000 requests per hour limit reached`;
        nextRunAt = new Date(Date.now() + scenario.recoveryTime!).toISOString();
        break;
      case "provider_error":
        errorMessage = `Provider outage - GitHub API returning 503 Service Unavailable`;
        nextRunAt = new Date(Date.now() + scenario.recoveryTime!).toISOString();
        break;
      case "network_error":
        errorMessage = `Network error - DNS resolution failed for api.github.com`;
        nextRunAt = new Date(Date.now() + scenario.recoveryTime!).toISOString();
        break;
      default:
        errorMessage = `Unknown error occurred during sync operation`;
        nextRunAt = new Date(Date.now() + 60000).toISOString();
    }

    jobs.push({
      id: generateSyncJobId(connection.id, createdAt.getTime() + i),
      tenantId: connection.tenantId,
      connectionId: connection.id,
      providerSlug: connection.providerSlug,
      status: "failed",
      kind,
      cursor:
        kind === "incremental" ? `cursor_${random.int(1000, 9999)}` : undefined,
      errorCount: random.int(1, 5), // Multiple retry attempts
      lastRunAt: new Date(
        createdAt.getTime() + random.int(1, 60) * 60 * 1000,
      ).toISOString(),
      nextRunAt: scenario.retryable ? nextRunAt : undefined,
      createdAt: createdAt.toISOString(),
      errorMessage,
    });
  }

  return jobs;
}

/**
 * Enhanced token scenarios including expired and expiring tokens.
 */
export function generateExpiredTokens(connection: DemoConnection): DemoToken[] {
  const random = new SeededRandom(
    `${connection.tenantId}-${connection.providerSlug}-expired-tokens`,
  );
  const tokens: DemoToken[] = [];

  // Generate one expired token
  const expiredAt = new Date(
    Date.now() - random.int(1, 30) * 24 * 60 * 60 * 1000,
  );
  tokens.push({
    id: generateTokenId(connection.id, "oauth_expired"),
    connectionId: connection.id,
    tokenType: "oauth",
    scopes: ["read", "write"],
    expiresAt: expiredAt.toISOString(),
    lastRefreshed: new Date(
      expiredAt.getTime() - random.int(1, 7) * 24 * 60 * 60 * 1000,
    ).toISOString(),
    status: "expired",
  });

  // Generate one token that needs refresh soon
  const expiresSoon = new Date(
    Date.now() + random.int(1, 7) * 24 * 60 * 60 * 1000,
  );
  tokens.push({
    id: generateTokenId(connection.id, "oauth_expiring"),
    connectionId: connection.id,
    tokenType: "oauth",
    scopes: ["read", "write"],
    expiresAt: expiresSoon.toISOString(),
    lastRefreshed: new Date(
      expiresSoon.getTime() - random.int(20, 30) * 24 * 60 * 60 * 1000,
    ).toISOString(),
    status: "active", // Still active but will expire soon
  });

  // Generate one refreshing token
  tokens.push({
    id: generateTokenId(connection.id, "oauth_refreshing"),
    connectionId: connection.id,
    tokenType: "oauth",
    scopes: ["read", "write"],
    expiresAt: new Date(
      Date.now() + random.int(30, 90) * 24 * 60 * 60 * 1000,
    ).toISOString(),
    lastRefreshed: new Date(
      Date.now() - random.int(1, 6) * 60 * 60 * 1000,
    ).toISOString(),
    status: "refreshing",
  });

  // Generate one revoked token
  tokens.push({
    id: generateTokenId(connection.id, "oauth_revoked"),
    connectionId: connection.id,
    tokenType: "oauth",
    scopes: ["read", "write"],
    expiresAt: new Date(
      Date.now() + random.int(30, 90) * 24 * 60 * 60 * 1000,
    ).toISOString(),
    lastRefreshed: new Date(
      Date.now() - random.int(1, 15) * 24 * 60 * 60 * 1000,
    ).toISOString(),
    status: "revoked",
  });

  return tokens;
}

/**
 * Enhanced rate limit scenarios including exhausted limits.
 */
export function generateRateLimitedConnections(
  connection: DemoConnection,
): DemoRateLimit[] {
  const rateLimits: DemoRateLimit[] = [];

  // Provider-specific rate limits
  const providerLimits = {
    github: { hourly: 5000, minute: 60 },
    slack: { hourly: 5000, minute: 200 },
    "google-workspace": { hourly: 10000, minute: 100 },
    jira: { hourly: 2500, minute: 60 },
    "zoho-cliq": { hourly: 10000, minute: 100 },
  };

  const limits = providerLimits[
    connection.providerSlug as keyof typeof providerLimits
  ] || { hourly: 5000, minute: 60 };

  // Generate exhausted rate limit for API endpoint
  rateLimits.push({
    id: generateRateLimitId(connection.id, "api"),
    connectionId: connection.id,
    providerSlug: connection.providerSlug,
    endpoint: "api/v4",
    currentLimit: limits.minute,
    remaining: 0, // Exhausted
    resetAt: new Date(Date.now() + 45000).toISOString(), // 45 seconds until reset
    retryAfter: 45,
  });

  // Generate nearly exhausted rate limit for search endpoint
  rateLimits.push({
    id: generateRateLimitId(connection.id, "search"),
    connectionId: connection.id,
    providerSlug: connection.providerSlug,
    endpoint: "search",
    currentLimit: 30, // Search has lower limits
    remaining: 2, // Nearly exhausted
    resetAt: new Date(Date.now() + 120000).toISOString(), // 2 minutes until reset
    retryAfter: undefined,
  });

  // Generate healthy rate limit for webhooks
  rateLimits.push({
    id: generateRateLimitId(connection.id, "webhooks"),
    connectionId: connection.id,
    providerSlug: connection.providerSlug,
    endpoint: "webhooks",
    currentLimit: 1000,
    remaining: 987, // Healthy
    resetAt: new Date(Date.now() + 3600000).toISOString(), // 1 hour until reset
    retryAfter: undefined,
  });

  return rateLimits;
}

/**
 * Enhanced webhook failure scenarios.
 */
export function generateFailedWebhooks(
  connection: DemoConnection,
): DemoWebhook[] {
  const random = new SeededRandom(
    `${connection.tenantId}-${connection.providerSlug}-failed-webhooks`,
  );
  const failedWebhookCount = Math.max(1, Math.floor(random.int(2, 6) * 0.2)); // 20% failure rate
  const webhooks: DemoWebhook[] = [];

  const eventTypes = {
    github: ["push", "pull_request", "issues"],
    slack: ["message", "mention", "reaction"],
    "google-workspace": ["mail.receive", "drive.change", "calendar.event"],
    jira: ["jira:issue_created", "jira:sprint_started", "jira:worklog_updated"],
    "zoho-cliq": ["message", "mention", "reaction"],
  };

  const providerEvents = eventTypes[
    connection.providerSlug as keyof typeof eventTypes
  ] || ["generic"];

  for (let i = 0; i < failedWebhookCount; i++) {
    const eventType = random.pick(providerEvents);
    const createdAt = new Date(Date.now() - random.int(1, 3) * 60 * 60 * 1000); // Last 3 hours

    let payload: Record<string, unknown>;

    // Generate realistic payload based on provider and event type
    switch (connection.providerSlug) {
      case "github":
        if (eventType === "push") {
          payload = {
            ref: "refs/heads/main",
            repository: { name: random.pick(MOCK_REPOSITORIES) },
            commits: [
              {
                message: "Fix critical bug",
                author: { email: random.pick(MOCK_USERS).email },
              },
            ],
            sender: {
              login: random
                .pick(MOCK_USERS)
                .name.toLowerCase()
                .replace(" ", ""),
            },
          };
        } else {
          payload = {
            action: "opened",
            issue: { number: random.int(1, 999), title: "Bug report" },
            sender: {
              login: random
                .pick(MOCK_USERS)
                .name.toLowerCase()
                .replace(" ", ""),
            },
          };
        }
        break;
      case "slack":
        payload = {
          type: "message",
          channel: random.pick(MOCK_CHANNELS),
          user: random
            .pick(MOCK_SLACK_USERS)
            .name.toLowerCase()
            .replace(" ", ""),
          text: "Hello team!",
          event_ts: (Date.now() / 1000).toString(),
        };
        break;
      case "google-workspace":
        payload = {
          emailAddress: random.pick(MOCK_USERS).email,
          message: {
            id: `msg_${random.int(10000, 99999)}`,
            threadId: `thread_${random.int(1000, 9999)}`,
          },
        };
        break;
      case "jira":
        payload = {
          issue: {
            key: `PROJ-${random.int(100, 999)}`,
            fields: { summary: "New feature request" },
          },
          user: { displayName: random.pick(MOCK_USERS).name },
        };
        break;
      default:
        payload = {};
    }

    // Simulate signature verification failures (30% of failed webhooks)
    const isVerified = random.next() > 0.3;

    webhooks.push({
      id: generateWebhookId(
        connection.providerSlug,
        eventType,
        createdAt.getTime() + i,
      ),
      tenantId: connection.tenantId,
      connectionId: connection.id,
      providerSlug: connection.providerSlug,
      eventType,
      payload,
      signature: isVerified
        ? `sha256=${generateId(`webhook_signature_${eventType}`)}`
        : undefined,
      verified: isVerified,
      processedAt: undefined, // Failed webhooks are not processed
      createdAt: createdAt.toISOString(),
    });
  }

  return webhooks;
}

/**
 * Simulates exponential backoff retry logic for failed operations.
 */
export function simulateExponentialBackoff(
  attemptNumber: number,
  baseDelay: number = 1000,
  maxDelay: number = 300000,
): number {
  const exponentialDelay = baseDelay * Math.pow(2, attemptNumber - 1);
  const jitter = Math.random() * 0.1 * exponentialDelay; // Add 10% jitter
  return Math.min(maxDelay, exponentialDelay + jitter);
}

/**
 * Simulates token refresh workflow.
 */
export function simulateTokenRefresh(token: DemoToken): DemoToken {
  const now = new Date();
  const newExpiresAt = new Date(now.getTime() + 60 * 24 * 60 * 60 * 1000); // 60 days from now

  return {
    ...token,
    expiresAt: newExpiresAt.toISOString(),
    lastRefreshed: now.toISOString(),
    status: "active" as const,
  };
}

/**
 * Simulates partial sync failure with recovery.
 */
export function simulatePartialSyncFailure(
  connection: DemoConnection,
  totalSignals: number,
): {
  successful: number;
  failed: number;
  retryAfter: number;
  errorMessages: string[];
} {
  const random = new SeededRandom(
    `${connection.tenantId}-${connection.providerSlug}-partial-failure`,
  );
  const failureRate = 0.1 + random.next() * 0.2; // 10-30% failure rate
  const failed = Math.floor(totalSignals * failureRate);
  const successful = totalSignals - failed;

  const possibleErrorMessages = [
    `Failed to fetch ${failed} signals due to API timeout`,
    `Partial sync completed - ${failed} signals failed to process`,
    `Rate limit reached during sync, ${failed} signals pending`,
  ];

  const selectedErrorMessages = possibleErrorMessages.slice(
    0,
    Math.min(3, failed),
  );

  return {
    successful,
    failed,
    retryAfter: simulateExponentialBackoff(1),
    errorMessages: selectedErrorMessages,
  };
}

/**
 * Webhook processing simulation functions for Task 3.2
 */

/**
 * Processes a webhook event and converts it to a signal.
 */
export function processWebhookToSignal(
  webhook: DemoWebhook,
): DemoSignal | null {
  if (!webhook.verified) {
    return null; // Skip unverified webhooks
  }

  // Strongly-typed webhook payload shapes for key providers.
  interface GitHubPushPayload {
    ref?: string;
    repository?: { name?: string };
    commits?: Array<{ message?: string }>;
  }

  interface GitHubIssueLike {
    number?: number;
    title?: string;
  }

  interface GitHubIssueEventPayload {
    action?: string;
    issue?: GitHubIssueLike;
  }

  interface SlackGenericPayload {
    channel?: string;
    user?: string;
    text?: string;
  }

  interface GoogleMailReceivePayload {
    emailAddress?: string;
    message?: { id?: string };
  }

  interface GoogleDriveChangePayload {
    fileId?: string;
  }

  interface GoogleCalendarEventPayload {
    eventId?: string;
  }

  interface JiraIssue {
    key?: string;
    fields?: {
      summary?: string;
    };
  }

  interface JiraUser {
    displayName?: string;
  }

  interface JiraIssueCreatedPayload {
    issue?: JiraIssue;
    user?: JiraUser;
  }

  interface JiraSprintStartedPayload {
    sprintId?: string | number;
    name?: string;
  }

  interface JiraWorklogUpdatedPayload {
    issue?: JiraIssue;
    timeSpent?: number | string;
  }

  interface ZohoCliqMessagePayload {
    channel?: string;
    user?: string;
  }

  const random = new SeededRandom(`${webhook.id}-signal-conversion`);
  const occurredAt = new Date(webhook.createdAt);

  let kind: string;
  let title: string;
  let summary: string;
  let url: string | undefined;
  let metadata: Record<string, unknown>;

  // Convert webhook to signal based on provider and event type
  switch (webhook.providerSlug) {
    case "github":
      switch (webhook.eventType) {
        case "push": {
          const payload = webhook.payload as GitHubPushPayload;
          kind = "commit";
          const repoName = payload.repository?.name || "unknown-repo";
          const firstMessage = payload.commits && payload.commits[0]?.message;
          title = `Commit: ${firstMessage || "Multiple commits"}`;
          summary = `Pushed to ${repoName}`;
          url = `https://github.com/demo/${repoName}/commits`;
          metadata = {
            repository: repoName,
            commits: payload.commits ? payload.commits.length : 1,
            ref: payload.ref,
          };
          break;
        }
        case "pull_request": {
          const payload = webhook.payload as GitHubIssueEventPayload;
          const action = payload.action || "updated";
          kind =
            action === "opened"
              ? "pull_request_opened"
              : action === "closed"
                ? "pull_request_closed"
                : "pull_request_closed";
          const issue = payload.issue;
          title = `PR: ${issue?.title || "Pull request"}`;
          summary = `${action} pull request`;
          url =
            issue?.number !== undefined
              ? `https://github.com/demo/repo/pull/${issue.number}`
              : undefined;
          metadata = {
            action,
            issue_number: issue?.number,
          };
          break;
        }
        case "issues": {
          const payload = webhook.payload as GitHubIssueEventPayload;
          const action = payload.action || "updated";
          kind =
            action === "opened"
              ? "issue_opened"
              : action === "closed"
                ? "issue_closed"
                : "issue_updated";
          const issue = payload.issue;
          title = `Issue: ${issue?.title || "Issue"}`;
          summary = `${action} issue`;
          url =
            issue?.number !== undefined
              ? `https://github.com/demo/repo/issues/${issue.number}`
              : undefined;
          metadata = {
            action,
            issue_number: issue?.number,
          };
          break;
        }
        default:
          return null;
      }
      break;

    case "slack":
      switch (webhook.eventType) {
        case "message": {
          const payload = webhook.payload as SlackGenericPayload;
          kind = "message_sent";
          const userName = payload.user || "Unknown User";
          title = `Message from ${userName}`;
          summary = `New message in ${payload.channel || "unknown-channel"}`;
          metadata = {
            channel: payload.channel,
            user: userName,
            text: payload.text,
          };
          break;
        }
        case "mention": {
          const payload = webhook.payload as SlackGenericPayload;
          kind = "mention";
          title = `Mention in ${payload.channel || "unknown-channel"}`;
          summary = "User mentioned in channel";
          metadata = {
            channel: payload.channel,
            user: payload.user,
          };
          break;
        }
        case "reaction_added": {
          const payload = webhook.payload as SlackGenericPayload;
          kind = "reaction_added";
          title = "Reaction added to message";
          summary = "User reacted to message";
          metadata = {
            reaction: "👍",
            user: payload.user,
          };
          break;
        }
        default:
          return null;
      }
      break;

    case "google-workspace":
      switch (webhook.eventType) {
        case "mail.receive": {
          const payload = webhook.payload as GoogleMailReceivePayload;
          kind = "email_received";
          title = `Email from ${payload.emailAddress || "unknown-sender"}`;
          summary = "New email received";
          metadata = {
            sender: payload.emailAddress,
            messageId: payload.message?.id,
          };
          break;
        }
        case "drive.change": {
          const payload = webhook.payload as GoogleDriveChangePayload;
          kind = "drive_file_modified";
          title = "File modified in Google Drive";
          summary = "Drive file updated";
          metadata = {
            fileId: payload.fileId,
            changeType: "modify",
          };
          break;
        }
        case "calendar.event": {
          const payload = webhook.payload as GoogleCalendarEventPayload;
          kind = "calendar_event_created";
          title = "Calendar event created";
          summary = "New event added to calendar";
          metadata = {
            eventId: payload.eventId,
          };
          break;
        }
        default:
          return null;
      }
      break;

    case "jira":
      switch (webhook.eventType) {
        case "jira:issue_created": {
          const payload = webhook.payload as JiraIssueCreatedPayload;
          const issue = payload.issue;
          kind = "issue_created";
          title = `Issue: ${issue?.fields?.summary || "New Issue"}`;
          summary = `Issue ${issue?.key || "UNKNOWN"} created`;
          metadata = {
            issueKey: issue?.key,
            summary: issue?.fields?.summary,
            assignee: payload.user?.displayName,
          };
          break;
        }
        case "jira:sprint_started": {
          const payload = webhook.payload as JiraSprintStartedPayload;
          kind = "sprint_started";
          title = "Sprint started";
          summary = "New sprint begun";
          metadata = {
            sprintId: payload.sprintId,
            sprintName: payload.name,
          };
          break;
        }
        case "jira:worklog_updated": {
          const payload = webhook.payload as JiraWorklogUpdatedPayload;
          const issue = payload.issue;
          kind = "issue_updated";
          title = `Work logged for ${issue?.key || "UNKNOWN"}`;
          summary = "Time tracking updated";
          metadata = {
            issueKey: issue?.key,
            timeSpent: payload.timeSpent,
          };
          break;
        }
        default:
          return null;
      }
      break;

    case "zoho-cliq":
      switch (webhook.eventType) {
        case "message": {
          const payload = webhook.payload as ZohoCliqMessagePayload;
          kind = "message_sent";
          title = `Message from ${payload.user || "Unknown User"}`;
          summary = `New message in ${payload.channel || "unknown-channel"}`;
          metadata = {
            channel: payload.channel,
            user: payload.user,
          };
          break;
        }
        default:
          return null;
      }
      break;

    default:
      return null;
  }

  const payloadUser =
    (webhook.payload as { user?: string } | undefined)?.user || "System";

  return {
    id: generateId(`signal_${webhook.id}`),
    tenantId: webhook.tenantId,
    providerSlug: webhook.providerSlug,
    connectionId: webhook.connectionId,
    kind,
    title,
    summary,
    author: payloadUser,
    occurredAt: occurredAt.toISOString(),
    discoveredAt: new Date().toISOString(),
    metadata,
    url,
    relevanceScore: random.int(60, 95),

    // Extended metadata fields
    rawPayload: webhook.payload,
    processingDetails: {
      fetchTime: random.int(50, 500),
      processingTime: random.int(10, 100),
      retryCount: 0,
    },
    relatedSignals: [],
    parentSignalId: undefined,
    childSignalIds: [],
    categories: ["webhook"],
    sentiment: "neutral" as const,
    urgency: "medium" as const,
    impact: {
      scope: "team" as const,
      affectedUsers: random.int(1, 5),
      estimatedCost: 0,
    },
    environment: "production" as const,
  };
}

/**
 * Simulates webhook processing with realistic delays and failures.
 */
export function simulateWebhookProcessing(webhook: DemoWebhook): Promise<{
  success: boolean;
  signal?: DemoSignal;
  processingTime: number;
  error?: string;
}> {
  const random = new SeededRandom(`${webhook.id}-processing`);
  const baseProcessingTime = random.int(100, 2000); // 100ms to 2s

  // Create a manual delay since simulateDelay expects predefined constants
  return new Promise(() => {
    setTimeout(() => {
      // Simulate processing failures (15% failure rate)
      if (random.next() < 0.15) {
        const errors = [
          "Webhook signature validation failed",
          "Payload schema validation error",
          "Temporary processing queue full",
          "Database connection timeout",
          "Rate limit exceeded for webhook processing",
        ];
        return {
          success: false,
          processingTime: baseProcessingTime,
          error: random.pick(errors),
        };
      }

      // Process webhook to signal
      const signal = processWebhookToSignal(webhook);

      if (!signal) {
        return {
          success: false,
          processingTime: baseProcessingTime,
          error: "Unable to convert webhook to signal",
        };
      }

      return {
        success: true,
        signal,
        processingTime: baseProcessingTime,
      };
    }, baseProcessingTime);
  });
}

/**
 * Generates webhook status monitoring data.
 */
export function generateWebhookStatusData(connections: DemoConnection[]): {
  totalWebhooks: number;
  processedWebhooks: number;
  failedWebhooks: number;
  successRate: number;
  averageProcessingLatency: number;
  queueDepth: number;
  providers: Array<{
    providerSlug: string;
    totalWebhooks: number;
    successRate: number;
    averageLatency: number;
    lastWebhookAt?: string;
  }>;
} {
  const random = new SeededRandom("webhook-status-monitoring");
  let totalWebhooks = 0;
  let processedWebhooks = 0;
  let failedWebhooks = 0;
  let totalProcessingTime = 0;
  const providerStats = new Map<
    string,
    {
      total: number;
      processed: number;
      totalTime: number;
      lastWebhook: Date;
    }
  >();

  connections.forEach((connection) => {
    const webhookCount = random.int(5, 50);
    const successRate = 0.85 + random.next() * 0.12; // 85-97% success rate
    const processedCount = Math.floor(webhookCount * successRate);
    const failedCount = webhookCount - processedCount;
    const avgLatency = random.int(200, 1500);

    totalWebhooks += webhookCount;
    processedWebhooks += processedCount;
    failedWebhooks += failedCount;
    totalProcessingTime += processedCount * avgLatency;

    const existing = providerStats.get(connection.providerSlug) || {
      total: 0,
      processed: 0,
      totalTime: 0,
      lastWebhook: new Date(0),
    };

    providerStats.set(connection.providerSlug, {
      total: existing.total + webhookCount,
      processed: existing.processed + processedCount,
      totalTime: existing.totalTime + processedCount * avgLatency,
      lastWebhook: new Date(
        Math.max(
          existing.lastWebhook.getTime(),
          Date.now() - random.int(1, 3600) * 1000,
        ),
      ),
    });
  });

  const overallSuccessRate =
    totalWebhooks > 0 ? processedWebhooks / totalWebhooks : 0;
  const averageLatency =
    processedWebhooks > 0 ? totalProcessingTime / processedWebhooks : 0;

  const providers = Array.from(providerStats.entries()).map(
    ([providerSlug, stats]) => ({
      providerSlug,
      totalWebhooks: stats.total,
      successRate: stats.total > 0 ? stats.processed / stats.total : 0,
      averageLatency:
        stats.processed > 0 ? stats.totalTime / stats.processed : 0,
      lastWebhookAt:
        stats.lastWebhook.getTime() > 0
          ? stats.lastWebhook.toISOString()
          : undefined,
    }),
  );

  return {
    totalWebhooks,
    processedWebhooks,
    failedWebhooks,
    successRate: overallSuccessRate,
    averageProcessingLatency: averageLatency,
    queueDepth: random.int(0, 10), // Simulate current queue depth
    providers,
  };
}

/**
 * Simulates webhook deduplication and idempotency.
 */
export function simulateWebhookDeduplication(webhooks: DemoWebhook[]): {
  uniqueWebhooks: DemoWebhook[];
  duplicates: Array<{
    originalId: string;
    duplicateId: string;
    detectedAt: string;
  }>;
} {
  const seen = new Map<string, DemoWebhook>();
  const duplicates: Array<{
    originalId: string;
    duplicateId: string;
    detectedAt: string;
  }> = [];

  webhooks.forEach((webhook) => {
    const key = `${webhook.providerSlug}-${webhook.eventType}-${JSON.stringify(webhook.payload)}`;

    if (seen.has(key)) {
      const original = seen.get(key)!;
      duplicates.push({
        originalId: original.id,
        duplicateId: webhook.id,
        detectedAt: new Date().toISOString(),
      });
    } else {
      seen.set(key, webhook);
    }
  });

  return {
    uniqueWebhooks: Array.from(seen.values()),
    duplicates,
  };
}

/**
 * Rate limiting simulation functions for Task 3.3
 */

/**
 * Simulates real-time rate limiting state for connections.
 */
export function simulateRateLimitingState(connection: DemoConnection): {
  currentLimits: DemoRateLimit[];
  queueStatus: {
    queuedOperations: number;
    estimatedWaitTime: number;
    priorityOperations: number;
  };
  limitStatus: {
    api: { limited: boolean; resetIn: number; requestsRemaining: number };
    search: { limited: boolean; resetIn: number; requestsRemaining: number };
    webhook: { limited: boolean; resetIn: number; requestsRemaining: number };
  };
} {
  const random = new SeededRandom(`${connection.id}-rate-limit-state`);
  const providerLimits = {
    github: { api: 60, search: 10, webhook: 1000 },
    slack: { api: 200, search: 20, webhook: 1000 },
    "google-workspace": { api: 100, search: 10, webhook: 1000 },
    jira: { api: 60, search: 20, webhook: 1000 },
    "zoho-cliq": { api: 100, search: 15, webhook: 1000 },
  };

  const limits = providerLimits[
    connection.providerSlug as keyof typeof providerLimits
  ] || { api: 60, search: 10, webhook: 1000 };

  // Simulate current usage
  const apiUsage = random.int(0, limits.api);
  const searchUsage = random.int(0, limits.search);
  const webhookUsage = random.int(0, limits.webhook);

  // Determine if rate limits are hit
  const apiLimited = apiUsage >= limits.api * 0.9; // 90% threshold
  const searchLimited = searchUsage >= limits.search * 0.9;
  const webhookLimited = webhookUsage >= limits.webhook * 0.9;

  // Generate rate limit objects
  const currentLimits: DemoRateLimit[] = [
    {
      id: generateRateLimitId(connection.id, "api"),
      connectionId: connection.id,
      providerSlug: connection.providerSlug,
      endpoint: "api",
      currentLimit: limits.api,
      remaining: Math.max(0, limits.api - apiUsage),
      resetAt: new Date(
        Date.now() + (apiLimited ? 45000 : 3600000),
      ).toISOString(),
      retryAfter: apiLimited ? 45 : undefined,
    },
    {
      id: generateRateLimitId(connection.id, "search"),
      connectionId: connection.id,
      providerSlug: connection.providerSlug,
      endpoint: "search",
      currentLimit: limits.search,
      remaining: Math.max(0, limits.search - searchUsage),
      resetAt: new Date(
        Date.now() + (searchLimited ? 120000 : 3600000),
      ).toISOString(),
      retryAfter: searchLimited ? 120 : undefined,
    },
    {
      id: generateRateLimitId(connection.id, "webhook"),
      connectionId: connection.id,
      providerSlug: connection.providerSlug,
      endpoint: "webhook",
      currentLimit: limits.webhook,
      remaining: Math.max(0, limits.webhook - webhookUsage),
      resetAt: new Date(
        Date.now() + (webhookLimited ? 30000 : 3600000),
      ).toISOString(),
      retryAfter: webhookLimited ? 30 : undefined,
    },
  ];

  // Simulate queue status
  const queuedOperations = random.int(0, apiLimited ? 15 : 3);
  const estimatedWaitTime = apiLimited
    ? random.int(30000, 120000)
    : random.int(1000, 10000);
  const priorityOperations = random.int(0, 2);

  return {
    currentLimits,
    queueStatus: {
      queuedOperations,
      estimatedWaitTime,
      priorityOperations,
    },
    limitStatus: {
      api: {
        limited: apiLimited,
        resetIn: apiLimited ? 45000 : 3600000,
        requestsRemaining: Math.max(0, limits.api - apiUsage),
      },
      search: {
        limited: searchLimited,
        resetIn: searchLimited ? 120000 : 3600000,
        requestsRemaining: Math.max(0, limits.search - searchUsage),
      },
      webhook: {
        limited: webhookLimited,
        resetIn: webhookLimited ? 30000 : 3600000,
        requestsRemaining: Math.max(0, limits.webhook - webhookUsage),
      },
    },
  };
}

/**
 * Simulates rate limit-aware operation scheduling.
 */
export function simulateRateLimitAwareScheduling(
  connections: DemoConnection[],
  pendingOperations: Array<{
    id: string;
    connectionId: string;
    priority: "low" | "medium" | "high";
    type: "sync" | "webhook_processing" | "api_call";
    createdAt: string;
  }>,
): {
  scheduledOperations: Array<{
    id: string;
    scheduledAt: string;
    estimatedDuration: number;
    reason: string;
  }>;
  queuedOperations: Array<{
    id: string;
    reason: string;
    estimatedWait: number;
  }>;
  rateLimitDelays: Array<{
    connectionId: string;
    endpoint: string;
    delay: number;
    resetAt: string;
  }>;
} {
  const random = new SeededRandom("rate-limit-scheduling");
  const scheduledOperations: Array<{
    id: string;
    scheduledAt: string;
    estimatedDuration: number;
    reason: string;
  }> = [];
  const queuedOperations: Array<{
    id: string;
    reason: string;
    estimatedWait: number;
  }> = [];
  const rateLimitDelays: Array<{
    connectionId: string;
    endpoint: string;
    delay: number;
    resetAt: string;
  }> = [];

  connections.forEach((connection) => {
    const rateLimitState = simulateRateLimitingState(connection);
    const connectionOperations = pendingOperations.filter(
      (op) => op.connectionId === connection.id,
    );

    connectionOperations.forEach((operation) => {
      const canProceed =
        !rateLimitState.limitStatus.api.limited ||
        operation.priority === "high";

      if (canProceed) {
        // Schedule the operation
        const delay = random.int(0, 5000); // 0-5 second delay
        scheduledOperations.push({
          id: operation.id,
          scheduledAt: new Date(Date.now() + delay).toISOString(),
          estimatedDuration: random.int(2000, 10000), // 2-10 seconds
          reason:
            rateLimitState.limitStatus.api.limited &&
            operation.priority === "high"
              ? "High priority, proceeding despite rate limit warning"
              : "Rate limit headroom available",
        });
      } else {
        // Queue the operation
        const waitTime =
          rateLimitState.limitStatus.api.resetIn + random.int(1000, 5000);
        queuedOperations.push({
          id: operation.id,
          reason: `API rate limit exceeded for ${connection.providerSlug}`,
          estimatedWait: waitTime,
        });
      }

      // Add rate limit delay info
      if (rateLimitState.limitStatus.api.limited) {
        rateLimitDelays.push({
          connectionId: connection.id,
          endpoint: "api",
          delay: rateLimitState.limitStatus.api.resetIn,
          resetAt: new Date(
            Date.now() + rateLimitState.limitStatus.api.resetIn,
          ).toISOString(),
        });
      }
    });
  });

  return {
    scheduledOperations,
    queuedOperations,
    rateLimitDelays,
  };
}

/**
 * Simulates rate limit backoff strategies.
 */
export function simulateRateLimitBackoff(
  connection: DemoConnection,
  retryCount: number,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  _operationType: "sync" | "api_call" | "webhook_processing", // intentionally unused: included for signature compatibility
): {
  backoffStrategy: "exponential" | "linear" | "adaptive";
  delay: number;
  maxRetries: number;
  estimatedRecoveryTime: number;
  recommendations: string[];
} {
  const random = new SeededRandom(`${connection.id}-backoff-${retryCount}`);

  // Choose backoff strategy based on operation type and retry count
  let backoffStrategy: "exponential" | "linear" | "adaptive";
  let delay: number;
  let maxRetries: number;

  if (retryCount < 2) {
    backoffStrategy = "exponential";
    delay = Math.min(30000, Math.pow(2, retryCount) * 1000); // 1s, 2s, 4s, 8s, 16s, 30s max
    maxRetries = 5;
  } else if (retryCount < 4) {
    backoffStrategy = "linear";
    delay = Math.min(120000, retryCount * 15000); // 15s, 30s, 45s, 60s
    maxRetries = 3;
  } else {
    backoffStrategy = "adaptive";
    delay = Math.min(300000, retryCount * 30000 + random.int(0, 60000)); // 30s increments with jitter
    maxRetries = 1;
  }

  const recommendations: string[] = [];

  if (retryCount === 0) {
    recommendations.push(
      "Initial request failed, retrying with exponential backoff",
    );
  } else if (retryCount < 3) {
    recommendations.push(
      "Continue with exponential backoff - rate limit should reset soon",
    );
  } else if (retryCount < 5) {
    recommendations.push("Switching to linear backoff to reduce API pressure");
    recommendations.push("Consider reducing operation frequency");
  } else {
    recommendations.push("Rate limit persistent - switch to adaptive strategy");
    recommendations.push("Consider pausing operations for this provider");
    recommendations.push("Upgrade API plan if available");
  }

  // Provider-specific recommendations
  if (connection.providerSlug === "github" && retryCount > 2) {
    recommendations.push(
      "GitHub API limit: Consider using authenticated requests for higher limits",
    );
  } else if (connection.providerSlug === "slack" && retryCount > 2) {
    recommendations.push("Slack rate limit: Consider batching operations");
  } else if (connection.providerSlug === "google-workspace" && retryCount > 2) {
    recommendations.push(
      "Google Workspace: Consider implementing exponential backoff with jitter",
    );
  }

  return {
    backoffStrategy,
    delay,
    maxRetries,
    estimatedRecoveryTime: delay + random.int(5000, 15000),
    recommendations,
  };
}

/**
 * Generates rate limit visualization data.
 */
export function generateRateLimitVisualization(connections: DemoConnection[]): {
  global: {
    totalConnections: number;
    limitedConnections: number;
    averageUtilization: number;
    peakUtilization: number;
  };
  providers: Array<{
    providerSlug: string;
    connections: number;
    limitedConnections: number;
    averageUtilization: number;
    endpoints: Array<{
      endpoint: string;
      utilization: number;
      status: "healthy" | "warning" | "critical";
      resetIn?: number;
    }>;
  }>;
  trends: Array<{
    timestamp: string;
    totalRequests: number;
    limitedRequests: number;
    averageLatency: number;
  }>;
} {
  const random = new SeededRandom("rate-limit-visualization");
  const now = Date.now();

  // Global statistics
  const totalConnections = connections.length;
  let limitedConnections = 0;
  let totalUtilization = 0;
  let peakUtilization = 0;

  // Provider breakdown
  type ProviderAggregation = {
    connections: number;
    limitedConnections: number;
    utilizationSum: number;
    peakUtilization: number;
    endpoints: Array<{
      endpoint: string;
      utilization: number;
      status: "healthy" | "warning" | "critical";
      resetIn?: number;
    }>;
  };

  const providers: Record<string, ProviderAggregation> = connections.reduce<
    Record<string, ProviderAggregation>
  >((acc, connection) => {
    if (!acc[connection.providerSlug]) {
      acc[connection.providerSlug] = {
        connections: 0,
        limitedConnections: 0,
        utilizationSum: 0,
        peakUtilization: 0,
        endpoints: [],
      };
    }
    return acc;
  }, {});

  connections.forEach((connection) => {
    const rateLimitState = simulateRateLimitingState(connection);
    const connectionLimited =
      rateLimitState.limitStatus.api.limited ||
      rateLimitState.limitStatus.search.limited ||
      rateLimitState.limitStatus.webhook.limited;

    if (connectionLimited) {
      limitedConnections++;
    }

    // Calculate utilization
    const apiUtilization =
      1 -
      rateLimitState.limitStatus.api.requestsRemaining /
        rateLimitState.currentLimits[0].currentLimit;
    const searchUtilization =
      1 -
      rateLimitState.limitStatus.search.requestsRemaining /
        rateLimitState.currentLimits[1].currentLimit;
    const webhookUtilization =
      1 -
      rateLimitState.limitStatus.webhook.requestsRemaining /
        rateLimitState.currentLimits[2].currentLimit;

    const connectionUtilization = Math.max(
      apiUtilization,
      searchUtilization,
      webhookUtilization,
    );
    totalUtilization += connectionUtilization;
    peakUtilization = Math.max(peakUtilization, connectionUtilization);

    // Update provider stats
    const provider = providers[connection.providerSlug];
    provider.connections++;
    if (connectionLimited) provider.limitedConnections++;
    provider.utilizationSum += connectionUtilization;
    provider.peakUtilization = Math.max(
      provider.peakUtilization,
      connectionUtilization,
    );

    // Add endpoint data
    provider.endpoints = [
      {
        endpoint: "API",
        utilization: apiUtilization,
        status:
          apiUtilization > 0.9
            ? "critical"
            : apiUtilization > 0.7
              ? "warning"
              : "healthy",
        resetIn: rateLimitState.limitStatus.api.limited
          ? rateLimitState.limitStatus.api.resetIn
          : undefined,
      },
      {
        endpoint: "Search",
        utilization: searchUtilization,
        status:
          searchUtilization > 0.9
            ? "critical"
            : searchUtilization > 0.7
              ? "warning"
              : "healthy",
        resetIn: rateLimitState.limitStatus.search.limited
          ? rateLimitState.limitStatus.search.resetIn
          : undefined,
      },
      {
        endpoint: "Webhook",
        utilization: webhookUtilization,
        status:
          webhookUtilization > 0.9
            ? "critical"
            : webhookUtilization > 0.7
              ? "warning"
              : "healthy",
        resetIn: rateLimitState.limitStatus.webhook.limited
          ? rateLimitState.limitStatus.webhook.resetIn
          : undefined,
      },
    ];
  });

  const averageUtilization =
    totalConnections > 0 ? totalUtilization / totalConnections : 0;

  // Generate trend data for the last 24 hours
  const trends = [];
  for (let i = 23; i >= 0; i--) {
    const timestamp = new Date(now - i * 60 * 60 * 1000).toISOString();
    const baseRequests = random.int(1000, 5000);
    const variation = Math.sin(i / 4) * 0.3; // Create some realistic variation
    const totalRequests = Math.floor(baseRequests * (1 + variation));
    const limitedRequests = Math.floor(
      totalRequests * (0.05 + random.next() * 0.15),
    );
    const averageLatency = random.int(200, 800);

    trends.push({
      timestamp,
      totalRequests,
      limitedRequests,
      averageLatency,
    });
  }

  return {
    global: {
      totalConnections,
      limitedConnections,
      averageUtilization,
      peakUtilization,
    },
    providers: Object.entries(providers).map(
      ([providerSlug, stats]: [string, ProviderAggregation]) => ({
        providerSlug,
        connections: stats.connections,
        limitedConnections: stats.limitedConnections,
        averageUtilization:
          stats.connections > 0 ? stats.utilizationSum / stats.connections : 0,
        endpoints: stats.endpoints,
      }),
    ),
    trends,
  };
}

// Re-export constants for convenience
export { MOCK_ORGANIZATIONS };
