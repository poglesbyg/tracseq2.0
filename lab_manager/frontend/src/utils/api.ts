import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';

// Enhanced API configuration
interface ApiConfig {
  baseUrl: string;
  timeout: number;
  retries: number;
  retryDelay: number;
}

interface ApiResponse<T = any> {
  data: T;
  status: number;
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

class ApiClient {
  private client: AxiosInstance;
  private config: ApiConfig;

  constructor(config: ApiConfig) {
    this.config = config;
    this.client = axios.create({
      baseURL: config.baseUrl,
      timeout: config.timeout,
      headers: {
        'Content-Type': 'application/json',
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
        
        // Add request ID for tracing
        config.headers['X-Request-ID'] = crypto.randomUUID();
        
        return config;
      },
      (error) => Promise.reject(error)
    );

    // Response interceptor
    this.client.interceptors.response.use(
      (response: AxiosResponse) => response,
      async (error) => {
        const originalRequest = error.config;
        
        // Handle 401 unauthorized
        if (error.response?.status === 401 && !originalRequest._retry) {
          originalRequest._retry = true;
          // Implement token refresh logic here
          return this.client(originalRequest);
        }

        // Handle retryable errors
        if (this.shouldRetry(error) && !originalRequest._retryCount) {
          originalRequest._retryCount = 0;
        }

        if (originalRequest._retryCount < this.config.retries) {
          originalRequest._retryCount += 1;
          await this.delay(this.config.retryDelay * originalRequest._retryCount);
          return this.client(originalRequest);
        }

        return Promise.reject(this.formatError(error));
      }
    );
  }

  private shouldRetry(error: any): boolean {
    return (
      error.code === 'NETWORK_ERROR' ||
      error.code === 'TIMEOUT' ||
      (error.response?.status >= 500 && error.response?.status < 600) ||
      error.response?.data?.retryable === true
    );
  }

  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  private formatError(error: any): ApiError {
    if (error.response?.data?.error) {
      return error.response.data.error;
    }

    return {
      error_id: crypto.randomUUID(),
      error_code: 'CLIENT_ERROR',
      message: error.message || 'An unexpected error occurred',
      severity: 'Medium',
      context: {
        status: error.response?.status?.toString() || 'unknown',
        url: error.config?.url || 'unknown',
      },
      retryable: false,
      timestamp: new Date().toISOString(),
    };
  }

  public async get<T>(url: string, config?: AxiosRequestConfig): Promise<ApiResponse<T>> {
    const response = await this.client.get(url, config);
    return response.data;
  }

  public async post<T>(url: string, data?: any, config?: AxiosRequestConfig): Promise<ApiResponse<T>> {
    const response = await this.client.post(url, data, config);
    return response.data;
  }

  public async put<T>(url: string, data?: any, config?: AxiosRequestConfig): Promise<ApiResponse<T>> {
    const response = await this.client.put(url, data, config);
    return response.data;
  }

  public async delete<T>(url: string, config?: AxiosRequestConfig): Promise<ApiResponse<T>> {
    const response = await this.client.delete(url, config);
    return response.data;
  }
}

// API clients configuration
const API_CONFIG = {
  labManager: new ApiClient({
    baseUrl: import.meta.env.PROD ? '' : '',
    timeout: 30000,
    retries: 3,
    retryDelay: 1000,
  }),
  rag: new ApiClient({
    baseUrl: '',
    timeout: 60000, // RAG operations can take longer
    retries: 2,
    retryDelay: 2000,
  }),
};

export default API_CONFIG;
export type { ApiResponse, ApiError }; 
