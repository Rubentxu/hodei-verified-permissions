// Configuration and Settings Types

export interface UserPreferences {
  id: string;
  userId: string;
  theme: 'light' | 'dark' | 'system';
  language: 'en' | 'es' | 'fr' | 'de';
  timezone: string;
  dateFormat: 'MM/dd/yyyy' | 'dd/MM/yyyy' | 'yyyy-MM-dd';
  timeFormat: '12h' | '24h';
  notifications: NotificationSettings;
  dashboard: DashboardPreferences;
  editor: EditorPreferences;
  playground: PlaygroundPreferences;
  createdAt: string;
  updatedAt: string;
}

export interface NotificationSettings {
  emailNotifications: boolean;
  pushNotifications: boolean;
  inAppNotifications: boolean;
  reminderNotifications: boolean;
  errorAlerts: boolean;
  successNotifications: boolean;
  weeklyReports: boolean;
  policyChangeAlerts: boolean;
  systemMaintenance: boolean;
}

export interface DashboardPreferences {
  layout: 'grid' | 'list' | 'cards';
  defaultView: 'overview' | 'policies' | 'templates' | 'activity';
  widgetsVisible: DashboardWidget[];
  refreshInterval: number; // seconds
  compactMode: boolean;
  showWelcomeGuide: boolean;
  showTips: boolean;
}

export interface DashboardWidget {
  id: string;
  type: 'metrics' | 'activity' | 'recent-changes' | 'policy-health' | 'authorization-requests' | 'policy-coverage';
  position: { x: number; y: number };
  size: { width: number; height: number };
  visible: boolean;
  settings?: Record<string, any>;
}

export interface EditorPreferences {
  fontSize: number;
  fontFamily: 'monospace' | 'system' | 'custom';
  lineNumbers: boolean;
  wordWrap: boolean;
  autoIndent: boolean;
  showInvisibles: boolean;
  theme: 'vs' | 'vs-dark' | 'hc-black';
  autoSave: boolean;
  autoComplete: boolean;
  syntaxValidation: boolean;
  showMinimap: boolean;
  tabSize: number;
  insertSpaces: boolean;
}

export interface PlaygroundPreferences {
  defaultTestCategory: TestCategory;
  autoRunTests: boolean;
  showEvaluationPath: boolean;
  enableRealTimeTesting: boolean;
  debugMode: boolean;
  autoSaveTestCases: boolean;
  performanceThresholds: PerformanceThresholds;
  mockDataMode: boolean;
}

export interface PerformanceThresholds {
  maxResponseTime: number; // milliseconds
  maxErrorRate: number; // percentage
  minThroughput: number; // requests per second
  maxConcurrentRequests: number;
  timeoutDuration: number; // seconds
}

export interface SystemSettings {
  id: string;
  organizationId?: string;
  security: SecuritySettings;
  api: ApiSettings;
  database: DatabaseSettings;
  integrations: IntegrationSettings;
  features: FeatureFlags;
  maintenance: MaintenanceSettings;
  createdAt: string;
  updatedAt: string;
  updatedBy: string;
}

export interface SecuritySettings {
  passwordPolicy: PasswordPolicy;
  sessionTimeout: number; // minutes
  maxLoginAttempts: number;
  requireMFA: boolean;
  allowGuestAccess: boolean;
  corsOrigins: string[];
  jwtSecret: string;
  encryptionAlgorithm: string;
  dataRetentionDays: number;
}

export interface PasswordPolicy {
  minLength: number;
  requireUppercase: boolean;
  requireLowercase: boolean;
  requireNumbers: boolean;
  requireSymbols: boolean;
  preventReuse: number; // previous passwords
  expirationDays: number;
}

export interface ApiSettings {
  rateLimit: RateLimitConfig;
  corsEnabled: boolean;
  apiVersioning: boolean;
  defaultVersion: string;
  deprecatedVersions: string[];
  enableSwagger: boolean;
  enableGraphQL: boolean;
  maxRequestSize: number; // bytes
  requestTimeout: number; // seconds
}

export interface RateLimitConfig {
  requestsPerMinute: number;
  requestsPerHour: number;
  burstLimit: number;
  whitelist: string[];
  blockDuration: number; // seconds
}

export interface DatabaseSettings {
  connectionPoolSize: number;
  queryTimeout: number; // seconds
  maxConnections: number;
  enableQueryLogging: boolean;
  slowQueryThreshold: number; // milliseconds
  backupRetentionDays: number;
  encryptionAtRest: boolean;
  sslRequired: boolean;
}

export interface IntegrationSettings {
  identityProviders: IdentityProvider[];
  externalServices: ExternalService[];
  webhooks: WebhookConfig[];
  ldapConfig?: LdapConfig;
  ssoConfig?: SsoConfig;
}

export interface IdentityProvider {
  id: string;
  name: string;
  type: 'cognito' | 'oidc' | 'saml' | 'ldap' | 'internal';
  enabled: boolean;
  configuration: Record<string, any>;
  metadata: ProviderMetadata;
  createdAt: string;
  updatedAt: string;
}

export interface ProviderMetadata {
  issuer: string;
  clientId?: string;
  clientSecret?: string;
  authorizationUrl?: string;
  tokenUrl?: string;
  userInfoUrl?: string;
  jwksUrl?: string;
  scopes: string[];
  redirectUris: string[];
  certificate?: string;
}

export interface ExternalService {
  id: string;
  name: string;
  type: 'webhook' | 'api' | 'database' | 'queue';
  enabled: boolean;
  endpoint: string;
  authentication: ServiceAuth;
  timeout: number;
  retryAttempts: number;
  healthCheckUrl?: string;
}

export interface ServiceAuth {
  type: 'none' | 'bearer' | 'basic' | 'api-key' | 'oauth2';
  credentials: Record<string, any>;
}

export interface WebhookConfig {
  id: string;
  name: string;
  url: string;
  events: string[];
  enabled: boolean;
  secret?: string;
  retryAttempts: number;
  timeout: number;
  headers: Record<string, string>;
}

export interface LdapConfig {
  server: string;
  port: number;
  useSSL: boolean;
  bindDN: string;
  bindPassword: string;
  baseDN: string;
  userSearchFilter: string;
  groupSearchFilter: string;
  groupMembershipAttribute: string;
}

export interface SsoConfig {
  provider: string;
  entityId: string;
  acsUrl: string;
  certificate: string;
  privateKey: string;
  assertionEncryption: boolean;
  nameIdFormat: string;
}

export interface FeatureFlags {
  id: string;
  flags: Record<string, FeatureFlag>;
  lastEvaluated: string;
  createdAt: string;
  updatedAt: string;
}

export interface FeatureFlag {
  key: string;
  enabled: boolean;
  description: string;
  environments: EnvironmentFlag[];
  rolloutStrategy?: RolloutStrategy;
  dependencies?: string[];
  createdAt: string;
  updatedAt: string;
}

export interface EnvironmentFlag {
  environment: 'development' | 'staging' | 'production';
  enabled: boolean;
  percentage?: number;
  userIds?: string[];
  conditions?: FlagCondition[];
}

export interface FlagCondition {
  attribute: string;
  operator: 'equals' | 'not_equals' | 'in' | 'not_in' | 'greater_than' | 'less_than';
  value: any;
}

export interface RolloutStrategy {
  type: 'percentage' | 'user_list' | 'condition';
  value: any;
  schedule?: RolloutSchedule;
}

export interface RolloutSchedule {
  startDate: string;
  endDate?: string;
  gradually: boolean;
  steps?: RolloutStep[];
}

export interface RolloutStep {
  percentage: number;
  date: string;
  description: string;
}

export interface MaintenanceSettings {
  enabled: boolean;
  scheduledMaintenance: ScheduledMaintenance[];
  emergencyMaintenance: EmergencyMaintenance[];
  maintenanceMode: boolean;
  notificationSettings: MaintenanceNotifications;
}

export interface ScheduledMaintenance {
  id: string;
  title: string;
  description: string;
  startTime: string;
  endTime: string;
  affectedServices: string[];
  notificationChannels: string[];
  status: 'scheduled' | 'in_progress' | 'completed' | 'cancelled';
}

export interface EmergencyMaintenance {
  id: string;
  title: string;
  description: string;
  startTime: string;
  estimatedDuration: number; // minutes
  affectedServices: string[];
  status: 'in_progress' | 'completed';
}

export interface MaintenanceNotifications {
  emailEnabled: boolean;
  smsEnabled: boolean;
  inAppEnabled: boolean;
  webhookEnabled: boolean;
  advanceNoticeHours: number;
}

export interface ConfigurationAudit {
  id: string;
  category: 'user_preferences' | 'system_settings' | 'security' | 'integrations' | 'features';
  action: 'create' | 'update' | 'delete' | 'enable' | 'disable';
  entityType: string;
  entityId: string;
  changes: ConfigurationChange[];
  userId: string;
  userName: string;
  ipAddress: string;
  userAgent: string;
  timestamp: string;
}

export interface ConfigurationChange {
  field: string;
  oldValue: any;
  newValue: any;
  type: 'added' | 'modified' | 'removed';
}

export interface ConfigurationTemplate {
  id: string;
  name: string;
  description: string;
  category: 'user_preferences' | 'system_settings' | 'organization' | 'environment';
  template: ConfigurationTemplateData;
  isPublic: boolean;
  tags: string[];
  createdBy: string;
  createdAt: string;
  updatedAt: string;
}

export interface ConfigurationTemplateData {
  userPreferences?: Partial<UserPreferences>;
  systemSettings?: Partial<SystemSettings>;
  featureFlags?: Partial<Record<string, any>>;
  dashboardWidgets?: DashboardWidget[];
  editorSettings?: Partial<EditorPreferences>;
}

export interface ConfigurationImportExport {
  type: 'import' | 'export';
  format: 'json' | 'yaml' | 'xml';
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

export interface ConfigurationBackup {
  id: string;
  name: string;
  description: string;
  data: ConfigurationTemplateData;
  version: string;
  checksum: string;
  size: number;
  createdAt: string;
  createdBy: string;
  expiresAt?: string;
}

export type TestCategory = 
  | 'access_control'
  | 'resource_permissions'
  | 'user_roles'
  | 'conditional_access'
  | 'compliance_testing'
  | 'performance'
  | 'security'
  | 'custom';

export interface ConfigurationFilters {
  category?: string[];
  environment?: string[];
  tags?: string[];
  lastModified?: {
    from?: string;
    to?: string;
  };
  search?: string;
}

export interface ConfigurationState {
  userPreferences?: UserPreferences;
  systemSettings?: SystemSettings;
  featureFlags?: FeatureFlags;
  currentEnvironment: 'development' | 'staging' | 'production';
  isLoading: boolean;
  error?: string;
  lastSync?: string;
  pendingChanges: ConfigurationChange[];
  isDirty: boolean;
}