import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import api from '../utils/axios';
import {
  FolderIcon,
  FolderOpenIcon,
  DocumentIcon,
  PlusIcon,
  MagnifyingGlassIcon,
  ChevronRightIcon,
  ChevronDownIcon,
  ArrowDownTrayIcon,
  CheckBadgeIcon,
  ClockIcon,
  UserGroupIcon,
  ArrowPathIcon,
} from '@heroicons/react/24/outline';
import { format } from 'date-fns';

interface Project {
  id: string;
  project_code: string;
  name: string;
  description?: string;
  status: 'planning' | 'active' | 'on_hold' | 'completed' | 'cancelled';
  priority: 'low' | 'medium' | 'high' | 'urgent';
  principal_investigator_name?: string;
  start_date?: string;
  target_end_date?: string;
  batch_count?: number;
  team_member_count?: number;
}

interface Batch {
  id: string;
  batch_number: string;
  project_id: string;
  batch_type: 'sample_receipt' | 'library_prep' | 'sequencing' | 'analysis';
  status: 'created' | 'in_progress' | 'pending_approval' | 'approved' | 'completed' | 'failed';
  sample_count: number;
  created_at: string;
}

interface ProjectFile {
  id: string;
  name: string;
  file_type: 'file' | 'folder';
  file_extension?: string;
  file_size_bytes?: number;
  parent_folder_id?: string;
  created_at: string;
  children?: ProjectFile[];
}

interface TemplateFile {
  id: string;
  name: string;
  category: string;
  file_type: string;
  version: string;
  description?: string;
  download_count: number;
}

export default function ProjectManagement() {
  const [selectedTab, setSelectedTab] = useState<'projects' | 'batches' | 'files' | 'templates'>('projects');
  const [selectedProject, setSelectedProject] = useState<Project | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set());

  // Fetch projects
  const { data: projects, isLoading: isLoadingProjects } = useQuery<Project[]>({
    queryKey: ['projects', searchTerm],
    queryFn: async () => {
      const params = searchTerm ? { search: searchTerm } : {};
      const response = await api.get('/api/projects', { params });
      return response.data;
    },
  });

  // Fetch batches
  const { data: batches, isLoading: isLoadingBatches } = useQuery<Batch[]>({
    queryKey: ['batches', searchTerm],
    queryFn: async () => {
      const params = searchTerm ? { batch_number: searchTerm } : {};
      const response = await api.get('/api/batches', { params });
      return response.data;
    },
    enabled: selectedTab === 'batches',
  });

  // Fetch project files
  const { data: projectFiles } = useQuery<ProjectFile[]>({
    queryKey: ['project-files', selectedProject?.id],
    queryFn: async () => {
      const response = await api.get(`/api/projects/${selectedProject?.id}/files`);
      return response.data;
    },
    enabled: !!selectedProject && selectedTab === 'files',
  });

  // Fetch templates
  const { data: templates } = useQuery<TemplateFile[]>({
    queryKey: ['templates'],
    queryFn: async () => {
      const response = await api.get('/api/templates/repository');
      return response.data;
    },
    enabled: selectedTab === 'templates',
  });

  const toggleFolder = (folderId: string) => {
    const newExpanded = new Set(expandedFolders);
    if (newExpanded.has(folderId)) {
      newExpanded.delete(folderId);
    } else {
      newExpanded.add(folderId);
    }
    setExpandedFolders(newExpanded);
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active':
      case 'completed':
      case 'approved':
        return 'bg-green-100 text-green-800';
      case 'in_progress':
      case 'pending_approval':
        return 'bg-yellow-100 text-yellow-800';
      case 'on_hold':
      case 'created':
        return 'bg-gray-100 text-gray-800';
      case 'failed':
      case 'cancelled':
        return 'bg-red-100 text-red-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'urgent':
        return 'text-red-600';
      case 'high':
        return 'text-orange-600';
      case 'medium':
        return 'text-yellow-600';
      case 'low':
        return 'text-green-600';
      default:
        return 'text-gray-600';
    }
  };

  const renderFileTree = (files: ProjectFile[], level = 0) => {
    return files.map((file) => (
      <div key={file.id} style={{ paddingLeft: `${level * 20}px` }}>
        <div
          className="flex items-center py-2 px-2 hover:bg-gray-50 cursor-pointer rounded"
          onClick={() => file.file_type === 'folder' && toggleFolder(file.id)}
        >
          {file.file_type === 'folder' ? (
            <>
              {expandedFolders.has(file.id) ? (
                <ChevronDownIcon className="h-4 w-4 text-gray-400 mr-1" />
              ) : (
                <ChevronRightIcon className="h-4 w-4 text-gray-400 mr-1" />
              )}
              {expandedFolders.has(file.id) ? (
                <FolderOpenIcon className="h-5 w-5 text-yellow-500 mr-2" />
              ) : (
                <FolderIcon className="h-5 w-5 text-gray-400 mr-2" />
              )}
            </>
          ) : (
            <>
              <div className="w-5" />
              <DocumentIcon className="h-5 w-5 text-gray-400 mr-2" />
            </>
          )}
          <span className="text-sm text-gray-900 flex-1">{file.name}</span>
          {file.file_size_bytes && (
            <span className="text-xs text-gray-500">
              {(file.file_size_bytes / 1024).toFixed(1)} KB
            </span>
          )}
        </div>
        {file.file_type === 'folder' && expandedFolders.has(file.id) && file.children && (
          <div>{renderFileTree(file.children, level + 1)}</div>
        )}
      </div>
    ));
  };

  return (
    <div className="px-4 sm:px-6 lg:px-8">
      {/* Header */}
      <div className="sm:flex sm:items-center">
        <div className="sm:flex-auto">
          <h1 className="text-xl font-semibold text-gray-900">Project Management</h1>
          <p className="mt-2 text-sm text-gray-700">
            Manage projects, track batches, and organize project files
          </p>
        </div>
        <div className="mt-4 sm:mt-0 sm:ml-16 sm:flex-none">
          <button
            type="button"
            className="inline-flex items-center justify-center rounded-md border border-transparent bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 sm:w-auto"
          >
            <PlusIcon className="-ml-1 mr-2 h-5 w-5" />
            New Project
          </button>
        </div>
      </div>

      {/* Tabs */}
      <div className="mt-4">
        <div className="sm:hidden">
          <select
            value={selectedTab}
            onChange={(e) => setSelectedTab(e.target.value as 'projects' | 'batches' | 'files' | 'templates')}
            className="block w-full rounded-md border-gray-300 focus:border-indigo-500 focus:ring-indigo-500"
          >
            <option value="projects">Projects</option>
            <option value="batches">Batches</option>
            <option value="files">Files</option>
            <option value="templates">Templates</option>
          </select>
        </div>
        <div className="hidden sm:block">
          <nav className="flex space-x-8" aria-label="Tabs">
            {['projects', 'batches', 'files', 'templates'].map((tab) => (
              <button
                key={tab}
                onClick={() => setSelectedTab(tab as 'projects' | 'batches' | 'files' | 'templates')}
                className={`${
                  selectedTab === tab
                    ? 'border-indigo-500 text-indigo-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                } whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm capitalize`}
              >
                {tab}
              </button>
            ))}
          </nav>
        </div>
      </div>

      {/* Search bar */}
      {(selectedTab === 'projects' || selectedTab === 'batches') && (
        <div className="mt-6">
          <div className="max-w-lg">
            <label htmlFor="search" className="sr-only">
              Search
            </label>
            <div className="relative">
              <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <MagnifyingGlassIcon className="h-5 w-5 text-gray-400" />
              </div>
              <input
                type="search"
                name="search"
                id="search"
                className="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                placeholder={selectedTab === 'batches' ? 'Search by batch number...' : 'Search projects...'}
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
              />
            </div>
          </div>
        </div>
      )}

      {/* Content */}
      <div className="mt-8">
        {selectedTab === 'projects' && (
          <div className="bg-white shadow overflow-hidden sm:rounded-md">
            {isLoadingProjects ? (
              <div className="flex justify-center py-8">
                <ArrowPathIcon className="h-8 w-8 animate-spin text-indigo-600" />
              </div>
            ) : (
              <ul className="divide-y divide-gray-200">
                {projects?.map((project) => (
                  <li key={project.id}>
                    <div className="px-4 py-4 sm:px-6 hover:bg-gray-50 cursor-pointer"
                         onClick={() => setSelectedProject(project)}>
                      <div className="flex items-center justify-between">
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center">
                            <p className="text-sm font-medium text-gray-900 truncate">
                              {project.project_code} - {project.name}
                            </p>
                            <span className={`ml-2 text-xs font-medium ${getPriorityColor(project.priority)}`}>
                              {project.priority}
                            </span>
                          </div>
                          <div className="mt-2 flex items-center text-sm text-gray-500">
                            <UserGroupIcon className="flex-shrink-0 mr-1.5 h-4 w-4 text-gray-400" />
                            {project.principal_investigator_name || 'No PI assigned'}
                            {project.target_end_date && (
                              <>
                                <span className="mx-2">•</span>
                                <ClockIcon className="flex-shrink-0 mr-1.5 h-4 w-4 text-gray-400" />
                                Due {format(new Date(project.target_end_date), 'MMM dd, yyyy')}
                              </>
                            )}
                          </div>
                        </div>
                        <div className="flex items-center space-x-4">
                          <div className="text-sm text-gray-500">
                            {project.batch_count || 0} batches
                          </div>
                          <span
                            className={`inline-flex rounded-full px-2 text-xs font-semibold leading-5 ${getStatusColor(
                              project.status
                            )}`}
                          >
                            {project.status}
                          </span>
                        </div>
                      </div>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}

        {selectedTab === 'batches' && (
          <div className="bg-white shadow overflow-hidden sm:rounded-md">
            {isLoadingBatches ? (
              <div className="flex justify-center py-8">
                <ArrowPathIcon className="h-8 w-8 animate-spin text-indigo-600" />
              </div>
            ) : (
              <ul className="divide-y divide-gray-200">
                {batches?.map((batch) => (
                  <li key={batch.id}>
                    <div className="px-4 py-4 sm:px-6 hover:bg-gray-50">
                      <div className="flex items-center justify-between">
                        <div>
                          <p className="text-sm font-medium text-gray-900">
                            Batch: {batch.batch_number}
                          </p>
                          <p className="text-sm text-gray-500">
                            Type: {batch.batch_type.replace('_', ' ')} • {batch.sample_count} samples
                          </p>
                          <p className="text-sm text-gray-500">
                            Created {format(new Date(batch.created_at), 'MMM dd, yyyy')}
                          </p>
                        </div>
                        <div className="flex items-center space-x-2">
                          {batch.status === 'pending_approval' && (
                            <CheckBadgeIcon className="h-5 w-5 text-yellow-500" />
                          )}
                          <span
                            className={`inline-flex rounded-full px-2 text-xs font-semibold leading-5 ${getStatusColor(
                              batch.status
                            )}`}
                          >
                            {batch.status.replace('_', ' ')}
                          </span>
                        </div>
                      </div>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}

        {selectedTab === 'files' && (
          <div className="bg-white shadow sm:rounded-lg">
            <div className="px-4 py-5 sm:p-6">
              {selectedProject ? (
                <>
                  <h3 className="text-lg leading-6 font-medium text-gray-900 mb-4">
                    Files for {selectedProject.name}
                  </h3>
                  <div className="border rounded-lg p-4">
                    {projectFiles && projectFiles.length > 0 ? (
                      renderFileTree(projectFiles)
                    ) : (
                      <p className="text-gray-500 text-center py-8">No files found</p>
                    )}
                  </div>
                </>
              ) : (
                <p className="text-gray-500 text-center py-8">
                  Select a project from the Projects tab to view files
                </p>
              )}
            </div>
          </div>
        )}

        {selectedTab === 'templates' && (
          <div className="bg-white shadow overflow-hidden sm:rounded-md">
            <ul className="divide-y divide-gray-200">
              {templates?.map((template) => (
                <li key={template.id}>
                  <div className="px-4 py-4 sm:px-6">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center">
                        <DocumentIcon className="h-10 w-10 text-gray-400" />
                        <div className="ml-4">
                          <p className="text-sm font-medium text-gray-900">
                            {template.name} v{template.version}
                          </p>
                          <p className="text-sm text-gray-500">
                            {template.category} • {template.file_type.toUpperCase()}
                          </p>
                          {template.description && (
                            <p className="text-sm text-gray-500">{template.description}</p>
                          )}
                        </div>
                      </div>
                      <div className="flex items-center space-x-4">
                        <span className="text-sm text-gray-500">
                          {template.download_count} downloads
                        </span>
                        <button className="inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500">
                          <ArrowDownTrayIcon className="-ml-0.5 mr-2 h-4 w-4" />
                          Download
                        </button>
                      </div>
                    </div>
                  </div>
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>
    </div>
  );
}