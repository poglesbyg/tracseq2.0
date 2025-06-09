# Lab Manager Frontend

A modern React TypeScript frontend for the Laboratory Management System with AI-powered document processing.

## Features

### 🧬 Core Laboratory Management
- **Dashboard** - Overview of laboratory operations
- **Templates** - Manage laboratory spreadsheet templates
- **Samples** - Track biological samples through their lifecycle
- **Sequencing** - Manage sequencing jobs and workflows
- **Storage** - Monitor sample storage and capacity
- **Reports** - Generate laboratory analytics and reports

### 🤖 AI-Powered Document Processing
- **RAG Submissions** - Upload laboratory documents for automatic data extraction
  - Support for PDF, DOCX, and TXT documents
  - AI-powered sample information extraction
  - Confidence scoring and validation
  - Natural language querying of data
  - Preview before creating samples
  - Automatic barcode generation

## Getting Started

### Prerequisites
- Node.js 18+
- Backend Lab Manager API running on port 3000

### Installation
```bash
npm install
```

### Development
```bash
npm run dev
```
The application will be available at `http://localhost:5173`

### Build
```bash
npm run build
```

### Testing
```bash
npm run test
```

## RAG Submissions Usage

1. **Navigate to AI Submissions** in the sidebar
2. **Upload Document**: Drag and drop or select a laboratory document
3. **Configure Settings**: 
   - Set confidence threshold (0.5-1.0)
   - Enable/disable automatic sample creation
4. **Preview or Process**: 
   - Use "Preview" to see extracted data without creating samples
   - Use "Process & Extract" to create samples immediately
5. **Natural Language Queries**: Ask questions about your data in plain English

### Supported Document Types
- **PDF**: Laboratory forms, submission sheets
- **DOCX**: Microsoft Word documents with sample information
- **TXT**: Plain text files with structured laboratory data

### AI Features
- **Automatic Data Extraction**: Extracts sample names, barcodes, storage locations
- **Confidence Scoring**: Provides reliability scores for extracted data
- **Smart Validation**: Identifies missing or unclear information
- **Natural Language Interface**: Query your data using everyday language

## Technology Stack
- **React 18** with TypeScript
- **Vite** for fast development and building
- **Tailwind CSS** for styling
- **TanStack React Query** for data fetching
- **Axios** for API communication
- **React Router DOM** for navigation
- **Headless UI** for accessible components

## API Integration
The frontend communicates with the Lab Manager backend via:
- REST API endpoints (`/api/*`)
- File upload for document processing
- Real-time query processing
- Health monitoring endpoints

## Development

### Project Structure

- `src/components/` - Reusable UI components
- `src/pages/` - Page components
- `src/hooks/` - Custom React hooks
- `src/api/` - API client and types

### Available Scripts

- `npm run dev` - Start development server
- `npm run build` - Build for production
- `npm run preview` - Preview production build
- `npm run lint` - Run ESLint
- `npm run type-check` - Run TypeScript type checking

## Contributing

1. Create a feature branch
2. Make your changes
3. Submit a pull request

## License

This project is licensed under the MIT License.
