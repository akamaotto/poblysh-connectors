'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useDemoUser, useDemoTenant, useDemoDispatch, setTenant } from '@/lib/demo/state';
import { generateMockTenant, MOCK_ORGANIZATIONS } from '@/lib/demo/mockData';

/**
 * Tenant creation and mapping page.
 * Shows how tenant IDs are created and mapped between Poblysh Core and Connectors service.
 */
export default function TenantPage() {
  const [organizationName, setOrganizationName] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const [showMapping, setShowMapping] = useState(false);
  
  const user = useDemoUser();
  const tenant = useDemoTenant();
  const dispatch = useDemoDispatch();
  const router = useRouter();

  // Redirect to login if no user
  useEffect(() => {
    if (!user) {
      router.push('/');
    }
  }, [user, router]);

  // If tenant already exists, redirect to integrations
  useEffect(() => {
    if (tenant) {
      router.push('/integrations');
    }
  }, [tenant, router]);

  const handleCreateTenant = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!organizationName.trim()) {
      setError('Please enter an organization name');
      return;
    }

    setIsLoading(true);
    setError('');

    try {
      // Simulate API call delay
      await new Promise(resolve => setTimeout(resolve, 1500));
      
      // Create mock tenant and set in state
      const mockTenant = generateMockTenant(organizationName.trim());
      dispatch(setTenant(mockTenant));
      
      // Show mapping visualization briefly before redirecting
      setShowMapping(true);
      setTimeout(() => {
        router.push('/integrations');
      }, 3000);
    } catch {
      setError('Failed to create tenant. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  const handleUseExample = (orgName: string) => {
    setOrganizationName(orgName);
    setError('');
  };

  if (!user) {
    return null;
  }

  return (
    <div className="min-h-screen bg-white py-12 px-4">
      <div className="max-w-4xl mx-auto">
        {/* Header */}
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold text-black mb-4">
            Create Your Organization
          </h1>
          <p className="text-lg text-gray-600">
            Set up your tenant to enable connector integrations
          </p>
        </div>

        <div className="grid md:grid-cols-2 gap-8">
          {/* Creation Form */}
          <div className="bg-white border border-gray-200 rounded-lg p-6">
            <h2 className="text-xl font-semibold text-black mb-6">
              Organization Details
            </h2>
            
            <form onSubmit={handleCreateTenant} className="space-y-6">
              <div>
                <label htmlFor="orgName" className="block text-sm font-medium text-gray-900 mb-2">
                  Organization Name
                </label>
                <input
                  id="orgName"
                  name="orgName"
                  type="text"
                  required
                  value={organizationName}
                  onChange={(e) => setOrganizationName(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-black focus:border-black"
                  placeholder="Enter your organization name"
                  disabled={isLoading}
                />
              </div>

              {error && (
                <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-md">
                  {error}
                </div>
              )}

              <button
                type="submit"
                disabled={isLoading || !organizationName.trim()}
                className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-black hover:bg-gray-800 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-black disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                {isLoading ? (
                  <span className="flex items-center">
                    <svg className="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    Creating Tenant...
                  </span>
                ) : (
                  'Create Organization'
                )}
              </button>
            </form>

            {/* Example Organizations */}
            <div className="mt-6 pt-6 border-t border-gray-200">
              <p className="text-sm text-gray-600 mb-3">Try an example organization:</p>
              <div className="flex flex-wrap gap-2">
                {MOCK_ORGANIZATIONS.slice(0, 4).map((org) => (
                  <button
                    key={org}
                    onClick={() => handleUseExample(org)}
                    className="px-3 py-1 text-xs bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-full transition-colors"
                    disabled={isLoading}
                  >
                    {org}
                  </button>
                ))}
              </div>
            </div>
          </div>

          {/* Educational Content */}
          <div className="space-y-6">
            {/* Tenant Mapping Explanation */}
            <div className="bg-white border border-gray-200 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-black mb-4">
                Understanding Tenant Mapping
              </h3>
              <div className="space-y-4">
                <div>
                  <h4 className="font-medium text-black mb-2">Two-Tenant Architecture</h4>
                  <p className="text-sm text-gray-600">
                    Poblysh uses a dual-tenant system where your organization has two identifiers:
                  </p>
                </div>
                
                <div className="space-y-3">
                  <div className="flex items-start space-x-3">
                    <div className="flex-shrink-0 w-8 h-8 bg-gray-100 rounded-full flex items-center justify-center">
                      <span className="text-black text-xs font-bold">1</span>
                    </div>
                    <div>
                      <p className="text-sm font-medium text-black">Poblysh Core Tenant ID</p>
                      <p className="text-xs text-gray-600">Identifies your organization in the main Poblysh platform</p>
                    </div>
                  </div>
                  
                  <div className="flex items-start space-x-3">
                    <div className="flex-shrink-0 w-8 h-8 bg-gray-100 rounded-full flex items-center justify-center">
                      <span className="text-black text-xs font-bold">2</span>
                    </div>
                    <div>
                      <p className="text-sm font-medium text-black">Connectors Tenant ID</p>
                      <p className="text-xs text-gray-600">Used in X-Tenant-Id header for Connectors API requests</p>
                    </div>
                  </div>
                </div>

                <div className="mt-4 p-3 bg-gray-50 border border-gray-200 rounded-md">
                  <div className="flex">
                    <div className="flex-shrink-0">
                      <svg className="h-5 w-5 text-gray-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                        <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
                      </svg>
                    </div>
                    <div className="ml-3">
                      <p className="text-sm text-gray-700">
                        <strong>In Production:</strong> The Connectors service uses the X-Tenant-Id header to isolate data between organizations.
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            {/* What's Next */}
            <div className="bg-gray-50 border border-gray-200 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-black mb-4">
                What&apos;s Next?
              </h3>
              <div className="space-y-3">
                <div className="flex items-center space-x-3">
                  <svg className="w-5 h-5 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  <span className="text-sm text-gray-900">Connect GitHub and Zoho Cliq integrations</span>
                </div>
                <div className="flex items-center space-x-3">
                  <svg className="w-5 h-5 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  <span className="text-sm text-gray-900">Discover signals from your connected services</span>
                </div>
                <div className="flex items-center space-x-3">
                  <svg className="w-5 h-5 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  <span className="text-sm text-gray-900">Experience signal grounding with cross-provider evidence</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Mapping Visualization */}
        {showMapping && (
          <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div className="bg-white rounded-lg p-8 max-w-2xl w-full mx-4">
              <h3 className="text-xl font-bold text-black mb-6">Tenant Created Successfully!</h3>
              
              <div className="space-y-4">
                <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
                  <h4 className="font-semibold text-black mb-2">Poblysh Core Tenant</h4>
                  <code className="text-sm text-gray-800 break-all">tenant_xxxxxxxxxxxx</code>
                </div>
                
                <div className="flex items-center justify-center">
                  <svg className="w-6 h-6 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
                  </svg>
                </div>
                
                <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
                  <h4 className="font-semibold text-black mb-2">Connectors Service Tenant</h4>
                  <code className="text-sm text-gray-800 break-all">xxxxxxxx-xxxx-4xxx-8xxx-xxxxxxxxxxxx</code>
                </div>
              </div>
              
              <div className="mt-6 p-4 bg-gray-50 rounded-lg">
                <p className="text-sm text-gray-600 text-center">
                  Redirecting you to the integrations page...
                </p>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}