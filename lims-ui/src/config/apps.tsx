import { AppDefinition } from '../types/apps';
import {
  BeakerIcon,
  DocumentIcon,
  QueueListIcon,
  TableCellsIcon,
  MapPinIcon,
  ChartBarIcon,
  SparklesIcon,
  CpuChipIcon,
  FolderIcon,
  ClipboardDocumentCheckIcon,
  ServerStackIcon,
  UsersIcon
} from '@heroicons/react/24/outline';
import { Finder, FileSystemItem } from '../components/Desktop/Finder';
import { WindowContext } from '../components/Desktop/Window';
import { useState, useEffect } from 'react';
import axios from 'axios';

// Import all pages
import Dashboard from '../pages/Dashboard';
import Templates from '../pages/Templates';
import Samples from '../pages/Samples';
import RagSubmissions from '../pages/RagSubmissions';
import RagSamples from '../pages/RagSamples';
import Sequencing from '../pages/Sequencing';
import Spreadsheets from '../pages/Spreadsheets';
import Storage from '../pages/Storage';
import Reports from '../pages/Reports';
import Profile from '../pages/Profile';
import Users from '../pages/Users';
import LibraryPrep from '../pages/LibraryPrep';
import QualityControl from '../pages/QualityControl';
import ProjectManagement from '../pages/ProjectManagement';
import FlowCellDesign from '../pages/FlowCellDesign';
import ServicesStatus from '../pages/ServicesStatus';

// Create a wrapper component for Finder
const FinderApp = ({ windowContext }: { windowContext?: WindowContext }) => {
  const [selectedItem, setSelectedItem] = useState<FileSystemItem | null>(null);
  const [items, setItems] = useState<FileSystemItem[]>([]);
  const [detailView, setDetailView] = useState<React.ReactNode>(null);

  useEffect(() => {
    // Fetch real data from the API
    const fetchData = async () => {
      try {
        const [samplesRes, templatesRes, projectsRes, reportsRes] = await Promise.all([
          axios.get('/api/samples'),
          axios.get('/api/templates'),
          axios.get('/api/projects'),
          axios.get('/api/reports')
        ]);

        const fileItems: FileSystemItem[] = [
          // Root folders
          { id: 'samples-folder', name: 'Samples', type: 'folder', parent: null, created: new Date(), modified: new Date(), children: [] },
          { id: 'templates-folder', name: 'Templates', type: 'folder', parent: null, created: new Date(), modified: new Date(), children: [] },
          { id: 'projects-folder', name: 'Projects', type: 'folder', parent: null, created: new Date(), modified: new Date(), children: [] },
          { id: 'reports-folder', name: 'Reports', type: 'folder', parent: null, created: new Date(), modified: new Date(), children: [] },
        ];

        // Add samples to the samples folder
        const samplesData = samplesRes.data?.data || samplesRes.data || [];
        if (Array.isArray(samplesData)) {
          samplesData.forEach((sample: {
            id: string;
            name?: string;
            barcode: string;
            created_at: string;
            updated_at?: string;
            status: string;
            location: string;
            metadata?: {
              volume_ul?: number;
              sample_type?: string;
              concentration_ng_ul?: number;
              project?: string;
            };
          }) => {
            fileItems.push({
              id: `sample-${sample.id}`,
              name: sample.name || sample.barcode,
              type: 'sample',
              parent: 'samples-folder',
              created: new Date(sample.created_at),
              modified: new Date(sample.updated_at || sample.created_at),
              size: sample.metadata?.volume_ul ? Math.round(sample.metadata.volume_ul * 100) : 1024,
              metadata: {
                ...sample,
                barcode: sample.barcode,
                status: sample.status,
                location: sample.location,
                sampleType: sample.metadata?.sample_type,
                concentration: sample.metadata?.concentration_ng_ul,
                project: sample.metadata?.project
              }
            });
          });
        }

        // Add templates to the templates folder
        const templatesData = templatesRes.data?.data || templatesRes.data || [];
        if (Array.isArray(templatesData)) {
          templatesData.forEach((template: any) => {
            fileItems.push({
              id: `template-${template.id}`,
              name: template.name,
              type: 'template',
              parent: 'templates-folder',
              created: new Date(template.created_at),
              modified: new Date(template.updated_at || template.created_at),
              size: 2048,
              metadata: {
                ...template,
                version: template.version,
                isActive: template.is_active
              }
            });
          });
        }

        // Add projects to the projects folder
        const projectsData = projectsRes.data?.data || projectsRes.data || [];
        if (Array.isArray(projectsData)) {
          projectsData.forEach((project: any) => {
            fileItems.push({
              id: `project-${project.id}`,
              name: `${project.project_code} - ${project.name}`,
              type: 'project',
              parent: 'projects-folder',
              created: new Date(project.created_at),
              modified: new Date(project.updated_at || project.created_at),
              size: project.budget_approved ? Math.round(project.budget_approved / 100) : 5120,
              metadata: {
                ...project,
                projectCode: project.project_code,
                projectType: project.project_type,
                status: project.status,
                priority: project.priority,
                department: project.department,
                budgetApproved: project.budget_approved,
                budgetUsed: project.budget_used
              }
            });
          });
        }

        // Add reports to the reports folder
        const reportsData = reportsRes.data?.data || reportsRes.data || [];
        if (Array.isArray(reportsData)) {
          reportsData.forEach((report: any) => {
            fileItems.push({
              id: `report-${report.id}`,
              name: report.name,
              type: 'report',
              parent: 'reports-folder',
              created: new Date(report.created_at),
              modified: new Date(report.completed_at || report.created_at),
              size: report.file_size || 1024,
              metadata: {
                ...report,
                format: report.format,
                status: report.status,
                filePath: report.file_path,
                description: report.description
              }
            });
          });
        }

        setItems(fileItems);
      } catch (error) {
        console.error('Error fetching data:', error);
        // Fallback to mock data if API fails
        setItems([
          { id: 'samples-folder', name: 'Samples', type: 'folder', parent: null, created: new Date(), modified: new Date() },
          { id: 'templates-folder', name: 'Templates', type: 'folder', parent: null, created: new Date(), modified: new Date() },
          { id: 'projects-folder', name: 'Projects', type: 'folder', parent: null, created: new Date(), modified: new Date() },
          { id: 'reports-folder', name: 'Reports', type: 'folder', parent: null, created: new Date(), modified: new Date() },
        ]);
      }
    };

    fetchData();
  }, []);

  const handleItemOpen = (item: FileSystemItem) => {
    if (item.type === 'sample') {
      // Try to open in the Samples app
      if (windowContext?.openApp) {
        windowContext.openApp('samples', { 
          selectedSampleId: item.metadata?.id,
          selectedSample: item.metadata 
        });
      } else {
        // Fallback to showing details in a modal
        setDetailView(
          <div className="p-6 bg-white rounded-lg shadow-lg max-w-2xl mx-auto">
            <div className="flex justify-between items-start mb-4">
              <h2 className="text-2xl font-bold">{item.name}</h2>
              <button
                onClick={() => setDetailView(null)}
                className="text-gray-400 hover:text-gray-600"
              >
                <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="text-sm font-medium text-gray-500">Barcode</label>
                <p className="text-lg font-mono">{String(item.metadata?.barcode || '')}</p>
              </div>
              <div>
                <label className="text-sm font-medium text-gray-500">Status</label>
                <p className="text-lg">
                  <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                    item.metadata?.status === 'in_storage' ? 'bg-green-100 text-green-800' :
                    item.metadata?.status === 'in_sequencing' ? 'bg-blue-100 text-blue-800' :
                    item.metadata?.status === 'pending' ? 'bg-yellow-100 text-yellow-800' :
                    'bg-gray-100 text-gray-800'
                  }`}>
                    {String(item.metadata?.status || '').replace('_', ' ').toUpperCase()}
                  </span>
                </p>
              </div>
              <div className="col-span-2">
                <label className="text-sm font-medium text-gray-500">Location</label>
                <p className="text-lg">{item.metadata?.location}</p>
              </div>
              <div>
                <label className="text-sm font-medium text-gray-500">Type</label>
                <p className="text-lg capitalize">{item.metadata?.sampleType || 'Unknown'}</p>
              </div>
              {item.metadata?.concentration && (
                <div>
                  <label className="text-sm font-medium text-gray-500">Concentration</label>
                  <p className="text-lg">{item.metadata.concentration} ng/μl</p>
                </div>
              )}
              {item.metadata?.project && (
                <div className="col-span-2">
                  <label className="text-sm font-medium text-gray-500">Project</label>
                  <p className="text-lg">{item.metadata.project}</p>
                </div>
              )}
              {item.metadata?.description && (
                <div className="col-span-2">
                  <label className="text-sm font-medium text-gray-500">Description</label>
                  <p className="text-base text-gray-700">{item.metadata.description}</p>
                </div>
              )}
            </div>
            
            <div className="mt-6 flex gap-3">
              {windowContext?.openApp && (
                <button
                  onClick={() => {
                    windowContext.openApp('samples', { 
                      selectedSampleId: item.metadata?.id,
                      selectedSample: item.metadata 
                    });
                    setDetailView(null);
                  }}
                  className="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600"
                >
                  Open in Samples App
                </button>
              )}
              <button
                onClick={() => setDetailView(null)}
                className="px-4 py-2 bg-gray-200 text-gray-800 rounded hover:bg-gray-300"
              >
                Close
              </button>
            </div>
          </div>
        );
      }
    } else if (item.type === 'template') {
      // Try to open in the Templates app
      if (windowContext?.openApp) {
        windowContext.openApp('templates', { 
          selectedTemplateId: item.metadata?.id,
          selectedTemplate: item.metadata 
        });
      } else {
        // Fallback to showing details in a modal
        setDetailView(
          <div className="p-6 bg-white rounded-lg shadow-lg max-w-2xl mx-auto">
            <div className="flex justify-between items-start mb-4">
              <h2 className="text-2xl font-bold">{item.name}</h2>
              <button
                onClick={() => setDetailView(null)}
                className="text-gray-400 hover:text-gray-600"
              >
                <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="text-sm font-medium text-gray-500">Version</label>
                <p className="text-lg">{item.metadata?.version}</p>
              </div>
              <div>
                <label className="text-sm font-medium text-gray-500">Status</label>
                <p className="text-lg">
                  <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                    item.metadata?.isActive ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'
                  }`}>
                    {item.metadata?.isActive ? 'ACTIVE' : 'INACTIVE'}
                  </span>
                </p>
              </div>
              <div className="col-span-2">
                <label className="text-sm font-medium text-gray-500">Description</label>
                <p className="text-base text-gray-700">{item.metadata?.description || 'No description available'}</p>
              </div>
              <div>
                <label className="text-sm font-medium text-gray-500">Created</label>
                <p className="text-sm">{item.created.toLocaleDateString()}</p>
              </div>
              <div>
                <label className="text-sm font-medium text-gray-500">Modified</label>
                <p className="text-sm">{item.modified.toLocaleDateString()}</p>
              </div>
            </div>
            
            <div className="mt-6 flex gap-3">
              {windowContext?.openApp && (
                <button
                  onClick={() => {
                    windowContext.openApp('templates', { 
                      selectedTemplateId: item.metadata?.id,
                      selectedTemplate: item.metadata 
                    });
                    setDetailView(null);
                  }}
                  className="px-4 py-2 bg-purple-500 text-white rounded hover:bg-purple-600"
                >
                  Open in Templates App
                </button>
              )}
              <button
                onClick={() => setDetailView(null)}
                className="px-4 py-2 bg-gray-200 text-gray-800 rounded hover:bg-gray-300"
              >
                Close
              </button>
            </div>
          </div>
        );
      }
    } else if (item.type === 'project') {
      // Try to open in the Projects app
      if (windowContext?.openApp) {
        windowContext.openApp('projects', { 
          selectedProjectId: item.metadata?.id,
          selectedProject: item.metadata 
        });
      } else {
        // Fallback to showing details in a modal
        setDetailView(
          <div className="p-6 bg-white rounded-lg shadow-lg max-w-2xl mx-auto">
            <div className="flex justify-between items-start mb-4">
              <h2 className="text-2xl font-bold">{item.metadata?.name}</h2>
              <button
                onClick={() => setDetailView(null)}
                className="text-gray-400 hover:text-gray-600"
              >
                <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="text-sm font-medium text-gray-500">Project Code</label>
                <p className="text-lg font-mono">{item.metadata?.projectCode}</p>
              </div>
              <div>
                <label className="text-sm font-medium text-gray-500">Status</label>
                <p className="text-lg">
                  <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                    item.metadata?.status === 'active' ? 'bg-green-100 text-green-800' :
                    item.metadata?.status === 'completed' ? 'bg-blue-100 text-blue-800' :
                    item.metadata?.status === 'planning' ? 'bg-yellow-100 text-yellow-800' :
                    'bg-gray-100 text-gray-800'
                  }`}>
                    {item.metadata?.status?.toUpperCase()}
                  </span>
                </p>
              </div>
              <div>
                <label className="text-sm font-medium text-gray-500">Priority</label>
                <p className="text-lg">
                  <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                    item.metadata?.priority === 'urgent' ? 'bg-red-100 text-red-800' :
                    item.metadata?.priority === 'high' ? 'bg-orange-100 text-orange-800' :
                    item.metadata?.priority === 'medium' ? 'bg-yellow-100 text-yellow-800' :
                    'bg-gray-100 text-gray-800'
                  }`}>
                    {item.metadata?.priority?.toUpperCase()}
                  </span>
                </p>
              </div>
              <div>
                <label className="text-sm font-medium text-gray-500">Type</label>
                <p className="text-lg capitalize">{item.metadata?.projectType?.replace('_', ' ')}</p>
              </div>
              <div>
                <label className="text-sm font-medium text-gray-500">Department</label>
                <p className="text-lg">{item.metadata?.department}</p>
              </div>
              <div>
                <label className="text-sm font-medium text-gray-500">Budget</label>
                <p className="text-lg">
                  ${item.metadata?.budgetUsed?.toLocaleString()} / ${item.metadata?.budgetApproved?.toLocaleString()}
                </p>
              </div>
              {item.metadata?.description && (
                <div className="col-span-2">
                  <label className="text-sm font-medium text-gray-500">Description</label>
                  <p className="text-base text-gray-700">{item.metadata.description}</p>
                </div>
              )}
            </div>
            
            <div className="mt-6 flex gap-3">
              {windowContext?.openApp && (
                <button
                  onClick={() => {
                    windowContext.openApp('projects', { 
                      selectedProjectId: item.metadata?.id,
                      selectedProject: item.metadata 
                    });
                    setDetailView(null);
                  }}
                  className="px-4 py-2 bg-amber-500 text-white rounded hover:bg-amber-600"
                >
                  Open in Projects App
                </button>
              )}
              <button
                onClick={() => setDetailView(null)}
                className="px-4 py-2 bg-gray-200 text-gray-800 rounded hover:bg-gray-300"
              >
                Close
              </button>
            </div>
          </div>
        );
      }
    } else if (item.type === 'report') {
      // Try to open in the Reports app
      if (windowContext?.openApp) {
        windowContext.openApp('reports', { 
          selectedReportId: item.metadata?.id,
          selectedReport: item.metadata 
        });
      } else {
        // Fallback to showing details in a modal
        setDetailView(
          <div className="p-6 bg-white rounded-lg shadow-lg max-w-2xl mx-auto">
            <div className="flex justify-between items-start mb-4">
              <h2 className="text-2xl font-bold">{item.name}</h2>
              <button
                onClick={() => setDetailView(null)}
                className="text-gray-400 hover:text-gray-600"
              >
                <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="text-sm font-medium text-gray-500">Format</label>
                <p className="text-lg uppercase">{item.metadata?.format}</p>
              </div>
              <div>
                <label className="text-sm font-medium text-gray-500">Status</label>
                <p className="text-lg">
                  <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                    item.metadata?.status === 'completed' ? 'bg-green-100 text-green-800' :
                    item.metadata?.status === 'processing' ? 'bg-blue-100 text-blue-800' :
                    item.metadata?.status === 'pending' ? 'bg-yellow-100 text-yellow-800' :
                    'bg-gray-100 text-gray-800'
                  }`}>
                    {item.metadata?.status?.toUpperCase()}
                  </span>
                </p>
              </div>
              <div>
                <label className="text-sm font-medium text-gray-500">Size</label>
                <p className="text-lg">{((item.size || 0) / 1024).toFixed(1)} KB</p>
              </div>
              <div>
                <label className="text-sm font-medium text-gray-500">Created</label>
                <p className="text-sm">{item.created.toLocaleDateString()}</p>
              </div>
              {item.metadata?.description && (
                <div className="col-span-2">
                  <label className="text-sm font-medium text-gray-500">Description</label>
                  <p className="text-base text-gray-700">{item.metadata.description}</p>
                </div>
              )}
              {item.metadata?.filePath && (
                <div className="col-span-2">
                  <label className="text-sm font-medium text-gray-500">File Path</label>
                  <p className="text-sm font-mono text-gray-600">{item.metadata.filePath}</p>
                </div>
              )}
            </div>
            
            <div className="mt-6 flex gap-3">
              {windowContext?.openApp && (
                <button
                  onClick={() => {
                    windowContext.openApp('reports', { 
                      selectedReportId: item.metadata?.id,
                      selectedReport: item.metadata 
                    });
                    setDetailView(null);
                  }}
                  className="px-4 py-2 bg-orange-500 text-white rounded hover:bg-orange-600"
                >
                  Open in Reports App
                </button>
              )}
              {item.metadata?.status === 'completed' && item.metadata?.filePath && (
                <button
                  onClick={() => {
                    // Create a download link
                    const link = document.createElement('a');
                    link.href = `/api/reports/download/${item.metadata?.id}`;
                    link.download = item.name + '.' + (item.metadata?.format || 'pdf');
                    document.body.appendChild(link);
                    link.click();
                    document.body.removeChild(link);
                  }}
                  className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
                >
                  Download Report
                </button>
              )}
              <button
                onClick={() => setDetailView(null)}
                className="px-4 py-2 bg-gray-200 text-gray-800 rounded hover:bg-gray-300"
              >
                Close
              </button>
            </div>
          </div>
        );
      }
    }
  };
  
  return (
    <div className="relative h-full">
      <Finder
        items={items}
        onItemOpen={handleItemOpen}
        onItemSelect={setSelectedItem}
        selectedItemId={selectedItem?.id}
      />
      {detailView && (
        <div className="absolute inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          {detailView}
        </div>
      )}
    </div>
  );
};

export const apps: AppDefinition[] = [
  // System Apps
  {
    id: 'finder',
    name: 'Finder',
    icon: <FolderIcon className="w-6 h-6" />,
    component: FinderApp,
    defaultSize: { width: 900, height: 600 },
    dockIconClass: 'bg-gradient-to-br from-blue-400 to-blue-600',
    category: 'system',
    description: 'Browse laboratory files and samples'
  },
  
  // Laboratory Apps
  {
    id: 'dashboard',
    name: 'Dashboard',
    icon: <ChartBarIcon className="w-6 h-6" />,
    component: Dashboard,
    defaultSize: { width: 1200, height: 800 },
    dockIconClass: 'bg-gradient-to-br from-indigo-400 to-indigo-600',
    category: 'laboratory',
    description: 'Laboratory overview and analytics'
  },
  {
    id: 'samples',
    name: 'Samples',
    icon: <BeakerIcon className="w-6 h-6" />,
    component: Samples,
    defaultSize: { width: 1000, height: 700 },
    dockIconClass: 'bg-gradient-to-br from-green-400 to-green-600',
    category: 'laboratory',
    description: 'Sample management and tracking'
  },
  {
    id: 'templates',
    name: 'Templates',
    icon: <DocumentIcon className="w-6 h-6" />,
    component: Templates,
    defaultSize: { width: 900, height: 700 },
    dockIconClass: 'bg-gradient-to-br from-purple-400 to-purple-600',
    category: 'laboratory',
    description: 'Laboratory templates and workflows'
  },
  {
    id: 'library-prep',
    name: 'Library Prep',
    icon: <BeakerIcon className="w-6 h-6" />,
    component: LibraryPrep,
    defaultSize: { width: 1000, height: 700 },
    dockIconClass: 'bg-gradient-to-br from-indigo-400 to-indigo-600',
    category: 'laboratory',
    description: 'Library preparation workflows'
  },
  {
    id: 'quality-control',
    name: 'Quality Control',
    icon: <ClipboardDocumentCheckIcon className="w-6 h-6" />,
    component: QualityControl,
    defaultSize: { width: 1000, height: 700 },
    dockIconClass: 'bg-gradient-to-br from-yellow-400 to-yellow-600',
    category: 'laboratory',
    description: 'QC workflows and validation'
  },
  {
    id: 'flow-cell-design',
    name: 'Flow Cell Design',
    icon: <CpuChipIcon className="w-6 h-6" />,
    component: FlowCellDesign,
    defaultSize: { width: 1100, height: 750 },
    dockIconClass: 'bg-gradient-to-br from-pink-400 to-pink-600',
    category: 'laboratory',
    description: 'Design and optimize flow cells'
  },
  {
    id: 'sequencing',
    name: 'Sequencing',
    icon: <QueueListIcon className="w-6 h-6" />,
    component: Sequencing,
    defaultSize: { width: 1000, height: 700 },
    dockIconClass: 'bg-gradient-to-br from-cyan-400 to-cyan-600',
    category: 'laboratory',
    description: 'Sequencing runs and monitoring'
  },

  // AI Apps
  {
    id: 'rag-submissions',
    name: 'AI Submissions',
    icon: <SparklesIcon className="w-6 h-6" />,
    component: RagSubmissions,
    defaultSize: { width: 1100, height: 750 },
    dockIconClass: 'bg-gradient-to-br from-violet-400 to-violet-600',
    category: 'analysis',
    description: 'AI-powered submission processing'
  },
  {
    id: 'rag-samples',
    name: 'AI Samples',
    icon: <CpuChipIcon className="w-6 h-6" />,
    component: RagSamples,
    defaultSize: { width: 1100, height: 750 },
    dockIconClass: 'bg-gradient-to-br from-fuchsia-400 to-fuchsia-600',
    category: 'analysis',
    description: 'AI sample analysis and insights'
  },

  // Data Management
  {
    id: 'spreadsheets',
    name: 'Spreadsheets',
    icon: <TableCellsIcon className="w-6 h-6" />,
    component: Spreadsheets,
    defaultSize: { width: 1200, height: 800 },
    dockIconClass: 'bg-gradient-to-br from-emerald-400 to-emerald-600',
    category: 'data',
    description: 'Spreadsheet data management'
  },
  {
    id: 'storage',
    name: 'Storage',
    icon: <MapPinIcon className="w-6 h-6" />,
    component: Storage,
    defaultSize: { width: 900, height: 600 },
    dockIconClass: 'bg-gradient-to-br from-teal-400 to-teal-600',
    category: 'data',
    description: 'Storage location management'
  },
  {
    id: 'reports',
    name: 'Reports',
    icon: <ChartBarIcon className="w-6 h-6" />,
    component: Reports,
    defaultSize: { width: 1000, height: 700 },
    dockIconClass: 'bg-gradient-to-br from-orange-400 to-orange-600',
    category: 'analysis',
    description: 'Generate and view reports'
  },

  // Project Management
  {
    id: 'projects',
    name: 'Projects',
    icon: <FolderIcon className="w-6 h-6" />,
    component: ProjectManagement,
    defaultSize: { width: 1000, height: 700 },
    dockIconClass: 'bg-gradient-to-br from-amber-400 to-amber-600',
    category: 'data',
    description: 'Project management and tracking'
  },

  // System Apps
  {
    id: 'services',
    name: 'Services',
    icon: <ServerStackIcon className="w-6 h-6" />,
    component: ServicesStatus,
    defaultSize: { width: 900, height: 600 },
    dockIconClass: 'bg-gradient-to-br from-gray-400 to-gray-600',
    category: 'system',
    description: 'System services monitoring'
  },
  {
    id: 'users',
    name: 'Users',
    icon: <UsersIcon className="w-6 h-6" />,
    component: Users,
    defaultSize: { width: 1000, height: 700 },
    dockIconClass: 'bg-gradient-to-br from-red-400 to-red-600',
    category: 'admin',
    description: 'User management'
  },
  {
    id: 'profile',
    name: 'Profile',
    icon: <UsersIcon className="w-6 h-6" />,
    component: Profile,
    defaultSize: { width: 800, height: 600 },
    dockIconClass: 'bg-gradient-to-br from-slate-400 to-slate-600',
    category: 'system',
    description: 'User profile settings'
  }
];