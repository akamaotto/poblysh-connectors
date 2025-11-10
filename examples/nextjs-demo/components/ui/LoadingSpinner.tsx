'use client';

import { RefreshCw } from 'lucide-react';

/**
 * LoadingSpinner component.
 * Displays a standardized loading indicator with customizable size and text.
 */
export interface LoadingSpinnerProps {
  size?: 'sm' | 'md' | 'lg';
  text?: string;
  className?: string;
}

export function LoadingSpinner({
  size = 'md',
  text,
  className = '',
}: LoadingSpinnerProps) {
  const getSizeClasses = () => {
    switch (size) {
      case 'sm':
        return 'w-4 h-4';
      case 'md':
        return 'w-6 h-6';
      case 'lg':
        return 'w-8 h-8';
      default:
        return 'w-6 h-6';
    }
  };

  return (
    <div
      className={`flex items-center gap-2 ${className}`}
      role="status"
      aria-label="Loading"
      aria-live="polite"
    >
      <RefreshCw
        className={`${getSizeClasses()} animate-spin text-gray-500`}
        aria-hidden="true"
      />
      {text && (
        <span className="text-sm text-gray-600">{text}</span>
      )}
    </div>
  );
}