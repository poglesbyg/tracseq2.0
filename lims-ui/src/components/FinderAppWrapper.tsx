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
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setIsLoading(true);
        setError(null);
        
        // Use the new comprehensive finder endpoint
        const finderRes = await axios.get('/api/finder/all-data?limit=1000');
        const ragSubmissionsRes = await axios.get('/api/rag/submissions');

        const fileItems: FileSystemItem[] = [
          { id: 'samples-folder', name: 'Samples', type: 'folder', parent: null, created: new Date(), modified: new Date(), children: [] },
          { id: 'templates-folder', name: 'Templates', type: 'folder', parent: null, created: new Date(), modified: new Date(), children: [] },
          { id: 'projects-folder', name: 'Projects', type: 'folder', parent: null, created: new Date(), modified: new Date(), children: [] },
          { id: 'reports-folder', name: 'Reports', type: 'folder', parent: null, created: new Date(), modified: new Date(), children: [] },
          { id: 'documents-folder', name: 'Documents', type: 'folder', parent: null, created: new Date(), modified: new Date(), children: [] },
          { id: 'storage-folder', name: 'Storage', type: 'folder', parent: null, created: new Date(), modified: new Date(), children: [] },
          { id: 'sequencing-folder', name: 'Sequencing', type: 'folder', parent: null, created: new Date(), modified: new Date(), children: [] },
          { id: 'qc-folder', name: 'Quality Control', type: 'folder', parent: null, created: new Date(), modified: new Date(), children: [] },
          { id: 'library-folder', name: 'Library Prep', type: 'folder', parent: null, created: new Date(), modified: new Date(), children: [] },
        ];

        // Process unified finder data
        const finderData = finderRes.data?.data || [];
        if (Array.isArray(finderData)) {
          finderData.forEach((item: any) => {
            let parent = 'samples-folder'; // default
            let metadata: any = {};
            let size = 1024;

            // Determine parent folder and metadata based on item type
            switch (item.category) {
              case 'samples':
                parent = 'samples-folder';
                metadata = {
                  id: item.id,
                  barcode: item.barcode,
                  status: item.status,
                  location: item.location || 'Unknown',
                  sampleType: item.sample_type,
                  concentration: item.concentration,
                  project: item.patient_id,
                  description: item.notes || item.description
                };
                size = item.volume ? Math.round(parseFloat(item.volume) * 100) : 1024;
                break;

              case 'templates':
                parent = 'templates-folder';
                metadata = {
                  id: item.id,
                  version: item.version,
                  isActive: item.status === 'Published',
                  description: item.description
                };
                size = 2048;
                break;

              case 'projects':
                parent = 'projects-folder';
                metadata = {
                  id: item.id,
                  name: item.name,
                  projectCode: item.project_code,
                  projectType: item.project_type,
                  status: item.status,
                  priority: item.priority,
                  department: item.department,
                  budgetApproved: item.budget_approved,
                  budgetUsed: item.budget_used,
                  description: item.description
                };
                size = item.budget_approved ? Math.round(item.budget_approved / 100) : 5120;
                break;

              case 'reports':
                parent = 'reports-folder';
                metadata = {
                  id: item.id,
                  format: item.format,
                  status: item.status,
                  filePath: item.file_path,
                  description: item.description
                };
                size = item.file_size || 1024;
                break;

              case 'storage':
                parent = 'storage-folder';
                metadata = {
                  id: item.id,
                  zoneType: item.zone_type,
                  temperature: item.temperature,
                  capacity: item.capacity,
                  currentUsage: item.current_usage,
                  utilization: item.utilization,
                  locationCode: item.location_code,
                  description: item.description
                };
                size = item.capacity || 1024;
                break;

              case 'sequencing':
                parent = 'sequencing-folder';
                metadata = {
                  id: item.id,
                  jobId: item.job_id,
                  platform: item.platform,
                  sampleCount: item.sample_count,
                  status: item.status,
                  priority: item.priority,
                  estimatedCost: item.estimated_cost,
                  description: item.description
                };
                size = (item.sample_count || 1) * 100;
                break;

              case 'qc':
                parent = 'qc-folder';
                metadata = {
                  id: item.id,
                  sampleId: item.sample_id,
                  score: item.score,
                  decision: item.decision,
                  assessedBy: item.assessed_by,
                  comments: item.comments
                };
                size = 512;
                break;

              case 'library':
                parent = 'library-folder';
                metadata = {
                  id: item.id,
                  libraryId: item.library_id,
                  libraryType: item.library_type,
                  concentration: item.concentration,
                  volume: item.volume,
                  fragmentSize: item.fragment_size,
                  preparedBy: item.prepared_by
                };
                size = 256;
                break;
            }
            
            fileItems.push({
              id: item.id,
              name: item.name,
              type: item.type as any,
              parent: parent,
              created: new Date(item.created_at || item.submission_date || item.assessment_date || Date.now()),
              modified: new Date(item.updated_at || item.created_at || item.submission_date || item.assessment_date || Date.now()),
              size: size,
              metadata
            });
          });
        }

        // Process RAG submissions (uploaded documents)
        const ragSubmissionsData = ragSubmissionsRes.data?.data || ragSubmissionsRes.data || [];
        if (Array.isArray(ragSubmissionsData)) {
          ragSubmissionsData.forEach((submission: any) => {
            const metadata = {
              id: submission.id,
              filename: submission.filename,
              status: submission.status,
              submittedBy: submission.submittedBy,
              submitterEmail: submission.submitterEmail,
              confidenceScore: submission.confidenceScore,
              filePath: submission.file_path,
              extractedData: submission.extracted_data,
              description: `Uploaded by ${submission.submittedBy} - ${submission.status}`
            };
            
            fileItems.push({
              id: `document-${submission.id}`,
              name: submission.filename,
              type: 'document',
              parent: 'documents-folder',
              created: new Date(submission.submittedDate),
              modified: new Date(submission.submittedDate),
              size: submission.file_size || 1024,
              metadata
            });
          });
        }

        setItems(fileItems);
        setIsLoading(false);
      } catch (error) {
        console.error('Error fetching data:', error);
        setError('Failed to load data from server. Using offline mode.');
        setItems([
          { id: 'samples-folder', name: 'Samples', type: 'folder', parent: null, created: new Date(), modified: new Date() },
          { id: 'templates-folder', name: 'Templates', type: 'folder', parent: null, created: new Date(), modified: new Date() },
          { id: 'projects-folder', name: 'Projects', type: 'folder', parent: null, created: new Date(), modified: new Date() },
          { id: 'reports-folder', name: 'Reports', type: 'folder', parent: null, created: new Date(), modified: new Date() },
          { id: 'documents-folder', name: 'Documents', type: 'folder', parent: null, created: new Date(), modified: new Date() },
        ]);
        setIsLoading(false);
      }
    };

    fetchData();
  }, []);

  const handleItemOpen = async (item: FileSystemItem) => {
    if (!item.metadata || typeof item.metadata !== 'object') return;
    
    const metadata = item.metadata as SampleMetadata | TemplateMetadata | ProjectMetadata | ReportMetadata;
    
    // Handle different item types
    if (item.type === 'report') {
      // For reports, try to open the file directly
      const reportMetadata = metadata as ReportMetadata;
      if (reportMetadata.filePath) {
        try {
          // Call the API to open the file
          await axios.post(`/api/files/${reportMetadata.id}/open`);
          
          // Show a notification or open in a new window
          if (reportMetadata.format === 'PDF') {
            window.open(`/api/files/${reportMetadata.id}/download`, '_blank');
          } else {
            // For other formats, open in the appropriate app
            const appId = 'reports';
            if (windowContext?.openApp) {
              windowContext.openApp(appId, { 
                selectedItemId: reportMetadata.id,
                selectedItem: reportMetadata 
              });
            }
          }
        } catch (error) {
          console.error('Failed to open file:', error);
          // Fallback to opening in the reports app
          const appId = 'reports';
          if (windowContext?.openApp) {
            windowContext.openApp(appId, { 
              selectedItemId: reportMetadata.id,
              selectedItem: reportMetadata 
            });
          }
        }
      }
    } else if (item.type === 'document') {
      // For documents, try to open the file directly or show in RAG submissions
      const docMetadata = metadata as any;
      try {
        // Try to open the file directly
        if (docMetadata.filePath) {
          // For PDFs, open in new tab
          if (docMetadata.filename?.toLowerCase().endsWith('.pdf')) {
            window.open(`/api/files/${docMetadata.id}/download`, '_blank');
          } else {
            // For other formats, open in RAG submissions view
            if (windowContext?.openApp) {
              windowContext.openApp('rag-submissions', { 
                selectedItemId: docMetadata.id,
                selectedItem: docMetadata 
              });
            }
          }
        }
      } catch (error) {
        console.error('Failed to open document:', error);
        // Fallback to RAG submissions view
        if (windowContext?.openApp) {
          windowContext.openApp('rag-submissions', { 
            selectedItemId: docMetadata.id,
            selectedItem: docMetadata 
          });
        }
      }
    } else {
      // For other item types, open in the appropriate app
      const appId = item.type === 'sample' ? 'samples' :
                    item.type === 'template' ? 'templates' :
                    item.type === 'project' ? 'projects' : null;
      
      if (appId && windowContext?.openApp) {
        windowContext.openApp(appId, { 
          selectedItemId: metadata.id,
          selectedItem: metadata 
        });
      }
    }
  };

  return (
    <div className="relative h-full">
      {isLoading ? (
        <div className="flex items-center justify-center h-full">
          <div className="text-center">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4"></div>
            <p className="text-gray-600">Loading laboratory data...</p>
          </div>
        </div>
      ) : (
        <>
          {error && (
            <div className="bg-yellow-50 border border-yellow-200 rounded-md p-3 mb-4 mx-4 mt-4">
              <div className="flex">
                <div className="flex-shrink-0">
                  <svg className="h-5 w-5 text-yellow-400" viewBox="0 0 20 20" fill="currentColor">
                    <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                  </svg>
                </div>
                <div className="ml-3">
                  <p className="text-sm text-yellow-800">{error}</p>
                </div>
              </div>
            </div>
          )}
          <Finder
            items={items}
            onItemOpen={handleItemOpen}
            onItemSelect={setSelectedItem}
            selectedItemId={selectedItem?.id}
          />
        </>
      )}
      {_detailView && (
        <div className="absolute inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          {_detailView}
        </div>
      )}
    </div>
  );
}; 