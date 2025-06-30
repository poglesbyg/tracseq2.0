# Testing Guide for Lab Manager Frontend

## Overview

This document covers the comprehensive test suite for the Lab Manager frontend, with special focus on the enhanced spreadsheet functionality.

## Test Structure

```
frontend/src/
├── components/
│   └── __tests__/
│       └── SpreadsheetDataViewer.test.tsx   # Enhanced data viewer tests
├── pages/
│   └── __tests__/
│       └── Spreadsheets.test.tsx            # Spreadsheet page tests
└── setupTests.ts                            # Global test configuration
```

## Running Tests

### All Tests
```bash
npm test
```

### Spreadsheet-Specific Tests
```bash
npm run test:spreadsheets
```

### Watch Mode (for development)
```bash
npm run test:watch
```

### Coverage Report
```bash
npm run test:coverage
```

## Test Coverage

### SpreadsheetDataViewer Component

**✅ Features Tested:**

1. **Basic Rendering**
   - Component initialization with dataset props
   - Header information display
   - Action button presence
   - Close functionality

2. **Data Loading & Display**
   - Loading states
   - Data table rendering
   - Column headers with type indicators
   - Cell value formatting by data type

3. **Statistics Panel**
   - Toggle functionality
   - Column statistics calculation
   - Data type detection
   - Min/max/average for numeric columns

4. **Sorting Functionality**
   - Column header click sorting
   - Sort direction cycling (asc → desc → none)
   - Type-aware sorting (numeric, date, text)
   - Sort indicators in UI

5. **Row Selection**
   - Individual row selection
   - Select all functionality
   - Selection counter display
   - Visual feedback for selected rows

6. **Export Functionality**
   - Export dropdown visibility
   - CSV export simulation
   - JSON export simulation
   - Filename generation

7. **Fullscreen Mode**
   - Toggle fullscreen state
   - UI layout adaptation

8. **Search & Filtering**
   - Search input functionality
   - Rows per page selection
   - Filter toggle buttons
   - Search term handling

9. **Data Type Detection**
   - Number formatting and detection
   - Email link creation
   - URL link creation
   - Boolean badge formatting
   - Date formatting

10. **Error Handling**
    - Network error display
    - Empty data states
    - Missing column headers handling

### Spreadsheets Page Component

**✅ Features Tested:**

1. **Basic Rendering**
   - Page header and description
   - Action buttons (Upload, Search)
   - Table structure

2. **Dataset Display**
   - Dataset listing in table format
   - File type badges (CSV, XLSX)
   - File size formatting
   - Row/column counts
   - Upload status with correct colors
   - User information display
   - Creation dates

3. **Dataset Interaction**
   - View Data button visibility (completed only)
   - Click-to-view functionality
   - Row hover effects
   - Status-based tooltips
   - Event propagation handling

4. **Case Sensitivity Fixes**
   - Mixed-case status handling
   - Case-insensitive comparison
   - Statistics calculation accuracy

5. **Modal Interactions**
   - Upload modal opening/closing
   - Search modal opening/closing
   - Data viewer opening/closing

6. **Error Handling**
   - Network error display
   - Failed upload indicators
   - Empty state handling

## Mock Setup

### Axios Mocking
```typescript
jest.mock('axios');
const mockedAxios = axios as jest.Mocked<typeof axios>;
```

### Component Mocking
- FileUploadModal → Simple test double
- SpreadsheetSearchModal → Simple test double  
- SpreadsheetDataViewer → Simple test double

### Global Mocks
- URL.createObjectURL/revokeObjectURL for export tests
- window.confirm for delete confirmation
- window.matchMedia for responsive design

## Test Data

### Mock Dataset Structure
```typescript
{
  id: string;
  filename: string;
  original_filename: string;
  file_type: 'csv' | 'xlsx';
  file_size: number;
  total_rows: number;
  total_columns: number;
  column_headers: string[];
  upload_status: 'Completed' | 'Processing' | 'Failed';
  error_message: string | null;
  uploaded_by: string;
  created_at: string;
  updated_at: string;
  metadata: object;
}
```

### Mock Data Response
```typescript
{
  records: Array<{
    id: string;
    row_number: number;
    row_data: Record<string, any>;
    created_at: string;
  }>;
  total_count: number;
}
```

## Best Practices

### 1. Test Isolation
- Each test is isolated with `beforeEach` cleanup
- Mock reset between tests
- Independent test data

### 2. Async Testing
- Use `waitFor()` for async operations
- Proper loading state testing
- Error boundary testing

### 3. User Interaction
- Test user events (clicks, typing)
- Verify UI responses
- Check state changes

### 4. Mock Strategy
- Mock external dependencies (axios)
- Mock complex child components
- Keep mocks simple and focused

## Adding New Tests

### For New Features
1. Create test file in appropriate `__tests__` directory
2. Follow existing naming convention: `Component.test.tsx`
3. Use describe blocks to group related tests
4. Include all interaction states (loading, success, error)

### Test Template
```typescript
import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import YourComponent from '../YourComponent';

// Mock setup
jest.mock('axios');

const createTestWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } }
  });
  
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
};

describe('YourComponent', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders correctly', () => {
    const TestWrapper = createTestWrapper();
    render(
      <TestWrapper>
        <YourComponent />
      </TestWrapper>
    );
    
    expect(screen.getByText('Expected Text')).toBeInTheDocument();
  });
});
```

## Coverage Goals

- **Statements**: > 80%
- **Branches**: > 75%
- **Functions**: > 80%
- **Lines**: > 80%

## Troubleshooting

### Common Issues

1. **Tests timing out**: Increase timeout or use `waitFor()`
2. **Mock not working**: Ensure correct import path
3. **Query client errors**: Wrap component in test provider
4. **Event not firing**: Check element selection and event type

### Debugging Tips

1. Use `screen.debug()` to see rendered DOM
2. Check console for React warnings
3. Verify mock calls with `expect(mock).toHaveBeenCalledWith()`
4. Use `--verbose` flag for detailed test output

## Performance

### Test Performance Tips
- Use `jest.clearAllMocks()` instead of recreating mocks
- Minimize component re-renders in tests
- Use `screen.getAllBy*` only when necessary
- Avoid testing implementation details

---

## Summary

The test suite provides comprehensive coverage of:
- ✅ Enhanced data viewer with all new features
- ✅ Spreadsheet page interactions
- ✅ Case sensitivity fixes
- ✅ Error handling and edge cases
- ✅ User interaction flows
- ✅ Data type detection and formatting

Run `npm run test:spreadsheets` to verify all spreadsheet functionality is working correctly! 
