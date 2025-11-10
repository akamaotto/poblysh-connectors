/**
 * ID generation utilities for the demo.
 * Provides deterministic ID generation based on seeds.
 */

/**
 * Generates a deterministic ID based on a seed.
 * Uses a simple hash function to create consistent IDs.
 */
export function generateId(seed: string): string {
  // Simple hash function for consistent ID generation
  let hash = 0;
  for (let i = 0; i < seed.length; i++) {
    const char = seed.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash; // Convert to 32-bit integer
  }
  
  // Convert to positive and format as hex
  return Math.abs(hash).toString(16).padStart(8, '0');
}

/**
 * Generates a UUID-like ID based on a seed.
 * Creates more realistic-looking IDs for the demo.
 */
export function generateUuid(seed: string): string {
  // Create multiple different hashes from the seed to ensure uniqueness
  const hash1 = generateId(seed);
  const hash2 = generateId(seed + '_suffix1');
  const hash3 = generateId(seed + '_suffix2');
  const timestamp = Date.now().toString(16).padStart(12, '0');

  // Format as xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
  // Using different hash parts to avoid repetition
  return [
    hash1.substring(0, 8),
    hash2.substring(0, 4),
    '4' + hash3.substring(1, 4),
    '8' + hash1.substring(2, 4),
    timestamp + hash2.substring(0, 4)
  ].join('-');
}

/**
 * Generates a tenant ID pair based on an organization name.
 * Returns both the Poblysh tenant ID and Connectors tenant ID.
 */
export function generateTenantIds(organizationName: string): {
  tenantId: string;
  connectorsTenantId: string;
} {
  const baseSeed = organizationName.toLowerCase().replace(/[^a-z0-9]/g, '');

  return {
    tenantId: 'tenant_' + generateId(baseSeed + '_poblysh'),
    connectorsTenantId: generateUuid(baseSeed + '_connectors'),
  };
}

/**
 * Generates a sync job ID based on connection and timestamp.
 */
export function generateSyncJobId(connectionId: string, timestamp?: number): string {
  const ts = timestamp || Date.now();
  return generateUuid(`syncjob_${connectionId}_${ts}`);
}

/**
 * Generates a webhook event ID based on provider and event type.
 */
export function generateWebhookId(providerSlug: string, eventType: string, timestamp?: number): string {
  const ts = timestamp || Date.now();
  return generateUuid(`webhook_${providerSlug}_${eventType}_${ts}`);
}

/**
 * Generates a token ID based on connection and token type.
 */
export function generateTokenId(connectionId: string, tokenType: string): string {
  return generateUuid(`token_${connectionId}_${tokenType}`);
}

/**
 * Generates a rate limit ID based on connection and endpoint.
 */
export function generateRateLimitId(connectionId: string, endpoint: string): string {
  const endpointSafe = endpoint.replace(/[^a-zA-Z0-9]/g, '_');
  return generateUuid(`ratelimit_${connectionId}_${endpointSafe}`);
}