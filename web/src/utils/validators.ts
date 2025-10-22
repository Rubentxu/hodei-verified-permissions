/**
 * Validators - Input validation functions
 */

/**
 * Validate email format
 */
export const isValidEmail = (email: string): boolean => {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
};

/**
 * Validate JSON string
 */
export const isValidJSON = (str: string): boolean => {
  try {
    JSON.parse(str);
    return true;
  } catch {
    return false;
  }
};

/**
 * Validate Cedar policy syntax (basic check)
 */
export const isValidCedarPolicy = (policy: string): boolean => {
  const trimmed = policy.trim();
  return (
    (trimmed.startsWith('permit') || trimmed.startsWith('forbid')) &&
    trimmed.endsWith(';')
  );
};

/**
 * Validate policy store ID format
 */
export const isValidPolicyStoreId = (id: string): boolean => {
  // UUID v4 format
  const uuidRegex =
    /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
  return uuidRegex.test(id);
};

/**
 * Validate policy ID format
 */
export const isValidPolicyId = (id: string): boolean => {
  // Alphanumeric with hyphens and underscores
  const policyIdRegex = /^[a-zA-Z0-9_-]+$/;
  return policyIdRegex.length > 0 && policyIdRegex.test(id);
};

/**
 * Validate entity identifier format
 */
export const isValidEntityIdentifier = (identifier: string): boolean => {
  // Format: EntityType::"entity-id"
  const identifierRegex = /^[a-zA-Z_][a-zA-Z0-9_]*::"[^"]*"$/;
  return identifierRegex.test(identifier);
};

/**
 * Validate schema JSON structure
 */
export const isValidSchema = (schema: string): boolean => {
  try {
    const parsed = JSON.parse(schema);
    // Check for required schema fields
    const hasEntityTypes = 'entityTypes' in parsed || Object.values(parsed).some(
      (v: unknown) => typeof v === 'object' && v !== null && 'entityTypes' in (v as object)
    );
    const hasActions = 'actions' in parsed || Object.values(parsed).some(
      (v: unknown) => typeof v === 'object' && v !== null && 'actions' in (v as object)
    );
    return hasEntityTypes || hasActions;
  } catch {
    return false;
  }
};

/**
 * Validate URL format
 */
export const isValidURL = (url: string): boolean => {
  try {
    new URL(url);
    return true;
  } catch {
    return false;
  }
};

/**
 * Validate non-empty string
 */
export const isNonEmptyString = (str: string): boolean => {
  return typeof str === 'string' && str.trim().length > 0;
};

/**
 * Validate required fields in object
 */
export const hasRequiredFields = (
  obj: Record<string, unknown>,
  requiredFields: string[]
): boolean => {
  return requiredFields.every((field) => {
    const value = obj[field];
    return value !== undefined && value !== null && value !== '';
  });
};
