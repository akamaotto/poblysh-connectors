import { generateMockSyncJobs, generateMockTokens, generateMockWebhooks } from '../mockData'
import { DemoConnection } from '../types'

describe('Mock Data Generators', () => {
  const mockConnection: DemoConnection = {
    id: 'test-connection-1',
    tenantId: 'test-tenant-1',
    providerSlug: 'github',
    displayName: 'Test GitHub Connection',
    status: 'connected',
    createdAt: '2024-01-01T00:00:00.000Z',
    lastSyncAt: '2024-01-01T12:00:00.000Z',
  }

  describe('generateMockSyncJobs', () => {
    it('generates sync jobs with correct structure', () => {
      const syncJobs = generateMockSyncJobs(mockConnection)

      expect(syncJobs.length).toBeGreaterThan(0) // Should generate some jobs
      expect(syncJobs[0]).toMatchObject({
        tenantId: mockConnection.tenantId,
        connectionId: mockConnection.id,
        providerSlug: mockConnection.providerSlug,
      })
      expect(syncJobs[0]).toHaveProperty('id')
      expect(syncJobs[0]).toHaveProperty('status')
      expect(syncJobs[0]).toHaveProperty('kind')
      expect(syncJobs[0]).toHaveProperty('createdAt')
    })

    it('generates deterministic results', () => {
      const jobs1 = generateMockSyncJobs(mockConnection)
      const jobs2 = generateMockSyncJobs(mockConnection)

      expect(jobs1).toEqual(jobs2)
    })
  })

  describe('generateMockTokens', () => {
    it('generates tokens with correct structure', () => {
      const tokens = generateMockTokens(mockConnection)

      expect(tokens.length).toBeGreaterThan(0) // Should generate some tokens
      expect(tokens[0]).toMatchObject({
        connectionId: mockConnection.id,
        tokenType: expect.stringMatching(/^(oauth|api_key|service_account)$/),
        status: expect.stringMatching(/^(active|expired|revoked|refreshing)$/),
      })
      expect(tokens[0]).toHaveProperty('id')
      expect(tokens[0]).toHaveProperty('scopes')
    })

    it('generates tokens with valid scopes for OAuth tokens', () => {
      const tokens = generateMockTokens(mockConnection)
      const oauthToken = tokens.find(token => token.tokenType === 'oauth')

      expect(oauthToken?.scopes).toBeDefined()
      expect(Array.isArray(oauthToken?.scopes)).toBe(true)
    })
  })

  describe('generateMockWebhooks', () => {
    it('generates webhooks with correct structure', () => {
      const webhooks = generateMockWebhooks(mockConnection)

      expect(webhooks.length).toBeGreaterThan(0) // Should generate some webhooks
      expect(webhooks[0]).toMatchObject({
        tenantId: mockConnection.tenantId,
        connectionId: mockConnection.id,
        providerSlug: mockConnection.providerSlug,
        verified: expect.any(Boolean),
      })
      expect(webhooks[0]).toHaveProperty('id')
      expect(webhooks[0]).toHaveProperty('eventType')
      expect(webhooks[0]).toHaveProperty('payload')
      expect(webhooks[0]).toHaveProperty('createdAt')
    })

    it('generates realistic webhook payloads', () => {
      const webhooks = generateMockWebhooks(mockConnection)

      webhooks.forEach(webhook => {
        expect(typeof webhook.payload).toBe('object')
        expect(webhook.payload).not.toBeNull()
      })
    })
  })
})