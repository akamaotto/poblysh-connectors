'use client';

import React from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useDemoUser, useDemoTenant, useDemoConnections } from '@/lib/demo/state';

/**
 * DemoNavbar component.
 * Provides navigation and progress tracking for the demo flow.
 */
export function DemoNavbar() {
  const pathname = usePathname();
  const user = useDemoUser();
  const tenant = useDemoTenant();
  const connections = useDemoConnections();

  // Define navigation steps
  const navSteps = [
    {
      path: '/',
      label: 'Login',
      description: 'Get started',
      completed: !!user,
    },
    {
      path: '/tenant',
      label: 'Tenant',
      description: 'Create organization',
      completed: !!tenant,
      disabled: !user,
    },
    {
      path: '/integrations',
      label: 'Integrations',
      description: 'Connect services',
      completed: connections.length > 0,
      disabled: !tenant,
    },
    {
      path: '/signals',
      label: 'Signals',
      description: 'Discover activity',
      completed: false, // Will be updated when signals are generated
      disabled: connections.length === 0,
    },
  ];

  // Calculate progress percentage
  const completedSteps = navSteps.filter(step => step.completed).length;
  const progressPercentage = (completedSteps / navSteps.length) * 100;

  return (
    <nav className="bg-white border-b border-gray-200 sticky top-0 z-40">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between items-center h-16">
          {/* Logo/Title */}
          <div className="flex items-center">
            <Link href="/" className="flex items-center space-x-2">
              <div className="w-8 h-8 bg-black rounded-lg flex items-center justify-center">
                <span className="text-white font-bold text-sm">PC</span>
              </div>
              <span className="text-xl font-semibold text-black">
                Poblysh Connectors Demo
              </span>
            </Link>
          </div>

          {/* User Info */}
          {user && (
            <div className="flex items-center space-x-4">
              <div className="text-sm text-gray-600">
                <span className="font-medium">{user.name}</span>
                {tenant && (
                  <span className="ml-2 text-gray-400">
                    ({tenant.name})
                  </span>
                )}
              </div>
              <div className="w-8 h-8 bg-gray-200 rounded-full flex items-center justify-center">
                <span className="text-gray-600 text-sm font-medium">
                  {user.name.charAt(0).toUpperCase()}
                </span>
              </div>
            </div>
          )}
        </div>

        {/* Progress Bar */}
        {user && (
          <div className="py-4">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium text-gray-900">
                Demo Progress
              </span>
              <span className="text-sm text-gray-500">
                {completedSteps} of {navSteps.length} steps
              </span>
            </div>
            
            {/* Progress bar visual */}
            <div className="w-full bg-gray-200 rounded-full h-2 mb-4">
              <div 
                className="bg-black h-2 rounded-full transition-all duration-300"
                style={{ width: `${progressPercentage}%` }}
              />
            </div>

            {/* Navigation steps */}
            <div className="flex flex-wrap gap-2 sm:gap-4">
              {navSteps.map((step, index) => {
                const isActive = pathname === step.path;
                const isCompleted = step.completed;
                const isDisabled = step.disabled;

                return (
                  <Link
                    key={step.path}
                    href={isDisabled ? '#' : step.path}
                    className={`
                      flex items-center space-x-2 px-3 py-2 rounded-lg text-sm font-medium transition-colors
                      ${isActive 
                        ? 'bg-gray-100 text-black border border-gray-300' 
                        : isCompleted 
                        ? 'bg-gray-50 text-gray-700 border border-gray-200 hover:bg-gray-100'
                        : isDisabled
                        ? 'bg-gray-50 text-gray-400 border border-gray-200 cursor-not-allowed'
                        : 'bg-white text-gray-600 border border-gray-200 hover:bg-gray-50'
                      }
                    `}
                    onClick={(e) => {
                      if (isDisabled) {
                        e.preventDefault();
                      }
                    }}
                  >
                    {/* Step number */}
                    <div className={`
                      w-5 h-5 rounded-full flex items-center justify-center text-xs font-semibold
                      ${isActive 
                        ? 'bg-black text-white' 
                        : isCompleted 
                        ? 'bg-gray-600 text-white'
                        : isDisabled
                        ? 'bg-gray-300 text-gray-500'
                        : 'bg-gray-200 text-gray-600'
                      }
                    `}>
                      {isCompleted ? '✓' : index + 1}
                    </div>
                    
                    {/* Step label and description */}
                    <div>
                      <div className="font-medium">{step.label}</div>
                      <div className="text-xs opacity-75">{step.description}</div>
                    </div>
                  </Link>
                );
              })}
            </div>
          </div>
        )}
      </div>
    </nav>
  );
}