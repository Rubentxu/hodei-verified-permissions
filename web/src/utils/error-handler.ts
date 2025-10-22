/**
 * Error Handler - Centralized error handling
 */

export class AppError extends Error {
  constructor(
    public code: string,
    message: string,
    public statusCode: number = 500
  ) {
    super(message);
    this.name = 'AppError';
  }
}

export class ValidationError extends AppError {
  constructor(message: string) {
    super('VALIDATION_ERROR', message, 400);
    this.name = 'ValidationError';
  }
}

export class NotFoundError extends AppError {
  constructor(message: string) {
    super('NOT_FOUND', message, 404);
    this.name = 'NotFoundError';
  }
}

export class UnauthorizedError extends AppError {
  constructor(message: string = 'Unauthorized') {
    super('UNAUTHORIZED', message, 401);
    this.name = 'UnauthorizedError';
  }
}

export class ForbiddenError extends AppError {
  constructor(message: string = 'Forbidden') {
    super('FORBIDDEN', message, 403);
    this.name = 'ForbiddenError';
  }
}

export class NetworkError extends AppError {
  constructor(message: string = 'Network error') {
    super('NETWORK_ERROR', message, 0);
    this.name = 'NetworkError';
  }
}

/**
 * Handle API errors and return user-friendly message
 */
export const handleApiError = (error: unknown): string => {
  if (error instanceof AppError) {
    return error.message;
  }

  if (error instanceof Error) {
    // gRPC errors
    if (error.message.includes('UNAVAILABLE')) {
      return 'Server is unavailable. Please try again later.';
    }
    if (error.message.includes('UNAUTHENTICATED')) {
      return 'Authentication required. Please log in.';
    }
    if (error.message.includes('PERMISSION_DENIED')) {
      return 'You do not have permission to perform this action.';
    }
    if (error.message.includes('INVALID_ARGUMENT')) {
      return 'Invalid input provided.';
    }
    if (error.message.includes('NOT_FOUND')) {
      return 'Resource not found.';
    }

    return error.message;
  }

  return 'An unexpected error occurred';
};

/**
 * Log error with context
 */
export const logError = (
  error: unknown,
  context: Record<string, unknown> = {}
): void => {
  const timestamp = new Date().toISOString();
  const errorMessage =
    error instanceof Error ? error.message : String(error);

  console.error(`[${timestamp}] Error:`, {
    message: errorMessage,
    stack: error instanceof Error ? error.stack : undefined,
    context,
  });

  // In production, send to error tracking service
  if (process.env.NODE_ENV === 'production') {
    // TODO: Send to Sentry, LogRocket, etc.
  }
};

/**
 * Retry function with exponential backoff
 */
export const retryWithBackoff = async <T,>(
  fn: () => Promise<T>,
  maxRetries: number = 3,
  baseDelay: number = 1000
): Promise<T> => {
  let lastError: Error | null = null;

  for (let i = 0; i < maxRetries; i++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));

      if (i < maxRetries - 1) {
        const delay = baseDelay * Math.pow(2, i);
        await new Promise((resolve) => setTimeout(resolve, delay));
      }
    }
  }

  throw lastError || new Error('Max retries exceeded');
};
