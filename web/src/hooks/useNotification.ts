/**
 * useNotification - Toast notification hook
 */

import { useState, useCallback } from 'react';
import { NotificationMessage } from '../types';

export const useNotification = () => {
  const [notifications, setNotifications] = useState<NotificationMessage[]>([]);

  const addNotification = useCallback(
    (
      message: string,
      type: NotificationMessage['type'] = 'info',
      duration = 5000
    ) => {
      const id = `${Date.now()}-${Math.random()}`;
      const notification: NotificationMessage = {
        id,
        type,
        message,
        duration,
      };

      setNotifications((prev) => [...prev, notification]);

      if (duration > 0) {
        setTimeout(() => {
          removeNotification(id);
        }, duration);
      }

      return id;
    },
    []
  );

  const removeNotification = useCallback((id: string) => {
    setNotifications((prev) => prev.filter((n) => n.id !== id));
  }, []);

  const success = useCallback(
    (message: string, duration?: number) =>
      addNotification(message, 'success', duration),
    [addNotification]
  );

  const error = useCallback(
    (message: string, duration?: number) =>
      addNotification(message, 'error', duration),
    [addNotification]
  );

  const warning = useCallback(
    (message: string, duration?: number) =>
      addNotification(message, 'warning', duration),
    [addNotification]
  );

  const info = useCallback(
    (message: string, duration?: number) =>
      addNotification(message, 'info', duration),
    [addNotification]
  );

  return {
    notifications,
    addNotification,
    removeNotification,
    success,
    error,
    warning,
    info,
  };
};
