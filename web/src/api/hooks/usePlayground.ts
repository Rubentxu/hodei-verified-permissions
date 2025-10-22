/**
 * usePlayground - React Query hooks for Playground API
 */

import { useMutation } from '@tanstack/react-query';
import { AuthorizationResponse, ValidationResponse } from '../../types';

// Mock API client
const mockApiClient = {
  testAuthorization: async (data: {
    principal?: string;
    action?: string;
    resource?: string;
    context?: Record<string, unknown>;
    policies: string[];
    entities: unknown[];
  }): Promise<AuthorizationResponse> => {
    await new Promise((resolve) => setTimeout(resolve, 800));
    
    // Simple mock logic: if any policy starts with "permit", allow
    const hasPermit = data.policies.some((p) => p.trim().startsWith('permit'));
    
    return {
      decision: hasPermit ? 'ALLOW' : 'DENY',
      determiningPolicies: hasPermit ? [data.policies[0]] : [],
      errors: [],
    };
  },

  validatePolicy: async (data: {
    policy: string;
    schema?: string;
  }): Promise<ValidationResponse> => {
    await new Promise((resolve) => setTimeout(resolve, 500));
    
    const isValid = data.policy.trim().startsWith('permit') || data.policy.trim().startsWith('forbid');
    
    return {
      isValid,
      errors: isValid ? [] : ['Invalid Cedar policy syntax'],
      warnings: [],
    };
  },
};

/**
 * Hook to test authorization
 */
export const useTestAuthorization = () => {
  return useMutation({
    mutationFn: (data: {
      principal?: string;
      action?: string;
      resource?: string;
      context?: Record<string, unknown>;
      policies: string[];
      entities: unknown[];
    }) => mockApiClient.testAuthorization(data),
  });
};

/**
 * Hook to validate policy
 */
export const useValidatePolicy = () => {
  return useMutation({
    mutationFn: (data: { policy: string; schema?: string }) =>
      mockApiClient.validatePolicy(data),
  });
};
