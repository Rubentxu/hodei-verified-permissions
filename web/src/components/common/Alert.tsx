/**
 * Alert Component - Display alerts/notifications
 */

import React from 'react';
import { cn } from '../../utils/cn';
import { AlertCircle, CheckCircle, AlertTriangle, Info, X } from 'lucide-react';

export interface AlertProps extends React.HTMLAttributes<HTMLDivElement> {
  type?: 'success' | 'error' | 'warning' | 'info';
  title?: string;
  message: string;
  onClose?: () => void;
  closeable?: boolean;
}

const alertStyles = {
  success: {
    container: 'bg-green-50 border-green-200',
    text: 'text-green-800',
    icon: 'text-green-600',
    Icon: CheckCircle,
  },
  error: {
    container: 'bg-red-50 border-red-200',
    text: 'text-red-800',
    icon: 'text-red-600',
    Icon: AlertCircle,
  },
  warning: {
    container: 'bg-yellow-50 border-yellow-200',
    text: 'text-yellow-800',
    icon: 'text-yellow-600',
    Icon: AlertTriangle,
  },
  info: {
    container: 'bg-blue-50 border-blue-200',
    text: 'text-blue-800',
    icon: 'text-blue-600',
    Icon: Info,
  },
};

export const Alert = React.forwardRef<HTMLDivElement, AlertProps>(
  (
    {
      className,
      type = 'info',
      title,
      message,
      onClose,
      closeable = true,
      ...props
    },
    ref
  ) => {
    const style = alertStyles[type];
    const Icon = style.Icon;

    return (
      <div
        ref={ref}
        className={cn(
          'flex items-start gap-3 rounded-lg border p-4',
          style.container,
          className
        )}
        {...props}
      >
        <Icon className={cn('h-5 w-5 flex-shrink-0 mt-0.5', style.icon)} />
        <div className="flex-1">
          {title && (
            <h3 className={cn('font-semibold', style.text)}>{title}</h3>
          )}
          <p className={cn('text-sm', style.text)}>{message}</p>
        </div>
        {closeable && onClose && (
          <button
            onClick={onClose}
            className={cn(
              'flex-shrink-0 inline-flex text-gray-400 hover:text-gray-500 focus:outline-none',
              style.text
            )}
          >
            <X className="h-5 w-5" />
          </button>
        )}
      </div>
    );
  }
);

Alert.displayName = 'Alert';
