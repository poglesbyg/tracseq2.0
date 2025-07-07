import React, { useState, useEffect } from 'react';
import axios from 'axios';
import { 
  Finder, 
  FileSystemItem, 
  SampleMetadata, 
  TemplateMetadata, 
  ProjectMetadata, 
  ReportMetadata 
} from './Desktop/Finder';
import { WindowContext } from './Desktop/Window';

interface FinderAppWrapperProps {
  windowContext?: WindowContext;
}

// API Response Types
interface SampleApiResponse {
  id: string;
  name?: string;
  barcode: string;
  status: string;
  location: string;
  created_at: string;
  updated_at?: string;
  description?: string;
  metadata?: {
    sample_type?: string;
    concentration_ng_ul?: number;
    project?: string;
    volume_ul?: number;
  };
}

interface TemplateApiResponse {
  id: string;
  name: string;
  version: string;
  is_active: boolean;
  description?: string;
  created_at: string;
  updated_at?: string;
}

interface ProjectApiResponse {
  id: string;
  name: string;
  project_code: string;
  project_type: string;
  status: string;
  priority: string;
  department: string;
  budget_approved?: number;
  budget_used?: number;
  description?: string;
  created_at: string;
  updated_at?: string;
}

interface ReportApiResponse {
  id: string;
  name: string;
  format: string;
  status: string;
  file_path: string;
  file_size?: number;
  description?: string;
  created_at: string;
  completed_at?: string;
}

export const FinderAppWrapper: React.FC<FinderAppWrapperProps> = ({ windowContext }) => {
  const [selectedItem, setSelectedItem] = useState<FileSystemItem | null>(null);
  const [items, setItems] = useState<FileSystemItem[]>([]);
  const [_detailView, _setDetailView] = useState<React.ReactNode>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [samplesRes, templatesRes, projectsRes, reportsRes] = await Promise.all([
          axios.get('/api/samples'),
          axios.get('/api/templates'),
          axios.get('/api/projects'),
          axios.get('/api/reports')
        ]);

        const fileItems: FileSystemItem[] = [
          { id: 'samples-folder', name: 'Samples', type: 'folder', parent: null, created: new Date(), modified: new Date(), children: [] },
          { id: 'templates-folder', name: 'Templates', type: 'folder', parent: null, created: new Date(), modified: new Date(), children: [] },
          { id: 'projects-folder', name: 'Projects', type: 'folder', parent: null, created: new Date(), modified: new Date(), children: [] },
          { id: 'reports-folder', name: 'Reports', type: 'folder', parent: null, created: new Date(), modified: new Date(), children: [] },
        ];

        // Process samples
        const samplesData = samplesRes.data?.data || samplesRes.data || [];
        if (Array.isArray(samplesData)) {
          samplesData.forEach((sample: SampleApiResponse) => {
            const metadata: SampleMetadata = {
              id: sample.id,
              barcode: sample.barcode,
              status: sample.status,
              location: sample.location,
              sampleType: sample.metadata?.sample_type,
              concentration: sample.metadata?.concentration_ng_ul,
              project: sample.metadata?.project,
              description: sample.description
            };
            
            fileItems.push({
              id: `sample-${sample.id}`,
              name: sample.name || sample.barcode,
              type: 'sample',
              parent: 'samples-folder',
              created: new Date(sample.created_at),
              modified: new Date(sample.updated_at || sample.created_at),
              size: sample.metadata?.volume_ul ? Math.round(sample.metadata.volume_ul * 100) : 1024,
              metadata
            });
          });
        }

        // Process templates
        const templatesData = templatesRes.data?.data || templatesRes.data || [];
        if (Array.isArray(templatesData)) {
          templatesData.forEach((template: TemplateApiResponse) => {
            const metadata: TemplateMetadata = {
              id: template.id,
              version: template.version,
              isActive: template.is_active,
              description: template.description
            };
            
            fileItems.push({
              id: `template-${template.id}`,
              name: template.name,
              type: 'template',
              parent: 'templates-folder',
              created: new Date(template.created_at),
              modified: new Date(template.updated_at || template.created_at),
              size: 2048,
              metadata
            });
          });
        }

        // Process projects
        const projectsData = projectsRes.data?.data || projectsRes.data || [];
        if (Array.isArray(projectsData)) {
          projectsData.forEach((project: ProjectApiResponse) => {
            const metadata: ProjectMetadata = {
              id: project.id,
              name: project.name,
              projectCode: project.project_code,
              projectType: project.project_type,
              status: project.status,
              priority: project.priority,
              department: project.department,
              budgetApproved: project.budget_approved,
              budgetUsed: project.budget_used,
              description: project.description
            };
            
            fileItems.push({
              id: `project-${project.id}`,
              name: `${project.project_code} - ${project.name}`,
              type: 'project',
              parent: 'projects-folder',
              created: new Date(project.created_at),
              modified: new Date(project.updated_at || project.created_at),
              size: project.budget_approved ? Math.round(project.budget_approved / 100) : 5120,
              metadata
            });
          });
        }

        // Process reports
        const reportsData = reportsRes.data?.data || reportsRes.data || [];
        if (Array.isArray(reportsData)) {
          reportsData.forEach((report: ReportApiResponse) => {
            const metadata: ReportMetadata = {
              id: report.id,
              format: report.format,
              status: report.status,
              filePath: report.file_path,
              description: report.description
            };
            
            fileItems.push({
              id: `report-${report.id}`,
              name: report.name,
              type: 'report',
              parent: 'reports-folder',
              created: new Date(report.created_at),
              modified: new Date(report.completed_at || report.created_at),
              size: report.file_size || 1024,
              metadata
            });
          });
        }

        setItems(fileItems);
      } catch (error) {
        console.error('Error fetching data:', error);
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
    if (!item.metadata || typeof item.metadata !== 'object') return;
    
    const metadata = item.metadata as SampleMetadata | TemplateMetadata | ProjectMetadata | ReportMetadata;
    const appId = item.type === 'sample' ? 'samples' :
                  item.type === 'template' ? 'templates' :
                  item.type === 'project' ? 'projects' :
                  item.type === 'report' ? 'reports' : null;
    
    if (appId && windowContext?.openApp) {
      windowContext.openApp(appId, { 
        selectedItemId: metadata.id,
        selectedItem: metadata 
      });
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
      {_detailView && (
        <div className="absolute inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          {_detailView}
        </div>
      )}
    </div>
  );
}; 