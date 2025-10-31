// Authorization Playground Types

export interface AuthorizationTestCase {
  id: string;
  name: string;
  description: string;
  policyStoreId: string;
  principal: Principal;
  action: Action;
  resource: Resource;
  context?: Record<string, any>;
  expectedDecision: 'Allow' | 'Deny';
  tags: string[];
  category: TestCategory;
  isTemplate: boolean;
  createdAt: string;
  updatedAt: string;
  createdBy: string;
}

export interface TestScenario {
  id: string;
  name: string;
  description: string;
  policyStoreId: string;
  testCases: string[]; // Array of test case IDs
  tags: string[];
  category: TestCategory;
  executionMode: 'sequential' | 'parallel';
  stopOnFirstFailure: boolean;
  createdAt: string;
  updatedAt: string;
  createdBy: string;
}

export interface Principal {
  id: string;
  type: string;
  attributes: Record<string, any>;
  tags?: string[];
}

export interface Action {
  id: string;
  type: string;
  attributes: Record<string, any>;
}

export interface Resource {
  id: string;
  type: string;
  attributes: Record<string, any>;
  tags?: string[];
}

export interface AuthorizationResult {
  decision: 'Allow' | 'Deny';
  matchedPolicies: MatchedPolicy[];
  unmatchedPolicies: UnmatchedPolicy[];
  evaluationPath: EvaluationStep[];
  context: Record<string, any>;
  executionTime: number; // milliseconds
  timestamp: string;
  testCaseId: string;
}

export interface MatchedPolicy {
  id: string;
  name: string;
  effect: 'Permit' | 'Forbid';
  conditions: PolicyCondition[];
  score: number; // Match confidence score
  evaluationDetails: string;
}

export interface UnmatchedPolicy {
  id: string;
  name: string;
  reason: string;
  evaluationDetails: string;
}

export interface PolicyCondition {
  field: string;
  operator: 'equals' | 'not_equals' | 'in' | 'not_in' | 'exists' | 'not_exists' | 'greater_than' | 'less_than';
  value: any;
  isMet: boolean;
  reason?: string;
}

export interface EvaluationStep {
  step: number;
  description: string;
  policyId?: string;
  condition?: string;
  result: boolean;
  reason: string;
  timestamp: string;
}

export interface DebugSession {
  id: string;
  testCaseId: string;
  policyStoreId: string;
  currentStep: number;
  evaluationPath: EvaluationStep[];
  paused: boolean;
  breakpoints: number[]; // Step numbers where execution pauses
  startedAt: string;
  lastUpdated: string;
}

export interface PerformanceMetrics {
  totalRequests: number;
  successfulRequests: number;
  failedRequests: number;
  averageResponseTime: number; // milliseconds
  p95ResponseTime: number; // milliseconds
  p99ResponseTime: number; // milliseconds
  throughput: number; // requests per second
  errorRate: number; // percentage
  timeRange: {
    start: string;
    end: string;
  };
}

export interface LoadTestResult {
  id: string;
  name: string;
  scenarioId: string;
  configuration: LoadTestConfiguration;
  metrics: PerformanceMetrics;
  results: AuthorizationResult[];
  status: 'running' | 'completed' | 'failed' | 'cancelled';
  startedAt: string;
  completedAt?: string;
  createdBy: string;
}

export interface LoadTestConfiguration {
  duration: number; // seconds
  concurrentUsers: number;
  rampUpTime: number; // seconds
  requestsPerSecond: number;
  policyStoreId: string;
  testCases: string[];
}

export interface CoverageAnalysis {
  policyStoreId: string;
  totalPolicies: number;
  testedPolicies: number;
  untestedPolicies: string[]; // Policy IDs
  coveragePercentage: number;
  policyCoverage: PolicyCoverage[];
  recommendations: string[];
  lastAnalyzed: string;
}

export interface PolicyCoverage {
  policyId: string;
  policyName: string;
  testCases: string[]; // Test case IDs that exercise this policy
  coverageScore: number; // 0-100
  untestedConditions: string[];
  lastTested: string;
}

export interface PolicyRecommendation {
  id: string;
  type: 'missing_policy' | 'conflicting_policies' | 'untested_area' | 'performance_optimization';
  severity: 'low' | 'medium' | 'high' | 'critical';
  title: string;
  description: string;
  affectedPolicies: string[];
  recommendedActions: string[];
  estimatedImpact: string;
  category: TestCategory;
}

export interface TestTemplate {
  id: string;
  name: string;
  description: string;
  category: TestCategory;
  templateData: Partial<AuthorizationTestCase>;
  tags: string[];
  isPublic: boolean;
  createdAt: string;
  createdBy: string;
  usageCount: number;
}

export interface AuthorizationReport {
  id: string;
  name: string;
  description: string;
  policyStoreId: string;
  testScenarios: string[];
  performanceMetrics: PerformanceMetrics;
  coverageAnalysis: CoverageAnalysis;
  recommendations: PolicyRecommendation[];
  generatedAt: string;
  generatedBy: string;
  format: 'pdf' | 'html' | 'json' | 'csv';
  summary: ReportSummary;
}

export interface ReportSummary {
  totalTestCases: number;
  passedTestCases: number;
  failedTestCases: number;
  averageExecutionTime: number;
  policyCoverage: number;
  topRecommendations: string[];
  testDuration: number; // seconds
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

export interface PlaygroundSettings {
  autoSave: boolean;
  autoValidate: boolean;
  showEvaluationPath: boolean;
  enableRealTimeTesting: boolean;
  defaultTestCategory: TestCategory;
  performanceThresholds: {
    maxResponseTime: number; // milliseconds
    maxErrorRate: number; // percentage
    minThroughput: number; // requests per second
  };
}

export interface PlaygroundFilters {
  category?: TestCategory[];
  tags?: string[];
  status?: ('passed' | 'failed' | 'pending')[];
  dateRange?: {
    start: string;
    end: string;
  };
  executionTime?: {
    min: number;
    max: number;
  };
}

export interface PlaygroundState {
  currentTestCase?: AuthorizationTestCase;
  currentScenario?: TestScenario;
  currentDebugSession?: DebugSession;
  testResults: AuthorizationResult[];
  scenarios: TestScenario[];
  templates: TestTemplate[];
  settings: PlaygroundSettings;
  isRunning: boolean;
  selectedTestCases: string[];
  filters: PlaygroundFilters;
}

export interface CreateTestCaseRequest {
  name: string;
  description: string;
  policyStoreId: string;
  principal: Principal;
  action: Action;
  resource: Resource;
  context?: Record<string, any>;
  expectedDecision: 'Allow' | 'Deny';
  tags: string[];
  category: TestCategory;
  isTemplate?: boolean;
}

export interface CreateScenarioRequest {
  name: string;
  description: string;
  policyStoreId: string;
  testCases: string[];
  tags: string[];
  category: TestCategory;
  executionMode: 'sequential' | 'parallel';
  stopOnFirstFailure: boolean;
}

export interface RunScenarioRequest {
  scenarioId: string;
  options?: {
    saveResults: boolean;
    generateReport: boolean;
    performanceMode: boolean;
  };
}

export interface DebugSessionRequest {
  testCaseId: string;
  breakpoints?: number[];
  stepMode: boolean;
}

export interface LoadTestRequest {
  name: string;
  scenarioId: string;
  configuration: LoadTestConfiguration;
  notifyOnComplete?: boolean;
}