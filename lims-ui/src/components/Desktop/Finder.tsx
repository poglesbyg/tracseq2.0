import React, { useState } from 'react';
import { 
  FolderIcon, 
  DocumentIcon, 
  ChevronRightIcon,
  BeakerIcon,
  DocumentTextIcon,
  ViewColumnsIcon,
  Squares2X2Icon,
  ListBulletIcon,
  MagnifyingGlassIcon,
  ChevronLeftIcon
} from '@heroicons/react/24/outline';
import { FolderIcon as FolderSolidIcon } from '@heroicons/react/24/solid';

export interface FileSystemItem {
  id: string;
  name: string;
  type: 'folder' | 'sample' | 'template' | 'document' | 'project' | 'report';
  parent: string | null;
  created: Date;
  modified: Date;
  size?: number;
  metadata?: Record<string, unknown>;
  children?: string[];
}

interface FinderProps {
  items: FileSystemItem[];
  onItemOpen: (item: FileSystemItem) => void;
  onItemSelect: (item: FileSystemItem) => void;
  selectedItemId?: string;
}

type ViewMode = 'icon' | 'list' | 'column';

export const Finder: React.FC<FinderProps> = ({ items, onItemOpen, onItemSelect, selectedItemId }) => {
  const [viewMode, setViewMode] = useState<ViewMode>('icon');
  const [currentFolderId, setCurrentFolderId] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [columnPath, setColumnPath] = useState<string[]>([]);
  const [navigationHistory, setNavigationHistory] = useState<(string | null)[]>([null]);
  const [historyIndex, setHistoryIndex] = useState(0);
  const [showRecent, setShowRecent] = useState(false);

  const getItemById = (id: string) => items.find(item => item.id === id);
  
  const getChildren = (parentId: string | null) => {
    return items.filter(item => item.parent === parentId);
  };

  const getFilteredItems = () => {
    if (showRecent) {
      // Show items sorted by modified date (most recent first)
      return [...items]
        .filter(item => item.type !== 'folder')
        .sort((a, b) => b.modified.getTime() - a.modified.getTime())
        .slice(0, 20); // Show last 20 items
    }
    
    if (!searchQuery) return getChildren(currentFolderId);
    
    return items.filter(item => 
      item.name.toLowerCase().includes(searchQuery.toLowerCase()) &&
      (currentFolderId === null || item.parent === currentFolderId)
    );
  };

  const getIcon = (item: FileSystemItem) => {
    switch (item.type) {
      case 'folder':
        return <FolderSolidIcon className="w-12 h-12 text-blue-500" />;
      case 'sample':
        return <BeakerIcon className="w-12 h-12 text-green-500" />;
      case 'template':
        return <DocumentTextIcon className="w-12 h-12 text-purple-500" />;
      case 'project':
        return <FolderIcon className="w-12 h-12 text-amber-500" />;
      case 'report':
        return <DocumentTextIcon className="w-12 h-12 text-orange-500" />;
      default:
        return <DocumentIcon className="w-12 h-12 text-gray-500" />;
    }
  };

  const getSmallIcon = (item: FileSystemItem) => {
    switch (item.type) {
      case 'folder':
        return <FolderIcon className="w-5 h-5 text-blue-500" />;
      case 'sample':
        return <BeakerIcon className="w-5 h-5 text-green-500" />;
      case 'template':
        return <DocumentTextIcon className="w-5 h-5 text-purple-500" />;
      case 'project':
        return <FolderIcon className="w-5 h-5 text-amber-500" />;
      case 'report':
        return <DocumentTextIcon className="w-5 h-5 text-orange-500" />;
      default:
        return <DocumentIcon className="w-5 h-5 text-gray-500" />;
    }
  };

  const navigateTo = (folderId: string | null) => {
    if (folderId !== currentFolderId) {
      setCurrentFolderId(folderId);
      setShowRecent(false);
      
      // Update navigation history
      const newHistory = navigationHistory.slice(0, historyIndex + 1);
      newHistory.push(folderId);
      setNavigationHistory(newHistory);
      setHistoryIndex(newHistory.length - 1);
    }
  };

  const handleBack = () => {
    if (historyIndex > 0) {
      const newIndex = historyIndex - 1;
      setHistoryIndex(newIndex);
      setCurrentFolderId(navigationHistory[newIndex]);
      setShowRecent(false);
    }
  };

  const handleForward = () => {
    if (historyIndex < navigationHistory.length - 1) {
      const newIndex = historyIndex + 1;
      setHistoryIndex(newIndex);
      setCurrentFolderId(navigationHistory[newIndex]);
      setShowRecent(false);
    }
  };

  const handleItemClick = (item: FileSystemItem) => {
    onItemSelect(item);
    if (item.type === 'folder') {
      navigateTo(item.id);
      if (viewMode === 'column') {
        const currentIndex = columnPath.indexOf(item.parent || '');
        setColumnPath([...columnPath.slice(0, currentIndex + 1), item.id]);
      }
    }
  };

  const handleItemDoubleClick = (item: FileSystemItem) => {
    if (item.type === 'folder') {
      navigateTo(item.id);
    } else {
      onItemOpen(item);
    }
  };

  const renderSidebar = () => (
    <div className="w-48 bg-gray-50 border-r border-gray-200 p-3">
      <div className="space-y-1">
        <button 
          onClick={() => {
            navigateTo(null);
            setShowRecent(false);
          }}
          className={`w-full text-left px-3 py-1.5 rounded-md text-sm ${
            currentFolderId === null && !showRecent ? 'bg-blue-100 text-blue-700' : 'hover:bg-gray-100'
          }`}
        >
          All Files
        </button>
        <button 
          onClick={() => {
            setShowRecent(true);
            setCurrentFolderId(null);
          }}
          className={`w-full text-left px-3 py-1.5 rounded-md text-sm ${
            showRecent ? 'bg-blue-100 text-blue-700' : 'hover:bg-gray-100'
          }`}
        >
          Recent
        </button>
        <button 
          onClick={() => navigateTo('samples-folder')}
          className={`w-full text-left px-3 py-1.5 rounded-md text-sm ${
            currentFolderId === 'samples-folder' ? 'bg-blue-100 text-blue-700' : 'hover:bg-gray-100'
          }`}
        >
          Samples
        </button>
        <button 
          onClick={() => navigateTo('templates-folder')}
          className={`w-full text-left px-3 py-1.5 rounded-md text-sm ${
            currentFolderId === 'templates-folder' ? 'bg-blue-100 text-blue-700' : 'hover:bg-gray-100'
          }`}
        >
          Templates
        </button>
        <button 
          onClick={() => navigateTo('projects-folder')}
          className={`w-full text-left px-3 py-1.5 rounded-md text-sm ${
            currentFolderId === 'projects-folder' ? 'bg-blue-100 text-blue-700' : 'hover:bg-gray-100'
          }`}
        >
          Projects
        </button>
        <button 
          onClick={() => navigateTo('reports-folder')}
          className={`w-full text-left px-3 py-1.5 rounded-md text-sm ${
            currentFolderId === 'reports-folder' ? 'bg-blue-100 text-blue-700' : 'hover:bg-gray-100'
          }`}
        >
          Reports
        </button>
      </div>
    </div>
  );

  const renderIconView = () => (
    <div className="grid grid-cols-6 gap-4 p-4">
      {getFilteredItems().map(item => (
        <div
          key={item.id}
          onClick={() => handleItemClick(item)}
          onDoubleClick={() => handleItemDoubleClick(item)}
          className={`flex flex-col items-center p-3 rounded-lg cursor-pointer ${
            selectedItemId === item.id ? 'bg-blue-100' : 'hover:bg-gray-100'
          }`}
        >
          {getIcon(item)}
          <span className="mt-2 text-xs text-center text-gray-700 line-clamp-2">
            {item.name}
          </span>
        </div>
      ))}
    </div>
  );

  const renderListView = () => (
    <div className="p-2">
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b text-left text-gray-500">
            <th className="pb-2 pl-4 font-medium">Name</th>
            <th className="pb-2 font-medium">Modified</th>
            <th className="pb-2 font-medium">Size</th>
            <th className="pb-2 font-medium">Type</th>
          </tr>
        </thead>
        <tbody>
          {getFilteredItems().map(item => (
            <tr
              key={item.id}
              onClick={() => handleItemClick(item)}
              onDoubleClick={() => handleItemDoubleClick(item)}
              className={`border-b cursor-pointer ${
                selectedItemId === item.id ? 'bg-blue-50' : 'hover:bg-gray-50'
              }`}
            >
              <td className="py-2 pl-4 flex items-center gap-2">
                {getSmallIcon(item)}
                <span className="text-gray-900">{item.name}</span>
              </td>
              <td className="py-2 text-gray-500">
                {item.modified.toLocaleDateString()}
              </td>
              <td className="py-2 text-gray-500">
                {item.size ? `${(item.size / 1024).toFixed(1)} KB` : '--'}
              </td>
              <td className="py-2 text-gray-500 capitalize">{item.type}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );

  const renderColumnView = () => {
    const columns = [null, ...columnPath];
    
    return (
      <div className="flex h-full">
        {columns.map((folderId, index) => (
          <div key={folderId || 'root'} className="w-64 border-r border-gray-200 overflow-y-auto">
            <div className="py-1">
              {getChildren(folderId).map(item => (
                <div
                  key={item.id}
                  onClick={() => handleItemClick(item)}
                  onDoubleClick={() => handleItemDoubleClick(item)}
                  className={`px-4 py-1.5 flex items-center gap-2 cursor-pointer ${
                    selectedItemId === item.id 
                      ? 'bg-blue-500 text-white' 
                      : columnPath[index] === item.id 
                      ? 'bg-gray-100'
                      : 'hover:bg-gray-50'
                  }`}
                >
                  {getSmallIcon(item)}
                  <span className="flex-1 text-sm truncate">{item.name}</span>
                  {item.type === 'folder' && (
                    <ChevronRightIcon className="w-4 h-4 opacity-50" />
                  )}
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    );
  };

  return (
    <div className="flex flex-col h-full bg-white">
      {/* Toolbar */}
      <div className="h-12 border-b border-gray-200 flex items-center px-4 gap-4">
        {/* Navigation */}
        <div className="flex items-center gap-2">
          <button 
            onClick={handleBack}
            disabled={historyIndex === 0}
            className={`p-1 rounded ${
              historyIndex === 0 
                ? 'text-gray-300 cursor-not-allowed' 
                : 'hover:bg-gray-100 text-gray-600'
            }`}
          >
            <ChevronLeftIcon className="w-5 h-5" />
          </button>
          <button 
            onClick={handleForward}
            disabled={historyIndex === navigationHistory.length - 1}
            className={`p-1 rounded ${
              historyIndex === navigationHistory.length - 1 
                ? 'text-gray-300 cursor-not-allowed' 
                : 'hover:bg-gray-100 text-gray-600'
            }`}
          >
            <ChevronRightIcon className="w-5 h-5" />
          </button>
        </div>

        {/* Path */}
        <div className="flex-1 flex items-center gap-1 text-sm text-gray-600">
          <FolderIcon className="w-4 h-4" />
          <span>Laboratory Files</span>
          {showRecent && (
            <>
              <ChevronRightIcon className="w-3 h-3" />
              <span>Recent</span>
            </>
          )}
          {currentFolderId && !showRecent && (
            <>
              <ChevronRightIcon className="w-3 h-3" />
              <span>{getItemById(currentFolderId)?.name}</span>
            </>
          )}
        </div>

        {/* Search */}
        <div className="relative">
          <MagnifyingGlassIcon className="absolute left-2 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search"
            className="pl-8 pr-3 py-1 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-1 focus:ring-blue-500"
          />
        </div>

        {/* View Mode */}
        <div className="flex items-center gap-1 border-l pl-4">
          <button
            onClick={() => setViewMode('icon')}
            className={`p-1 rounded ${viewMode === 'icon' ? 'bg-gray-200' : 'hover:bg-gray-100'}`}
          >
            <Squares2X2Icon className="w-4 h-4 text-gray-600" />
          </button>
          <button
            onClick={() => setViewMode('list')}
            className={`p-1 rounded ${viewMode === 'list' ? 'bg-gray-200' : 'hover:bg-gray-100'}`}
          >
            <ListBulletIcon className="w-4 h-4 text-gray-600" />
          </button>
          <button
            onClick={() => setViewMode('column')}
            className={`p-1 rounded ${viewMode === 'column' ? 'bg-gray-200' : 'hover:bg-gray-100'}`}
          >
            <ViewColumnsIcon className="w-4 h-4 text-gray-600" />
          </button>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 flex overflow-hidden">
        {renderSidebar()}
        <div className="flex-1 overflow-auto">
          {viewMode === 'icon' && renderIconView()}
          {viewMode === 'list' && renderListView()}
          {viewMode === 'column' && renderColumnView()}
        </div>
      </div>
    </div>
  );
};