# 🌐 Frontend Architecture - React TypeScript UI

## Overview

The Lab Manager frontend is a modern React application built with TypeScript, providing an intuitive interface for laboratory sample management, storage tracking, and data analysis.

## 🏗️ Technology Stack

### Core Framework
```
React 18.3+        - Component framework with concurrent features
TypeScript 5.8+    - Type safety and developer experience  
Vite 6.3+          - Fast build tool and dev server
```

### UI & Styling
```
TailwindCSS 3.3+   - Utility-first CSS framework
Headless UI 2.2+   - Unstyled accessible components
Heroicons 2.2+     - Beautiful SVG icons
```

### State Management & Data
```
React Query 5.80+  - Server state management and caching
Axios 1.9+         - HTTP client for API communication
React Router 7.6+  - Client-side routing
```

### Development & Testing
```
Jest 29.7+         - Unit testing framework
Testing Library    - Component testing utilities
ESLint 9.25+       - Code linting and formatting
```

## 📁 Project Structure

```
frontend/
├── 📁 src/
│   ├── 📁 components/          # Reusable UI components
│   │   ├── 🧪 samples/         # Sample management components
│   │   ├── 🏪 storage/         # Storage management components
│   │   ├── 📊 spreadsheets/    # Data visualization components
│   │   ├── 👥 users/           # User management components
│   │   └── 🔧 common/          # Shared components
│   ├── 📁 pages/               # Route-level page components
│   │   ├── Dashboard.tsx       # Main dashboard
│   │   ├── Samples.tsx         # Sample management
│   │   ├── Storage.tsx         # Storage management
│   │   ├── Spreadsheets.tsx    # Data analysis
│   │   └── Profile.tsx         # User profile
│   ├── 📁 hooks/               # Custom React hooks
│   │   ├── useAuth.ts          # Authentication state
│   │   ├── useSamples.ts       # Sample data management
│   │   └── useStorage.ts       # Storage operations
│   ├── 📁 services/            # API service layer
│   │   ├── api.ts              # Base API configuration
│   │   ├── auth.ts             # Authentication services
│   │   ├── samples.ts          # Sample API calls
│   │   └── storage.ts          # Storage API calls
│   ├── 📁 types/               # TypeScript type definitions
│   │   ├── auth.ts             # Authentication types
│   │   ├── sample.ts           # Sample data types
│   │   └── storage.ts          # Storage types
│   ├── 📁 utils/               # Utility functions
│   │   ├── formatting.ts       # Data formatting helpers
│   │   ├── validation.ts       # Client-side validation
│   │   └── constants.ts        # Application constants
│   └── 📁 __tests__/           # Test files
│       ├── components/         # Component tests
│       ├── hooks/              # Hook tests
│       └── utils/              # Utility tests
├── 📁 public/                  # Static assets
├── 📄 package.json             # Dependencies and scripts
├── 📄 tailwind.config.js       # Tailwind configuration
├── 📄 vite.config.ts           # Vite configuration
└── 📄 tsconfig.json            # TypeScript configuration
```

## 🧩 Component Architecture

### Component Hierarchy
```
App
├── 🔐 AuthProvider            # Authentication context
├── 🌐 Router                  # Client-side routing
├── 📊 QueryProvider           # React Query setup
└── 📱 Layout                  # Main app layout
    ├── 🧭 Navigation          # Main navigation
    ├── 📄 PageContent         # Route-specific content
    └── 🍞 Notifications       # Toast notifications
```

### Core Components

#### **🧪 Sample Management**
```typescript
// Sample list with search, filtering, and pagination
<SampleList 
  filters={filters}
  onSampleSelect={handleSelect}
  onStateChange={handleStateChange}
/>

// Sample creation and editing modal
<SampleModal 
  sample={sample}
  isOpen={isOpen}
  onSave={handleSave}
  onClose={handleClose}
/>

// Sample state transition component
<SampleStateManager 
  sample={sample}
  onStateChange={handleStateChange}
  allowedTransitions={transitions}
/>
```

#### **🏪 Storage Management**
```typescript
// Storage location hierarchy view
<StorageLocationTree 
  locations={locations}
  onLocationSelect={handleLocationSelect}
  onCapacityWarning={handleWarning}
/>

// Sample assignment interface
<StorageAssignment 
  sample={sample}
  availableLocations={locations}
  onAssign={handleAssign}
/>

// Capacity monitoring dashboard
<CapacityOverview 
  utilizationData={data}
  onThresholdAlert={handleAlert}
/>
```

#### **📊 Data Visualization**
```typescript
// Spreadsheet data viewer with search and pagination
<SpreadsheetDataViewer 
  dataset={dataset}
  onClose={handleClose}
/>

// Interactive data table with sorting and filtering
<DataTable 
  data={data}
  columns={columns}
  onSort={handleSort}
  onFilter={handleFilter}
/>

// Chart components for analytics
<SampleAnalytics 
  data={analyticsData}
  chartType="bar"
  onDrillDown={handleDrillDown}
/>
```

## 🔧 State Management

### React Query for Server State
```typescript
// Sample data queries
const useSamples = (filters: SampleFilters) => {
  return useQuery({
    queryKey: ['samples', filters],
    queryFn: () => api.samples.list(filters),
    staleTime: 5 * 60 * 1000, // 5 minutes
    refetchOnWindowFocus: false,
  });
};

// Sample mutations
const useCreateSample = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: api.samples.create,
    onSuccess: () => {
      queryClient.invalidateQueries(['samples']);
      toast.success('Sample created successfully');
    },
    onError: (error) => {
      toast.error(`Failed to create sample: ${error.message}`);
    },
  });
};
```

### Context for Global State
```typescript
// Authentication context
interface AuthContextType {
  user: User | null;
  isAuthenticated: boolean;
  login: (credentials: LoginCredentials) => Promise<void>;
  logout: () => void;
  hasPermission: (permission: string) => boolean;
}

// Theme and UI preferences
interface ThemeContextType {
  theme: 'light' | 'dark';
  toggleTheme: () => void;
  sidebarCollapsed: boolean;
  toggleSidebar: () => void;
}
```

## 🎨 Design System

### Color Palette
```css
/* Primary Colors */
--primary-50:  #eff6ff;
--primary-500: #3b82f6;
--primary-600: #2563eb;
--primary-700: #1d4ed8;

/* Semantic Colors */
--success: #10b981;
--warning: #f59e0b;
--error:   #ef4444;
--info:    #06b6d4;

/* Temperature Zone Colors */
--temp-minus80: #1e40af;  /* -80°C - Deep blue */
--temp-minus20: #3b82f6;  /* -20°C - Blue */
--temp-4c:      #06b6d4;  /* 4°C - Cyan */
--temp-rt:      #10b981;  /* RT - Green */
--temp-37c:     #f59e0b;  /* 37°C - Amber */
```

### Component Styling Patterns
```typescript
// Consistent button variants
const buttonVariants = {
  primary: "bg-blue-600 hover:bg-blue-700 text-white",
  secondary: "bg-gray-200 hover:bg-gray-300 text-gray-900",
  danger: "bg-red-600 hover:bg-red-700 text-white",
  ghost: "hover:bg-gray-100 text-gray-700",
};

// Status badge styles
const statusStyles = {
  pending: "bg-yellow-100 text-yellow-800",
  validated: "bg-green-100 text-green-800", 
  in_storage: "bg-blue-100 text-blue-800",
  in_sequencing: "bg-purple-100 text-purple-800",
  completed: "bg-gray-100 text-gray-800",
};
```

## 🔄 Data Flow Patterns

### Unidirectional Data Flow
```
User Action → Event Handler → API Call → React Query → Component Re-render
```

### Form Management Pattern
```typescript
const SampleForm = ({ sample, onSubmit }: SampleFormProps) => {
  const [formData, setFormData] = useState(sample || initialData);
  const [errors, setErrors] = useState<ValidationErrors>({});
  
  const validateForm = useCallback(() => {
    const validationErrors = validateSample(formData);
    setErrors(validationErrors);
    return Object.keys(validationErrors).length === 0;
  }, [formData]);
  
  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (validateForm()) {
      await onSubmit(formData);
    }
  };
  
  return (
    <form onSubmit={handleSubmit}>
      {/* Form fields with error handling */}
    </form>
  );
};
```

### Error Boundary Pattern
```typescript
class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }
  
  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }
  
  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Error caught by boundary:', error, errorInfo);
    // Send to error reporting service
  }
  
  render() {
    if (this.state.hasError) {
      return <ErrorFallback error={this.state.error} />;
    }
    
    return this.props.children;
  }
}
```

## 🔍 Search & Filtering

### Advanced Search Implementation
```typescript
const useAdvancedSearch = () => {
  const [searchParams, setSearchParams] = useState<SearchParams>({
    query: '',
    filters: {},
    sortBy: 'created_at',
    sortOrder: 'desc',
    page: 1,
    limit: 50,
  });
  
  const debouncedSearch = useMemo(
    () => debounce((params: SearchParams) => {
      // Trigger search with debounced parameters
    }, 300),
    []
  );
  
  const updateSearch = useCallback((updates: Partial<SearchParams>) => {
    const newParams = { ...searchParams, ...updates, page: 1 };
    setSearchParams(newParams);
    debouncedSearch(newParams);
  }, [searchParams, debouncedSearch]);
  
  return { searchParams, updateSearch };
};
```

### Filter Components
```typescript
// Multi-select filter for sample types
<FilterSelect
  label="Sample Type"
  options={sampleTypes}
  selected={filters.sampleType}
  onChange={(value) => updateFilter('sampleType', value)}
  multiple
/>

// Date range picker for temporal filtering
<DateRangeFilter
  label="Collection Date"
  startDate={filters.startDate}
  endDate={filters.endDate}
  onChange={(range) => updateFilter('dateRange', range)}
/>

// Temperature zone filter with visual indicators
<TemperatureZoneFilter
  selected={filters.temperatureZone}
  onChange={(zone) => updateFilter('temperatureZone', zone)}
  showCapacity
/>
```

## 📱 Responsive Design

### Breakpoint Strategy
```css
/* Mobile-first responsive design */
.container {
  @apply px-4;
  
  @screen sm {
    @apply px-6;
  }
  
  @screen lg {
    @apply px-8;
  }
  
  @screen xl {
    @apply px-12;
  }
}

/* Component responsiveness */
.sample-grid {
  @apply grid grid-cols-1;
  
  @screen md {
    @apply grid-cols-2;
  }
  
  @screen lg {
    @apply grid-cols-3;
  }
  
  @screen xl {
    @apply grid-cols-4;
  }
}
```

### Mobile-Optimized Components
```typescript
// Responsive navigation that collapses on mobile
<MobileNavigation 
  isOpen={isMobileMenuOpen}
  onToggle={toggleMobileMenu}
  items={navigationItems}
/>

// Touch-friendly data tables for mobile
<ResponsiveTable 
  data={data}
  stackOnMobile
  swipeActions={['edit', 'delete']}
/>
```

## 🧪 Testing Strategy

### Component Testing
```typescript
// Sample component test
describe('SampleCard', () => {
  const mockSample = {
    id: 'sample-1',
    name: 'Test Sample',
    status: 'validated',
    barcode: 'TEST-001',
  };
  
  test('renders sample information correctly', () => {
    render(<SampleCard sample={mockSample} />);
    
    expect(screen.getByText('Test Sample')).toBeInTheDocument();
    expect(screen.getByText('TEST-001')).toBeInTheDocument();
    expect(screen.getByText('Validated')).toBeInTheDocument();
  });
  
  test('handles state change', async () => {
    const onStateChange = jest.fn();
    render(<SampleCard sample={mockSample} onStateChange={onStateChange} />);
    
    fireEvent.click(screen.getByText('Change State'));
    fireEvent.click(screen.getByText('In Storage'));
    
    expect(onStateChange).toHaveBeenCalledWith('sample-1', 'in_storage');
  });
});
```

### Integration Testing
```typescript
// End-to-end workflow test
test('sample creation workflow', async () => {
  render(<App />);
  
  // Navigate to samples page
  fireEvent.click(screen.getByText('Samples'));
  
  // Open create sample modal
  fireEvent.click(screen.getByText('Create Sample'));
  
  // Fill form
  fireEvent.change(screen.getByLabelText('Sample Name'), {
    target: { value: 'New Test Sample' }
  });
  
  // Submit form
  fireEvent.click(screen.getByText('Create'));
  
  // Verify sample appears in list
  await waitFor(() => {
    expect(screen.getByText('New Test Sample')).toBeInTheDocument();
  });
});
```

## 🚀 Performance Optimization

### Code Splitting
```typescript
// Route-level code splitting
const Dashboard = lazy(() => import('./pages/Dashboard'));
const Samples = lazy(() => import('./pages/Samples'));
const Storage = lazy(() => import('./pages/Storage'));

// Component-level splitting for large features
const SpreadsheetViewer = lazy(() => import('./components/SpreadsheetViewer'));
```

### Memoization Strategies
```typescript
// Expensive computation memoization
const filteredSamples = useMemo(() => {
  return samples.filter(sample => 
    matchesSearchCriteria(sample, searchParams)
  );
}, [samples, searchParams]);

// Callback memoization to prevent unnecessary re-renders
const handleSampleSelect = useCallback((sampleId: string) => {
  setSelectedSample(samples.find(s => s.id === sampleId));
}, [samples]);
```

### Virtual Scrolling for Large Datasets
```typescript
// Virtual list for thousands of samples
import { FixedSizeList as List } from 'react-window';

const SampleVirtualList = ({ samples }: { samples: Sample[] }) => (
  <List
    height={600}
    itemCount={samples.length}
    itemSize={80}
    itemData={samples}
  >
    {SampleRow}
  </List>
);
```

## 🔒 Security & Validation

### Client-Side Validation
```typescript
// Form validation schema
const sampleValidationSchema = z.object({
  name: z.string().min(3, 'Name must be at least 3 characters'),
  barcode: z.string().regex(/^[A-Z]{3}-\d{8}-\d{3}$/, 'Invalid barcode format'),
  volume: z.number().positive('Volume must be positive'),
  temperatureRequirement: z.enum(['-80c', '-20c', '4c', 'rt', '37c']),
});

// Validation hook
const useValidation = <T>(schema: ZodSchema<T>) => {
  const validate = useCallback((data: unknown): ValidationResult<T> => {
    try {
      const validated = schema.parse(data);
      return { success: true, data: validated, errors: {} };
    } catch (error) {
      if (error instanceof ZodError) {
        return {
          success: false,
          data: null,
          errors: error.flatten().fieldErrors,
        };
      }
      throw error;
    }
  }, [schema]);
  
  return { validate };
};
```

### XSS Protection
```typescript
// Safe HTML rendering with DOMPurify
import DOMPurify from 'dompurify';

const SafeHTML = ({ content }: { content: string }) => {
  const sanitizedContent = useMemo(() => {
    return DOMPurify.sanitize(content);
  }, [content]);
  
  return <div dangerouslySetInnerHTML={{ __html: sanitizedContent }} />;
};
```

## 📈 Analytics & Monitoring

### User Interaction Tracking
```typescript
// Analytics hook for user interactions
const useAnalytics = () => {
  const trackEvent = useCallback((event: string, properties?: Record<string, any>) => {
    // Send to analytics service
    analytics.track(event, {
      timestamp: new Date().toISOString(),
      userId: user?.id,
      ...properties,
    });
  }, [user]);
  
  return { trackEvent };
};

// Usage in components
const SampleCard = ({ sample }: SampleCardProps) => {
  const { trackEvent } = useAnalytics();
  
  const handleView = () => {
    trackEvent('sample_viewed', { sampleId: sample.id });
    onView(sample);
  };
  
  return (
    <div onClick={handleView}>
      {/* Sample card content */}
    </div>
  );
};
```

### Performance Monitoring
```typescript
// Performance monitoring hook
const usePerformanceMonitoring = () => {
  const measureRender = useCallback((componentName: string) => {
    const startTime = performance.now();
    
    return () => {
      const endTime = performance.now();
      const renderTime = endTime - startTime;
      
      if (renderTime > 16) { // > 1 frame at 60fps
        console.warn(`Slow render detected: ${componentName} took ${renderTime}ms`);
      }
    };
  }, []);
  
  return { measureRender };
};
```

---

*The frontend architecture emphasizes type safety, performance, and maintainability while providing an intuitive user experience for laboratory professionals.*

*Context added by Giga storage-management-flows* 
