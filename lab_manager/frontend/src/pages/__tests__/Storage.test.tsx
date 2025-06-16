import { render, screen } from '@testing-library/react';
import Storage from '../Storage';

// Mock the StorageManagement component
jest.mock('../../components/StorageManagement', () => {
  return function MockStorageManagement() {
    return <div data-testid="storage-management">Storage Management Component</div>;
  };
});

describe('Storage', () => {
  it('renders the StorageManagement component', () => {
    render(<Storage />);
    
    expect(screen.getByTestId('storage-management')).toBeInTheDocument();
  });

  it('has correct layout classes', () => {
    render(<Storage />);
    
    const container = screen.getByTestId('storage-management').parentElement;
    expect(container).toHaveClass('px-4', 'sm:px-6', 'lg:px-8');
  });
}); 
