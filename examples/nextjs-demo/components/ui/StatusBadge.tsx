'use client';

import { CheckCircle2, XCircle, AlertTriangle, Clock, RefreshCw } from 'lucide-react';

/**
 * StatusBadge component.
 * Displays a standardized status indicator with appropriate colors and icons.
 */
export interface StatusBadgeProps {
  status: 'success' | 'error' | 'warning' | 'pending' | 'loading';
  children: React.ReactNode;
  className?: string;
  showIcon?: boolean;
}

export function StatusBadge({
  status,
  children,
  className = '',
  showIcon = true,
}: StatusBadgeProps) {
  const getStatusConfig = () => {
    switch (status) {
      case 'success':
        return {
          bgColor: 'bg-green-100',
          textColor: 'text-green-800',
          borderColor: 'border-green-200',
          icon: <CheckCircle2 className="w-3 h-3" />,
        };
      case 'error':
        return {
          bgColor: 'bg-red-100',
          textColor: 'text-red-800',
          borderColor: 'border-red-200',
          icon: <XCircle className="w-3 h-3" />,
        };
      case 'warning':
        return {
          bgColor: 'bg-yellow-100',
          textColor: 'text-yellow-800',
          borderColor: 'border-yellow-200',
          icon: <AlertTriangle className="w-3 h-3" />,
        };
      case 'pending':
        return {
          bgColor: 'bg-gray-100',
          textColor: 'text-gray-800',
          borderColor: 'border-gray-200',
          icon: <Clock className="w-3 h-3" />,
        };
      case 'loading':
        return {
          bgColor: 'bg-blue-100',
          textColor: 'text-blue-800',
          borderColor: 'border-blue-200',
          icon: <RefreshCw className="w-3 h-3 animate-spin" />,
        };
      default:
        return {
          bgColor: 'bg-gray-100',
          textColor: 'text-gray-800',
          borderColor: 'border-gray-200',
          icon: <Clock className="w-3 h-3" />,
        };
    }
  };

  const config = getStatusConfig();

  return (
    <span
      className={`
        inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium
        border ${config.bgColor} ${config.textColor} ${config.borderColor}
        ${className}
      `}
      role="status"
      aria-label={`Status: ${status}`}
    >
      {showIcon && config.icon}
      {children}
    </span>
  );
}