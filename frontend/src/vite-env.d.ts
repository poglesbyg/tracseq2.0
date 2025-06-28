/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_GATEWAY_URL: string;
  readonly VITE_API_BASE_URL: string;
  readonly VITE_WS_URL: string;
  readonly VITE_API_TIMEOUT: string;
  readonly VITE_API_RETRIES: string;
  readonly VITE_APP_NAME: string;
  readonly VITE_APP_VERSION: string;
  readonly VITE_APP_DESCRIPTION: string;
  readonly VITE_FEATURE_RAG_SUBMISSIONS: string;
  readonly VITE_FEATURE_BATCH_OPERATIONS: string;
  readonly VITE_FEATURE_ADVANCED_SEARCH: string;
  readonly VITE_FEATURE_REAL_TIME_NOTIFICATIONS: string;
  readonly VITE_DEV_MODE: string;
  readonly VITE_DEBUG_MODE: string;
  readonly VITE_MOCK_API: string;
  readonly VITE_HOT_RELOAD: string;
  readonly VITE_SOURCE_MAPS: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
