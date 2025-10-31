// API Console Types

export interface ApiRequest {
  id: string;
  name: string;
  method: HttpMethod;
  url: string;
  headers: Record<string, string>;
  queryParams: QueryParam[];
  body?: string | FormData;
  bodyType: BodyType;
  authentication?: AuthenticationConfig;
  timeout?: number;
  createdAt: string;
  updatedAt: string;
  tags?: string[];
  description?: string;
}

export interface HttpResponse {
  id: string;
  requestId: string;
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: string;
  size: number;
  duration: number;
  timestamp: string;
  isError: boolean;
  error?: ApiError;
  cached?: boolean;
}

export interface ApiError {
  type: 'network' | 'timeout' | 'validation' | 'authentication' | 'server' | 'unknown';
  message: string;
  details?: string;
  code?: string | number;
  stack?: string;
}

export interface ApiCollection {
  id: string;
  name: string;
  description: string;
  baseUrl: string;
  version: string;
  requests: ApiRequest[];
  environments: Environment[];
  variables: Record<string, string>;
  auth?: AuthenticationConfig;
  createdAt: string;
  updatedAt: string;
  tags?: string[];
}

export interface Environment {
  id: string;
  name: string;
  baseUrl?: string;
  variables: Record<string, string>;
  headers: Record<string, string>;
  authentication?: AuthenticationConfig;
  isActive: boolean;
}

export interface QueryParam {
  key: string;
  value: string;
  enabled: boolean;
  description?: string;
}

export interface AuthenticationConfig {
  type: AuthType;
  credentials: Record<string, any>;
  enabled: boolean;
  testResult?: AuthTestResult;
}

export interface AuthTestResult {
  success: boolean;
  message: string;
  timestamp: string;
  userInfo?: any;
  permissions?: string[];
}

export interface ApiDocumentation {
  id: string;
  title: string;
  description: string;
  version: string;
  baseUrl: string;
  endpoints: ApiEndpoint[];
  schemas: ApiSchema[];
  examples: ApiExample[];
  tags: ApiTag[];
  generatedAt: string;
}

export interface ApiEndpoint {
  id: string;
  path: string;
  method: HttpMethod;
  summary: string;
  description: string;
  parameters: ApiParameter[];
  requestBody?: ApiRequestBody;
  responses: ApiEndpointResponse[];
  tags: string[];
  deprecated?: boolean;
  security?: SecurityScheme[];
}

export interface ApiParameter {
  name: string;
  in: ParameterLocation;
  required: boolean;
  type: string;
  description?: string;
  example?: any;
  schema?: ApiSchema;
}

export interface ApiRequestBody {
  required: boolean;
  content: Record<string, ApiMediaType>;
}

export interface ApiMediaType {
  schema: ApiSchema;
  example?: any;
  examples?: Record<string, ApiExample>;
}

export interface ApiEndpointResponse {
  statusCode: number;
  description: string;
  content: Record<string, ApiMediaType>;
}

export interface ApiSchema {
  id: string;
  name: string;
  type: string;
  description?: string;
  properties?: Record<string, ApiProperty>;
  required?: string[];
  enum?: any[];
  items?: ApiSchema;
  additionalProperties?: boolean;
  example?: any;
}

export interface ApiProperty {
  type: string;
  description?: string;
  format?: string;
  example?: any;
  enum?: any[];
  items?: ApiSchema;
}

export interface ApiExample {
  id: string;
  name: string;
  summary?: string;
  value: any;
  description?: string;
}

export interface ApiTag {
  name: string;
  description?: string;
}

export interface SecurityScheme {
  type: SecuritySchemeType;
  name?: string;
  in?: string;
  scheme?: string;
  bearerFormat?: string;
  description?: string;
}

export interface CodeSample {
  language: ProgrammingLanguage;
  code: string;
  description?: string;
}

export interface PerformanceMetrics {
  requestId: string;
  duration: number;
  size: number;
  throughput: number;
  errorRate: number;
  responseTime: number;
  timestamp: string;
}

export interface LoadTestConfig {
  name: string;
  url: string;
  method: HttpMethod;
  concurrency: number;
  iterations: number;
  rampUpTime?: number;
  payload?: string;
  headers?: Record<string, string>;
  authentication?: AuthenticationConfig;
}

export interface LoadTestResult {
  id: string;
  config: LoadTestConfig;
  metrics: LoadTestMetrics;
  timestamp: string;
  duration: number;
  error?: string;
}

export interface LoadTestMetrics {
  totalRequests: number;
  successfulRequests: number;
  failedRequests: number;
  averageResponseTime: number;
  minResponseTime: number;
  maxResponseTime: number;
  throughput: number;
  errorRate: number;
  requestsPerSecond: number;
  responseTimePercentiles: {
    p50: number;
    p90: number;
    p95: number;
    p99: number;
  };
}

export interface ApiHistoryEntry {
  id: string;
  request: ApiRequest;
  response?: HttpResponse;
  timestamp: string;
  duration?: number;
  isFavorite: boolean;
  tags?: string[];
}

export interface ApiConsoleState {
  currentCollection?: ApiCollection;
  currentEnvironment?: Environment;
  selectedRequest?: ApiRequest;
  requests: ApiRequest[];
  responses: Record<string, HttpResponse>;
  collections: ApiCollection[];
  documentation?: ApiDocumentation;
  history: ApiHistoryEntry[];
  performanceMetrics: PerformanceMetrics[];
  isLoading: boolean;
  error?: string;
  showDocumentation: boolean;
  showCodeSamples: boolean;
  selectedLanguage: ProgrammingLanguage;
  autoSave: boolean;
  theme: 'light' | 'dark' | 'system';
}

export interface ApiConsoleFilters {
  searchQuery: string;
  tags: string[];
  methods: HttpMethod[];
  environments: string[];
  favoritesOnly: boolean;
  dateRange?: {
    from?: string;
    to?: string;
  };
}

export interface ApiConsoleSort {
  field: 'name' | 'method' | 'url' | 'timestamp' | 'duration' | 'status';
  order: 'asc' | 'desc';
}

export interface ApiImportExport {
  type: 'import' | 'export';
  format: 'postman' | 'openapi' | 'insomnia' | 'curl';
  data: string;
  validationResults?: ValidationResult[];
  warnings?: string[];
  errors?: string[];
}

export interface ValidationResult {
  field: string;
  status: 'valid' | 'invalid' | 'warning';
  message: string;
  suggestedValue?: any;
}

// Enums
export type HttpMethod = 
  | 'GET' 
  | 'POST' 
  | 'PUT' 
  | 'PATCH' 
  | 'DELETE' 
  | 'HEAD' 
  | 'OPTIONS' 
  | 'TRACE';

export type BodyType = 
  | 'none' 
  | 'json' 
  | 'xml' 
  | 'form-data' 
  | 'x-www-form-urlencoded' 
  | 'raw' 
  | 'file';

export type AuthType = 
  | 'none' 
  | 'bearer' 
  | 'basic' 
  | 'api-key' 
  | 'oauth2' 
  | 'jwt' 
  | 'aws-signature' 
  | 'digest';

export type ParameterLocation = 
  | 'path' 
  | 'query' 
  | 'header' 
  | 'cookie' 
  | 'body';

export type SecuritySchemeType = 
  | 'http' 
  | 'apiKey' 
  | 'oauth2' 
  | 'openIdConnect';

export type ProgrammingLanguage = 
  | 'javascript' 
  | 'typescript' 
  | 'python' 
  | 'java' 
  | 'curl' 
  | 'csharp' 
  | 'php' 
  | 'ruby' 
  | 'go' 
  | 'swift' 
  | 'kotlin' 
  | 'rust';

// Constants
export const HTTP_METHOD_COLORS: Record<HttpMethod, string> = {
  GET: 'bg-green-100 text-green-800',
  POST: 'bg-blue-100 text-blue-800',
  PUT: 'bg-yellow-100 text-yellow-800',
  PATCH: 'bg-yellow-100 text-yellow-800',
  DELETE: 'bg-red-100 text-red-800',
  HEAD: 'bg-gray-100 text-gray-800',
  OPTIONS: 'bg-purple-100 text-purple-800',
  TRACE: 'bg-indigo-100 text-indigo-800'
};

export const HTTP_STATUS_COLORS: Record<number, string> = {
  200: 'text-green-600',
  201: 'text-green-600',
  204: 'text-green-600',
  400: 'text-yellow-600',
  401: 'text-orange-600',
  403: 'text-orange-600',
  404: 'text-red-600',
  409: 'text-red-600',
  422: 'text-yellow-600',
  500: 'text-red-600',
  502: 'text-red-600',
  503: 'text-red-600'
};

// Helper functions
export const getMethodColor = (method: HttpMethod): string => 
  HTTP_METHOD_COLORS[method] || 'bg-gray-100 text-gray-800';

export const getStatusColor = (status: number): string => {
  if (status >= 200 && status < 300) return HTTP_STATUS_COLORS[200];
  if (status >= 400 && status < 500) return HTTP_STATUS_COLORS[400];
  if (status >= 500) return HTTP_STATUS_COLORS[500];
  return 'text-gray-600';
};

export const formatDuration = (ms: number): string => {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  return `${(ms / 60000).toFixed(1)}m`;
};

export const formatSize = (bytes: number): string => {
  if (bytes < 1024) return `${bytes}B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)}MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)}GB`;
};

export const getErrorType = (error: any): ApiError['type'] => {
  if (error.name === 'NetworkError') return 'network';
  if (error.name === 'TimeoutError') return 'timeout';
  if (error.status === 401) return 'authentication';
  if (error.status && error.status >= 500) return 'server';
  return 'unknown';
};