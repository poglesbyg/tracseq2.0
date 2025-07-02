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

export const apps: AppDefinition[] = [
  // Laboratory Apps
  {
    id: 'dashboard',
    name: 'Dashboard',
    icon: <ChartBarIcon className="w-6 h-6" />,
    component: Dashboard,
    defaultSize: { width: 1200, height: 800 },
    dockIconClass: 'bg-gradient-to-br from-blue-400 to-blue-600',
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