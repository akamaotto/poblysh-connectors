'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useDemoUser, useDemoTenant, useDemoConnections, useDemoSignals, useDemoDispatch, setUser, resetState } from '@/lib/demo/state';
import { generateMockUser } from '@/lib/demo/mockData';

/**
 * Landing page with mock login functionality.
 * This is the entry point for the Poblysh Connectors demo.
 */
export default function Home() {
  const [email, setEmail] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  
  const currentUser = useDemoUser();
  const currentTenant = useDemoTenant();
  const connections = useDemoConnections();
  const signals = useDemoSignals();
  const dispatch = useDemoDispatch();
  const router = useRouter();

  // Intelligent redirect logic for existing sessions
  useEffect(() => {
    if (currentUser) {
      if (!currentTenant) {
        router.push('/tenant');
      } else if (connections.length === 0) {
        router.push('/integrations');
      } else if (signals.length === 0) {
        router.push('/integrations'); // Redirect to integrations to prompt for signal scanning
      } else {
        router.push('/signals');
      }
    }
  }, [currentUser, currentTenant, connections, signals, router]);

  if (currentUser) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-white px-4">
        <div className="max-w-md w-full space-y-8">
          {/* Header */}
          <div className="text-center">
            <div className="mx-auto h-12 w-12 bg-black rounded-lg flex items-center justify-center">
              <span className="text-white font-bold text-lg">PC</span>
            </div>
            <h2 className="mt-6 text-3xl font-bold text-black">
              Welcome Back, {currentUser.name}!
            </h2>
            <p className="mt-2 text-sm text-gray-600">
              Continue where you left off or start fresh
            </p>
          </div>

          {/* Continue Options */}
          <div className="mt-8 bg-white border border-gray-200 rounded-lg p-8 space-y-4">
            <p className="text-sm text-gray-700 text-center">
              You have an active demo session. Choose what to do next:
            </p>

            <button
              onClick={() => {
                // The redirect logic will handle this in useEffect
                router.refresh(); // Trigger re-evaluation of redirect logic
              }}
              className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-black hover:bg-gray-800 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-black transition-colors"
            >
              Continue Demo Session
            </button>

            <button
              onClick={() => {
                dispatch(resetState());
                setEmail('');
                setError('');
              }}
              className="w-full flex justify-center py-2 px-4 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-black transition-colors"
            >
              Reset Demo Flow
            </button>
          </div>

          {/* Current Status */}
          <div className="bg-gray-50 rounded-lg p-6">
            <h3 className="text-lg font-semibold text-black mb-4">Current Session Status</h3>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-600">User:</span>
                <span className="text-gray-900">{currentUser.email}</span>
              </div>
              {currentTenant && (
                <div className="flex justify-between">
                  <span className="text-gray-600">Tenant:</span>
                  <span className="text-gray-900">{currentTenant.name}</span>
                </div>
              )}
              <div className="flex justify-between">
                <span className="text-gray-600">Connections:</span>
                <span className="text-gray-900">{connections.length} connected</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Signals:</span>
                <span className="text-gray-900">{signals.length} discovered</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  }

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!email.trim()) {
      setError('Please enter an email address');
      return;
    }

    // Basic email validation
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!emailRegex.test(email)) {
      setError('Please enter a valid email address');
      return;
    }

    setIsLoading(true);
    setError('');

    try {
      // Simulate API call delay
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      // Create mock user and set in state
      const mockUser = generateMockUser(email.toLowerCase());
      dispatch(setUser(mockUser));
      
      // Navigate to tenant setup with delay to avoid render issues
      setTimeout(() => {
        router.push('/tenant');
      }, 100);
    } catch {
      setError('Login failed. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-white px-4">
      <div className="max-w-md w-full space-y-8">
        {/* Header */}
        <div className="text-center">
          <div className="mx-auto h-12 w-12 bg-black rounded-lg flex items-center justify-center">
            <span className="text-white font-bold text-lg">PC</span>
          </div>
          <h2 className="mt-6 text-3xl font-bold text-black">
            Poblysh Connectors Demo
          </h2>
          <p className="mt-2 text-sm text-gray-600">
            Experience the complete Connectors integration flow
          </p>
        </div>

        {/* Login Form */}
        <div className="mt-8 bg-white border border-gray-200 rounded-lg p-8">
          <form className="space-y-6" onSubmit={handleLogin}>
            <div>
              <label htmlFor="email" className="block text-sm font-medium text-gray-900 mb-2">
                Email Address
              </label>
              <input
                id="email"
                name="email"
                type="email"
                autoComplete="email"
                required
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-black focus:border-black"
                placeholder="Enter your email"
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
              disabled={isLoading}
              className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-black hover:bg-gray-800 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-black disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {isLoading ? (
                <span className="flex items-center">
                  <svg className="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                  Signing in...
                </span>
              ) : (
                'Continue to Demo'
              )}
            </button>
          </form>

          {/* Educational Info */}
          <div className="mt-6 p-4 bg-gray-50 border border-gray-200 rounded-md">
            <div className="flex">
              <div className="flex-shrink-0">
                <svg className="h-5 w-5 text-gray-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                  <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
                </svg>
              </div>
              <div className="ml-3">
                <p className="text-sm text-gray-700">
                  <strong>Mock Authentication:</strong> This is a demo login. Any email address will work to explore the Connectors integration flow.
                </p>
              </div>
            </div>
          </div>
        </div>

        {/* Demo Features */}
        <div className="bg-gray-50 rounded-lg p-6 space-y-4">
          <h3 className="text-lg font-semibold text-black mb-4">What You&apos;ll Experience</h3>
          
          <div className="space-y-3">
            <div className="flex items-start space-x-3">
              <div className="flex-shrink-0 w-6 h-6 bg-gray-200 rounded-full flex items-center justify-center">
                <span className="text-black text-xs font-semibold">1</span>
              </div>
              <div>
                <p className="text-sm font-medium text-black">Tenant Setup</p>
                <p className="text-xs text-gray-600">Create your organization and understand tenant mapping</p>
              </div>
            </div>
            
            <div className="flex items-start space-x-3">
              <div className="flex-shrink-0 w-6 h-6 bg-gray-200 rounded-full flex items-center justify-center">
                <span className="text-black text-xs font-semibold">2</span>
              </div>
              <div>
                <p className="text-sm font-medium text-black">Connect Integrations</p>
                <p className="text-xs text-gray-600">Set up GitHub and Zoho Cliq connections</p>
              </div>
            </div>
            
            <div className="flex items-start space-x-3">
              <div className="flex-shrink-0 w-6 h-6 bg-gray-200 rounded-full flex items-center justify-center">
                <span className="text-black text-xs font-semibold">3</span>
              </div>
              <div>
                <p className="text-sm font-medium text-black">Discover Signals</p>
                <p className="text-xs text-gray-600">Scan for and explore signals from connected services</p>
              </div>
            </div>
            
            <div className="flex items-start space-x-3">
              <div className="flex-shrink-0 w-6 h-6 bg-gray-200 rounded-full flex items-center justify-center">
                <span className="text-black text-xs font-semibold">4</span>
              </div>
              <div>
                <p className="text-sm font-medium text-black">Signal Grounding</p>
                <p className="text-xs text-gray-600">See how evidence from multiple providers strengthens signals</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}