# Lab Manager Frontend

This is the frontend application for the Lab Manager system, built with React, TypeScript, and Tailwind CSS.

## Features

- Modern, responsive UI built with Tailwind CSS
- Type-safe development with TypeScript
- Efficient state management with React Query
- File upload for spreadsheet templates
- Sample submission workflow
- Sequencing job management
- Real-time status updates

## Getting Started

### Prerequisites

- Node.js 16.x or later
- npm 7.x or later

### Installation

1. Install dependencies:
   ```bash
   npm install
   ```

2. Start the development server:
   ```bash
   npm run dev
   ```

The application will be available at `http://localhost:5173`.

### Building for Production

To create a production build:

```bash
npm run build
```

The built files will be in the `dist` directory.

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

## API Integration

The frontend communicates with the backend API through the following endpoints:

- `/api/templates` - Template management
- `/api/samples` - Sample submission
- `/api/sequencing/jobs` - Sequencing job management

API requests are automatically proxied to the backend server during development.

## Contributing

1. Create a feature branch
2. Make your changes
3. Submit a pull request

## License

This project is licensed under the MIT License.
