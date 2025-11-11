/**
 * Simple notification system for the demo application.
 * Provides user-friendly notifications for success, error, and warning states.
 */

export type NotificationType = 'success' | 'error' | 'warning' | 'info';

export interface Notification {
  id: string;
  type: NotificationType;
  title: string;
  message: string;
  duration?: number; // Auto-dismiss duration in milliseconds
  persistent?: boolean; // If true, requires manual dismissal
  actions?: Array<{
    label: string;
    action: () => void;
    variant?: 'primary' | 'secondary';
  }>;
}

/**
 * Notification manager for handling user notifications
 */
class NotificationManager {
  private listeners: Set<(notifications: Notification[]) => void> = new Set();
  private notifications: Notification[] = [];

  /**
   * Subscribe to notification changes
   */
  subscribe(listener: (notifications: Notification[]) => void) {
    this.listeners.add(listener);
    listener(this.notifications);

    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * Get current notifications
   */
  getNotifications(): Notification[] {
    return [...this.notifications];
  }

  /**
   * Add a notification
   */
  addNotification(notification: Omit<Notification, 'id'>): string {
    const id = `notification-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    const fullNotification: Notification = { id, ...notification };

    this.notifications.push(fullNotification);
    this.notifyListeners();

    // Auto-dismiss if not persistent
    if (!fullNotification.persistent && fullNotification.duration !== 0) {
      const duration = fullNotification.duration || this.getDefaultDuration(notification.type);
      setTimeout(() => {
        this.removeNotification(id);
      }, duration);
    }

    return id;
  }

  /**
   * Remove a notification
   */
  removeNotification(id: string): boolean {
    const index = this.notifications.findIndex(n => n.id === id);
    if (index !== -1) {
      this.notifications.splice(index, 1);
      this.notifyListeners();
      return true;
    }
    return false;
  }

  /**
   * Clear all notifications
   */
  clearNotifications(): void {
    this.notifications = [];
    this.notifyListeners();
  }

  /**
   * Convenience methods for different notification types
   */
  success(message: string, title?: string): string {
    return this.addNotification({
      type: 'success',
      title: title || 'Success',
      message,
      duration: 4000
    });
  }

  error(message: string, title?: string, persistent?: boolean): string {
    return this.addNotification({
      type: 'error',
      title: title || 'Error',
      message,
      persistent: persistent ?? true,
      duration: 0 // Errors are persistent by default
    });
  }

  warning(message: string, title?: string): string {
    return this.addNotification({
      type: 'warning',
      title: title || 'Warning',
      message,
      duration: 6000
    });
  }

  info(message: string, title?: string): string {
    return this.addNotification({
      type: 'info',
      title: title || 'Information',
      message,
      duration: 5000
    });
  }

  private notifyListeners(): void {
    const currentNotifications = [...this.notifications];
    this.listeners.forEach(listener => listener(currentNotifications));
  }

  private getDefaultDuration(type: NotificationType): number {
    switch (type) {
      case 'success': return 3000;
      case 'error': return 0; // Persistent
      case 'warning': return 5000;
      case 'info': return 4000;
      default: return 4000;
    }
  }
}

// Global notification manager instance
export const notificationManager = new NotificationManager();

/**
 * React hook for using notifications
 */
export function useNotifications() {
  const [notifications, setNotifications] = React.useState<Notification[]>([]);

  React.useEffect(() => {
    return notificationManager.subscribe(setNotifications);
  }, []);

  return {
    notifications,
    success: notificationManager.success.bind(notificationManager),
    error: notificationManager.error.bind(notificationManager),
    warning: notificationManager.warning.bind(notificationManager),
    info: notificationManager.info.bind(notificationManager),
    remove: notificationManager.removeNotification.bind(notificationManager),
    clear: notificationManager.clearNotifications.bind(notificationManager)
  };
}

// React import for the hook
import React from 'react';