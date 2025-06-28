import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse, AxiosError } from 'axios';

// Enhanced API configuration
interface ApiConfig {
  baseUrl: string;
  timeout: number;
  retries: number;
  retryDelay: number;
}

interface ApiResponse<T = unknown> {
  data: T;
  success: boolean;
  message?: string;
  errors?: string[];
}

interface ApiError {
  error_id: string;
  error_code: string;
  message: string;
  severity: 'Low' | 'Medium' | 'High' | 'Critical';
  context: Record<string, string>;
  retryable: boolean;
  timestamp: string;
}

class EnhancedApiClient {
  private client: AxiosInstance;
  private config: ApiConfig;

  constructor(apiConfig: ApiConfig) {
    this.config = apiConfig;
    this.client = axios.create({
      baseURL: apiConfig.baseUrl,
      timeout: apiConfig.timeout,
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json',
        'X-Client-Version': '2.0.0',
        'X-Client-Name': 'TracSeq Frontend',
      },
    });

    this.setupInterceptors();
  }

  private setupInterceptors(): void {
    // Request interceptor
    this.client.interceptors.request.use(
      (config) => {
        // Add auth token if available
        const token = localStorage.getItem('auth_token');
        if (token) {
          config.headers.Authorization = `Bearer ${token}`;
        }

        // Add request ID for distributed tracing
        config.headers['X-Request-ID'] = crypto.randomUUID();
        config.headers['X-Request-Timestamp'] = new Date().toISOString();

        // Add correlation ID for microservices
        const correlationId = sessionStorage.getItem('correlation_id') || crypto.randomUUID();
        config.headers['X-Correlation-ID'] = correlationId;
        sessionStorage.setItem('correlation_id', correlationId);

        return config;
      },
      (error) => {
        console.error('Request interceptor error:', error);
        return Promise.reject(error);
      }
    );

    // Response interceptor
    this.client.interceptors.response.use(
      (response: AxiosResponse) => {
        return response;
      },
      async (error: AxiosError) => {
        const originalRequest = error.config as AxiosRequestConfig & { _retry?: boolean; _retryCount?: number };

        // Handle 401 unauthorized - token refresh
        if (error.response?.status === 401 && !originalRequest._retry) {
          originalRequest._retry = true;
          
          try {
            await this.refreshToken();
            return this.client(originalRequest);
          } catch (refreshError) {
            // Refresh failed, redirect to login
            localStorage.removeItem('auth_token');
            window.location.href = '/login';
            return Promise.reject(refreshError);
          }
        }

        // Handle retryable errors
        if (this.shouldRetry(error) && (!originalRequest._retryCount || originalRequest._retryCount < this.config.retries)) {
          originalRequest._retryCount = (originalRequest._retryCount || 0) + 1;
          
          const delay = this.calculateRetryDelay(originalRequest._retryCount);
          await this.delay(delay);
          
          return this.client(originalRequest);
        }

        return Promise.reject(this.formatError(error));
      }
    );
  }

  private async refreshToken(): Promise<void> {
    const refreshToken = localStorage.getItem('refresh_token');
    if (!refreshToken) {
      throw new Error('No refresh token available');
    }

    const response = await axios.post('/api/auth/refresh', {
      refresh_token: refreshToken,
    });

    if (response.data.access_token) {
      localStorage.setItem('auth_token', response.data.access_token);
      if (response.data.refresh_token) {
        localStorage.setItem('refresh_token', response.data.refresh_token);
      }
    }
  }

  private shouldRetry(error: AxiosError): boolean {
    const status = error.response?.status;
    const isNetworkError = !error.response;
    const isServerError = status && status >= 500 && status < 600;
    const isRetryableClientError = status === 408 || status === 429; // Timeout or Too Many Requests
    const responseData = error.response?.data as any;
    const isRetryableResponse = responseData?.retryable === true;

    return isNetworkError || isServerError || isRetryableClientError || isRetryableResponse;
  }

  private calculateRetryDelay(retryCount: number): number {
    // Exponential backoff with jitter
    const baseDelay = 1000; // 1 second
    const maxDelay = 30000; // 30 seconds
    const exponentialDelay = Math.min(baseDelay * Math.pow(2, retryCount - 1), maxDelay);
    const jitter = Math.random() * 1000; // Add up to 1 second of jitter
    return exponentialDelay + jitter;
  }

  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  private formatError(error: AxiosError): ApiError {
    const status = error.response?.status;
    const data = error.response?.data as any;

    if (data?.error) {
      return data.error;
    }

    return {
      error_id: crypto.randomUUID(),
      error_code: this.getErrorCode(status),
      message: this.getErrorMessage(error),
      severity: this.getErrorSeverity(status),
      context: {
        status: status?.toString() || 'unknown',
        url: error.config?.url || 'unknown',
        method: error.config?.method?.toUpperCase() || 'unknown',
      },
      retryable: this.shouldRetry(error),
      timestamp: new Date().toISOString(),
    };
  }

  private getErrorCode(status?: number): string {
    switch (status) {
      case 400: return 'BAD_REQUEST';
      case 401: return 'UNAUTHORIZED';
      case 403: return 'FORBIDDEN';
      case 404: return 'NOT_FOUND';
      case 408: return 'TIMEOUT';
      case 429: return 'RATE_LIMITED';
      case 500: return 'INTERNAL_SERVER_ERROR';
      case 502: return 'BAD_GATEWAY';
      case 503: return 'SERVICE_UNAVAILABLE';
      case 504: return 'GATEWAY_TIMEOUT';
      default: return 'CLIENT_ERROR';
    }
  }

  private getErrorMessage(error: AxiosError): string {
    const data = error.response?.data as any;
    return data?.message || error.message || 'An unexpected error occurred';
  }

  private getErrorSeverity(status?: number): 'Low' | 'Medium' | 'High' | 'Critical' {
    if (!status) return 'High';
    if (status >= 500) return 'Critical';
    if (status >= 400) return 'Medium';
    return 'Low';
  }

  // HTTP Methods
  public async get<T>(url: string, config?: AxiosRequestConfig): Promise<ApiResponse<T>> {
    const response = await this.client.get(url, config);
    return response.data;
  }

  public async post<T>(url: string, data?: unknown, config?: AxiosRequestConfig): Promise<ApiResponse<T>> {
    const response = await this.client.post(url, data, config);
    return response.data;
  }

  public async put<T>(url: string, data?: unknown, config?: AxiosRequestConfig): Promise<ApiResponse<T>> {
    const response = await this.client.put(url, data, config);
    return response.data;
  }

  public async patch<T>(url: string, data?: unknown, config?: AxiosRequestConfig): Promise<ApiResponse<T>> {
    const response = await this.client.patch(url, data, config);
    return response.data;
  }

  public async delete<T>(url: string, config?: AxiosRequestConfig): Promise<ApiResponse<T>> {
    const response = await this.client.delete(url, config);
    return response.data;
  }

  public async upload<T>(url: string, file: File, onProgress?: (progress: number) => void): Promise<T> {
    const formData = new FormData();
    formData.append('file', file);

    const uploadConfig: AxiosRequestConfig = {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
      onUploadProgress: (event) => {
        if (onProgress && event.total) {
          const progress = Math.round((event.loaded * 100) / event.total);
          onProgress(progress);
        }
      },
    };

    const response = await this.client.post(url, formData, uploadConfig);
    return response.data;
  }

  // Health check
  public async healthCheck(): Promise<boolean> {
    try {
      await this.client.get('/health');
      return true;
    } catch {
      return false;
    }
  }
}

// Configuration based on environment
const getApiConfig = () => {
  const isDev = (import.meta as any).env?.MODE === 'development' || (import.meta as any).env?.DEV;
  return {
    baseUrl: isDev ? 'http://localhost:8089/api' : '/api',
    timeout: 30000,
    retries: 3,
    retryDelay: 1000,
  };
};

// API clients for different services
const createApiClient = (baseUrl: string): EnhancedApiClient => {
  return new EnhancedApiClient({
    baseUrl,
    timeout: 30000,
    retries: 3,
    retryDelay: 1000,
  });
};

// Main API client (goes through gateway)
export const apiClient = createApiClient(getApiConfig().baseUrl);

// Service-specific clients (all route through gateway)
const baseUrl = getApiConfig().baseUrl;
export const authClient = createApiClient(`${baseUrl}/auth`);
export const sampleClient = createApiClient(`${baseUrl}/samples`);
export const storageClient = createApiClient(`${baseUrl}/storage`);
export const templateClient = createApiClient(`${baseUrl}/templates`);
export const sequencingClient = createApiClient(`${baseUrl}/sequencing`);
export const notificationClient = createApiClient(`${baseUrl}/notifications`);
export const ragClient = createApiClient(`${baseUrl}/rag`);

export default apiClient; 