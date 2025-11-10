'use client';

import { ReactNode } from 'react';
import { Clock, Inbox, Search, AlertTriangle } from 'lucide-react';

/**
 * EmptyState component.
 * Displays a standardized empty state with icon, title, and description.
 */
export interface EmptyStateProps {
  type?: 'no-data' | 'search' | 'error' | 'loading';
  title?: string;
  description?: string;
  icon?: ReactNode;
  action?: ReactNode;
  className?: string;
}

export function EmptyState({
  type = 'no-data',
  title,
  description,
  icon,
  action,
  className = '',
}: EmptyStateProps) {
  const getDefaultContent = () => {
    switch (type) {
      case 'no-data':
        return {
          icon: icon || <Inbox className="w-8 h-8 text-gray-400" />,
          title: title || 'No data found',
          description: description || 'There are no items to display at this time.',
        };
      case 'search':
        return {
          icon: icon || <Search className="w-8 h-8 text-gray-400" />,
          title: title || 'No results found',
          description: description || 'Try adjusting your search criteria or filters.',
        };
      case 'error':
        return {
          icon: icon || <AlertTriangle className="w-8 h-8 text-red-400" />,
          title: title || 'Something went wrong',
          description: description || 'An error occurred while loading the data.',
        };
      case 'loading':
        return {
          icon: icon || <Clock className="w-8 h-8 text-blue-400" />,
          title: title || 'Loading',
          description: description || 'Please wait while we fetch the data.',
        };
      default:
        return {
          icon: icon || <Inbox className="w-8 h-8 text-gray-400" />,
          title: title || 'No data found',
          description: description || 'There are no items to display at this time.',
        };
    }
  };

  const { icon: defaultIcon, title: defaultTitle, description: defaultDescription } = getDefaultContent();

  return (
    <div
      className={`flex flex-col items-center justify-center p-8 text-center ${className}`}
      role="status"
      aria-live="polite"
    >
      <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mb-4">
        {defaultIcon}
      </div>
      <h3 className="text-lg font-medium text-gray-900 mb-2">
        {defaultTitle}
      </h3>
      <p className="text-gray-600 mb-4 max-w-md">
        {defaultDescription}
      </p>
      {action && (
        <div className="mt-4">
          {action}
        </div>
      )}
    </div>
  );
}