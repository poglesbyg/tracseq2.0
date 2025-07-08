// TracSeq 2.0 Frontend Configuration
export interface AppConfig {
  api: {
    gatewayUrl: string;
    baseUrl: string;
    wsUrl: string;
    timeout: number;
    retries: number;
  };
  app: {
    name: string;
    version: string;
    description: string;
  };
  features: {
    ragSubmissions: boolean;
    batchOperations: boolean;
    advancedSearch: boolean;
    realTimeNotifications: boolean;
  };
  development: {
    devMode: boolean;
    debugMode: boolean;
    mockApi: boolean;
    hotReload: boolean;
    sourceMaps: boolean;
  };
}

const isDevelopment = (import.meta as { env: Record<string, string | boolean | undefined> }).env.DEV || 
                      (import.meta as { env: Record<string, string | boolean | undefined> }).env.MODE === 'development';

// Default configuration
const defaultConfig: AppConfig = {
  api: {
    gatewayUrl: 'http://localhost:8089',
    baseUrl: 'http://localhost:8089',
    wsUrl: isDevelopment ? 'ws://localhost:8089/ws' : `ws://${typeof window !== 'undefined' ? window.location.host : 'localhost'}/ws`,
    timeout: 30000,
    retries: 3,
  },
  app: {
    name: 'TracSeq 2.0',
    version: '2.0.0',
    description: 'Laboratory Management System',
  },
  features: {
    ragSubmissions: true,
    batchOperations: true,
    advancedSearch: true,
    realTimeNotifications: true,
  },
  development: {
    devMode: isDevelopment,
    debugMode: isDevelopment,
    mockApi: false,
    hotReload: isDevelopment,
    sourceMaps: isDevelopment,
  },
};

// Override with environment variables if available
const env = (import.meta as { env: Record<string, string | boolean | undefined> }).env;
const config: AppConfig = {
  api: {
    gatewayUrl: env.VITE_API_GATEWAY_URL || defaultConfig.api.gatewayUrl,
    baseUrl: env.VITE_API_BASE_URL || defaultConfig.api.baseUrl,
    wsUrl: env.VITE_WS_URL || defaultConfig.api.wsUrl,
    timeout: Number(env.VITE_API_TIMEOUT) || defaultConfig.api.timeout,
    retries: Number(env.VITE_API_RETRIES) || defaultConfig.api.retries,
  },
  app: {
    name: env.VITE_APP_NAME || defaultConfig.app.name,
    version: env.VITE_APP_VERSION || defaultConfig.app.version,
    description: env.VITE_APP_DESCRIPTION || defaultConfig.app.description,
  },
  features: {
    ragSubmissions: env.VITE_FEATURE_RAG_SUBMISSIONS === 'true' || defaultConfig.features.ragSubmissions,
    batchOperations: env.VITE_FEATURE_BATCH_OPERATIONS === 'true' || defaultConfig.features.batchOperations,
    advancedSearch: env.VITE_FEATURE_ADVANCED_SEARCH === 'true' || defaultConfig.features.advancedSearch,
    realTimeNotifications: env.VITE_FEATURE_REAL_TIME_NOTIFICATIONS === 'true' || defaultConfig.features.realTimeNotifications,
  },
  development: {
    devMode: env.VITE_DEV_MODE === 'true' || defaultConfig.development.devMode,
    debugMode: env.VITE_DEBUG_MODE === 'true' || defaultConfig.development.debugMode,
    mockApi: env.VITE_MOCK_API === 'true' || defaultConfig.development.mockApi,
    hotReload: env.VITE_HOT_RELOAD === 'true' || defaultConfig.development.hotReload,
    sourceMaps: env.VITE_SOURCE_MAPS === 'true' || defaultConfig.development.sourceMaps,
  },
};

// Service endpoints
export const endpoints = {
  auth: `${config.api.baseUrl}/api/auth`,
  samples: `${config.api.baseUrl}/api/samples`,
  storage: `${config.api.baseUrl}/api/storage`,
  templates: `${config.api.baseUrl}/api/templates`,
  sequencing: `${config.api.baseUrl}/api/sequencing`,
  notifications: `${config.api.baseUrl}/api/notifications`,
  rag: `${config.api.baseUrl}/api/rag`,
  dashboard: `${config.api.baseUrl}/api/dashboard`,
  spreadsheets: `${config.api.baseUrl}/api/spreadsheets`,
  reports: `${config.api.baseUrl}/api/reports`,
  events: `${config.api.baseUrl}/api/events`,
  transactions: `${config.api.baseUrl}/api/transactions`,
};

export default config; 