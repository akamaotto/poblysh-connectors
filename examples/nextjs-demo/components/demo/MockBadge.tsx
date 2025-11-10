'use client';

import React from 'react';

/**
 * MockBadge component.
 * Displays a prominent indicator that this is a mock demo.
 */
export function MockBadge() {
  return (
    <div className="fixed top-4 left-4 z-50 bg-gray-100 border border-gray-300 text-gray-800 px-3 py-2 rounded-lg shadow-lg flex items-center gap-2">
      <svg 
        className="w-4 h-4" 
        fill="currentColor" 
        viewBox="0 0 20 20"
      >
        <path 
          fillRule="evenodd" 
          d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" 
          clipRule="evenodd" 
        />
      </svg>
      <span className="text-sm font-semibold">Mock Demo Only</span>
    </div>
  );
}