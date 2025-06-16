# Spreadsheet Frontend - Quick Start Guide

You now have a complete frontend interface for the spreadsheet processing service! Here's what you can do:

## 🚀 Getting Started

1. **Start the servers:**
   ```bash
   # Terminal 1: Start the backend
   cd lab_manager
   cargo run

   # Terminal 2: Start the frontend
   cd lab_manager/frontend
   npm run dev
   ```

2. **Access the interface:**
   - Open http://localhost:5173 in your browser
   - Navigate to "Spreadsheets" in the sidebar menu

## ✨ Features

### 📁 File Upload
- **Drag & Drop Interface**: Simply drag CSV/Excel files onto the upload area
- **Supported Formats**: CSV, XLSX, XLS
- **Excel Sheet Selection**: Choose specific sheets from Excel files
- **User Attribution**: Track who uploaded each file
- **Real-time Processing**: See upload progress and instant feedback

### 🔍 Advanced Search
- **Global Search**: Search across ALL uploaded spreadsheet data
- **Full-text Search**: Find data using natural language queries
- **Column Filters**: Filter by specific column values (e.g., Department=Oncology)
- **Combined Queries**: Mix text search with column filters
- **Real-time Results**: Instant search with debouncing
- **Pagination**: Handle large result sets efficiently

### 📊 Data Management
- **Dataset Overview**: See all uploaded files with statistics
- **File Details**: View file size, row count, upload status, and metadata
- **Data Viewer**: Browse individual dataset contents with filtering
- **Status Tracking**: Monitor upload progress and error states
- **Quick Actions**: Delete datasets, view data, search within files

### 💡 Smart Interface
- **Responsive Design**: Works on desktop, tablet, and mobile
- **Loading States**: Clear feedback during operations
- **Error Handling**: Helpful error messages and recovery options
- **Modern UI**: Clean, professional interface using TailwindCSS
- **Accessibility**: Keyboard navigation and screen reader support

## 🎯 Usage Examples

### Upload Laboratory Data
1. Click "Upload File" button
2. Drag your CSV/Excel file or browse to select
3. For Excel files, optionally specify sheet name
4. Add your name/email as uploader
5. Click "Upload File" - data is automatically processed and indexed

### Search Across All Data
1. Click "Search Data" button
2. Use the search bar for text queries like:
   - "LAB20240001" - find specific sample IDs
   - "Oncology high priority" - find records matching criteria
   - "RNA-Seq" - find analysis types
3. Use column filters for precise filtering:
   - Department: Oncology
   - Priority: High
   - Sample_Type: Blood

### View Dataset Contents
1. In the datasets table, click "View Data" for any completed upload
2. Browse paginated data with 50 rows per page
3. Use search to filter within the dataset
4. Apply column filters for specific data

### Real Laboratory Workflow
Perfect for processing your laboratory files like:
- `HTSF-Library-Prep-Queue-7-10-17_AHP_v5.xlsx`
- `Master AAMELIO_8879 KAPA mRNA Info GA-1.xlsx` 
- `QAQC_240520.xlsx`
- Custom CSV exports from lab instruments

## 🔧 API Integration

The frontend communicates with these backend endpoints:
- `POST /api/spreadsheets/upload` - File upload
- `GET /api/spreadsheets/search` - Search across all data
- `GET /api/spreadsheets/datasets` - List all datasets
- `GET /api/spreadsheets/datasets/:id` - Get specific dataset
- `DELETE /api/spreadsheets/datasets/:id` - Delete dataset
- `GET /api/spreadsheets/health` - Health check

## 📈 Performance

- **Efficient Pagination**: Handles millions of records
- **Indexed Search**: PostgreSQL full-text search for speed
- **Bulk Operations**: Optimized database insertions
- **Debounced Search**: Reduces server load during typing
- **Caching**: React Query caches results for better UX

## 🎨 UI Components Created

1. **`Spreadsheets.tsx`** - Main page with dataset management
2. **`FileUploadModal.tsx`** - Drag & drop file upload interface  
3. **`SpreadsheetSearchModal.tsx`** - Advanced search functionality
4. **`SpreadsheetDataViewer.tsx`** - Dataset content browser

## 🔐 Security & Validation

- File type validation (CSV, XLSX, XLS only)
- File size limits 
- SQL injection protection
- Input sanitization
- Error boundary handling

Your spreadsheet processing system is now ready for production use in your laboratory workflow!

## 🚨 Troubleshooting

### Frontend won't build
- Make sure you're in the `frontend` directory
- Run `npm install` to install dependencies
- The existing TypeScript errors in `SequencingJobDetails` are pre-existing and don't affect the new feature

### Backend API errors
- Ensure database migrations have run: `sqlx migrate run`
- Check the backend is running on port 3000
- Verify database connection in environment variables

### File upload fails
- Check file format (CSV, XLSX, XLS)
- Ensure file isn't corrupted
- Check backend logs for detailed error messages 
