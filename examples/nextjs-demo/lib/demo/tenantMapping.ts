/**
 * Tenant Mapping System
 *
 * Handles the bidirectional mapping between frontend-generated tenant IDs
 * and backend connectorsTenantId values for Mode B (real API) operations.
 */

export interface TenantMapping {
  /** Frontend-generated tenant ID (from DemoTenant.id) */
  frontendTenantId: string;
  /** Backend connectorsTenantId (UUID from real API) */
  connectorsTenantId: string;
  /** Creation timestamp */
  createdAt: string;
  /** Tenant name */
  name: string;
}

export interface TenantMappingStorage {
  /** All tenant mappings */
  mappings: TenantMapping[];
  /** Current mode (mock vs real) */
  mode: 'mock' | 'real';
}

const STORAGE_KEY = 'poblysh-tenant-mappings';

/**
 * Loads tenant mappings from localStorage
 */
export function loadTenantMappings(): TenantMappingStorage {
  if (typeof window === 'undefined') {
    return { mappings: [], mode: 'mock' };
  }

  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      return {
        mappings: parsed.mappings || [],
        mode: parsed.mode || 'mock'
      };
    }
  } catch (error) {
    console.warn('Failed to load tenant mappings from localStorage:', error);
  }

  return { mappings: [], mode: 'mock' };
}

/**
 * Saves tenant mappings to localStorage
 */
export function saveTenantMappings(storage: TenantMappingStorage): void {
  if (typeof window === 'undefined') {
    return;
  }

  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(storage));
  } catch (error) {
    console.warn('Failed to save tenant mappings to localStorage:', error);
  }
}

/**
 * Adds a new tenant mapping
 */
export function addTenantMapping(mapping: TenantMapping): void {
  const storage = loadTenantMappings();
  storage.mappings.push(mapping);
  saveTenantMappings(storage);
}

/**
 * Finds a tenant mapping by frontend tenant ID
 */
export function findMappingByFrontendId(frontendId: string): TenantMapping | null {
  const storage = loadTenantMappings();
  return storage.mappings.find(m => m.frontendTenantId === frontendId) || null;
}

/**
 * Finds a tenant mapping by backend connectors tenant ID
 */
export function findMappingByConnectorsId(connectorsId: string): TenantMapping | null {
  const storage = loadTenantMappings();
  return storage.mappings.find(m => m.connectorsTenantId === connectorsId) || null;
}

/**
 * Gets the connectors tenant ID for a given frontend tenant ID
 */
export function getConnectorsTenantId(frontendId: string): string | null {
  const mapping = findMappingByFrontendId(frontendId);
  return mapping ? mapping.connectorsTenantId : null;
}

/**
 * Updates the mode (mock vs real) for tenant mappings
 */
export function setTenantMappingMode(mode: 'mock' | 'real'): void {
  const storage = loadTenantMappings();
  storage.mode = mode;
  saveTenantMappings(storage);
}

/**
 * Clears all tenant mappings
 */
export function clearTenantMappings(): void {
  saveTenantMappings({ mappings: [], mode: 'mock' });
}

/**
 * Migrates existing mock tenant to real API by creating mapping
 */
export function migrateMockTenantToReal(frontendTenant: { id: string; name: string }, connectorsTenantId: string): TenantMapping {
  const mapping: TenantMapping = {
    frontendTenantId: frontendTenant.id,
    connectorsTenantId: connectorsTenantId,
    createdAt: new Date().toISOString(),
    name: frontendTenant.name
  };

  addTenantMapping(mapping);
  return mapping;
}