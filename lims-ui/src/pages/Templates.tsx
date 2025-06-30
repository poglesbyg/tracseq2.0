import { useState, useCallback } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import axios from '../utils/axios';
import { DocumentTextIcon, EyeIcon, TrashIcon, PencilIcon } from '@heroicons/react/24/outline';
import SpreadsheetViewer from '../components/SpreadsheetViewer';
import BatchSampleCreation from '../components/BatchSampleCreation';
import TemplateEditModal from '../components/TemplateEditModal';

  interface Template {
    id: string;
    name: string;
    description?: string;
    created_at: string;
    fields?: TemplateField[];
    metadata: Record<string, unknown>;
  }

interface TemplateField {
  name: string;
  type: string;
  required: boolean;
  defaultValue?: string | number | boolean | null;
}

interface SheetData {
  name: string;
  headers: string[];
  rows: string[][];
  total_rows: number;
  total_columns: number;
}

interface SpreadsheetData {
  sheet_names: string[];
  sheets: SheetData[];
}

  interface ParsedTemplateResponse {
    template: Template;
    data: SpreadsheetData;
  }

export default function Templates() {
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [viewingTemplate, setViewingTemplate] = useState<ParsedTemplateResponse | null>(null);
      const [deletingTemplate, setDeletingTemplate] = useState<Template | null>(null);
    const [editingTemplate, setEditingTemplate] = useState<Template | null>(null);
  const [creatingFromTemplate, setCreatingFromTemplate] = useState<ParsedTemplateResponse | null>(null);
  const queryClient = useQueryClient();

      const { data: templates, isLoading } = useQuery<Template[]>({
    queryKey: ['templates'],
    queryFn: async () => {
      const response = await axios.get('/api/templates');
      console.log('📊 Templates response:', response.data);
      
      // Handle API response structure - API returns {data: [...], templates: [...]}
      const apiData = response.data || {};
      const rawTemplates = apiData.data || apiData.templates || [];
      
      // Ensure we always return an array and transform to match Template interface
      const templatesArray = Array.isArray(rawTemplates) ? rawTemplates : [];
      
      // Transform API data to match Template interface
      const transformedTemplates = templatesArray.map((apiTemplate: Record<string, unknown>) => ({
        id: String(apiTemplate.id || ''),
        name: String(apiTemplate.name || ''),
        description: apiTemplate.description ? String(apiTemplate.description) : undefined,
        created_at: String(apiTemplate.created_at || new Date().toISOString()),
        fields: Array.isArray(apiTemplate.fields) ? apiTemplate.fields as TemplateField[] : [],
        metadata: (apiTemplate.metadata || {}) as Record<string, unknown>
      }));
      
      console.log(`✅ Final templates count: ${transformedTemplates.length}`);
      return transformedTemplates;
    },
  });

  const uploadMutation = useMutation({
    mutationFn: async (file: File) => {
      const formData = new FormData();
      formData.append('file', file);
      
      // Add template metadata - backend expects CreateTemplate format
      const templateData = {
        name: file.name.replace(/\.[^/.]+$/, ""), // Remove file extension
        description: `Uploaded spreadsheet: ${file.name}`,
        file_path: "", // Backend will set this
        file_type: "", // Backend will set this  
        fields: [], // Initialize with empty fields array
        metadata: {
          originalFileName: file.name,
          fileSize: file.size,
          uploadedAt: new Date().toISOString()
        }
      };
      
      formData.append('template', JSON.stringify(templateData));
      
      const response = await axios.post('/api/templates/upload', formData);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['templates'] });
      setSelectedFile(null);
    },
  });

  const viewTemplateMutation = useMutation({
    mutationFn: async (templateId: string) => {
      const response = await axios.get<ParsedTemplateResponse>(`/api/templates/${templateId}/data`);
      return response.data;
    },
    onSuccess: (data) => {
      setViewingTemplate(data);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: async (templateId: string) => {
      await axios.delete(`/api/templates/${templateId}`);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['templates'] });
      setDeletingTemplate(null);
    },
  });

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    if (event.target.files && event.target.files[0]) {
      setSelectedFile(event.target.files[0]);
    }
  };

  const handleUpload = async () => {
    if (selectedFile) {
      uploadMutation.mutate(selectedFile);
    }
  };

  const handleViewTemplate = async (templateId: string) => {
    viewTemplateMutation.mutate(templateId);
  };

      const handleEditTemplate = (template: Template) => {
    setEditingTemplate(template);
  };

      const handleDeleteTemplate = async (template: Template) => {
    setDeletingTemplate(template);
  };

  const confirmDelete = async () => {
    if (deletingTemplate) {
      deleteMutation.mutate(deletingTemplate.id);
    }
  };

  const cancelDelete = () => {
    setDeletingTemplate(null);
  };

      const handleCreateSamples = (template: Template, data: SpreadsheetData) => {
    setCreatingFromTemplate({ template, data });
    setViewingTemplate(null);
  };

  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);

    const files = e.dataTransfer.files;
    if (files && files.length > 0) {
      const file = files[0];
      if (file.name.match(/\.(xlsx|xls|csv)$/i)) {
        setSelectedFile(file);
      }
    }
  }, []);



  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div role="status" className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
      </div>
    );
  }

  return (
    <div className="px-4 sm:px-6 lg:px-8 py-8 bg-white min-h-screen">
      <div className="sm:flex sm:items-center">
        <div className="sm:flex-auto">
          <h1 className="text-2xl font-semibold text-gray-900">Templates</h1>
          <p className="mt-2 text-sm text-gray-700">
            A list of all spreadsheet templates used for sample submission.
          </p>
        </div>
        <div className="mt-4 sm:ml-16 sm:mt-0 sm:flex-none">
          <div
            className={`relative cursor-pointer rounded-md bg-white font-semibold text-indigo-600 focus-within:outline-none focus-within:ring-2 focus-within:ring-indigo-600 focus-within:ring-offset-2 hover:text-indigo-500 ${
              isDragging ? 'ring-2 ring-indigo-600' : ''
            }`}
            onDragEnter={handleDragEnter}
            onDragLeave={handleDragLeave}
            onDragOver={handleDragOver}
            onDrop={handleDrop}
            role="button"
            tabIndex={0}
          >
            <div className="p-4 border-2 border-dashed border-gray-300 rounded-md text-center">
              <DocumentTextIcon className="mx-auto h-12 w-12 text-gray-400" />
              <p className="mt-2 text-sm text-gray-600">
                Drag and drop your Excel file here, or{' '}
                <label
                  htmlFor="file-upload"
                  className="relative cursor-pointer rounded-md font-semibold text-indigo-600 focus-within:outline-none focus-within:ring-2 focus-within:ring-indigo-600 focus-within:ring-offset-2 hover:text-indigo-500"
                >
                  <span>click to select</span>
                  <input
                    id="file-upload"
                    name="file-upload"
                    type="file"
                    className="sr-only"
                    accept=".xlsx,.xls,.csv"
                    onChange={handleFileChange}
                  />
                </label>
              </p>
              <p className="mt-1 text-xs text-gray-500">Supports .xlsx, .xls, and .csv files</p>
            </div>
          </div>
          {selectedFile && (
            <button
              type="button"
              onClick={handleUpload}
              disabled={uploadMutation.isPending}
              className="mt-3 block w-full rounded-md bg-indigo-600 px-3 py-2 text-center text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
            >
              {uploadMutation.isPending ? 'Uploading...' : 'Confirm Upload'}
            </button>
          )}
        </div>
      </div>

      <div className="mt-8 flow-root">
        <div className="-mx-4 -my-2 overflow-x-auto sm:-mx-6 lg:-mx-8">
          <div className="inline-block min-w-full py-2 align-middle sm:px-6 lg:px-8">
            <div className="overflow-hidden shadow ring-1 ring-black ring-opacity-5 sm:rounded-lg">
              <table className="min-w-full divide-y divide-gray-300">
                <thead className="bg-gray-50">
                  <tr>
                    <th scope="col" className="py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-gray-900 sm:pl-6">
                      Name
                    </th>
                    <th scope="col" className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                      Description
                    </th>
                    <th scope="col" className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                      Created
                    </th>
                    <th scope="col" className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                      Actions
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-200 bg-white">
                  {Array.isArray(templates) && templates.map((template) => (
                    <tr key={template.id}>
                      <td className="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium text-gray-900 sm:pl-6">
                        <div className="flex items-center">
                          <DocumentTextIcon className="h-5 w-5 text-gray-400 mr-2" />
                          {template.name}
                        </div>
                      </td>
                      <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                        {template.description || '-'}
                      </td>
                      <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                                                  {new Date(template.created_at).toLocaleDateString()}
                      </td>
                      <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                        <div className="flex space-x-2">
                          <button
                            onClick={() => handleViewTemplate(template.id)}
                            className="inline-flex items-center px-2 py-1 text-xs font-medium text-indigo-600 bg-indigo-100 rounded-md hover:bg-indigo-200 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                          >
                            <EyeIcon className="h-4 w-4 mr-1" />
                            View Data
                          </button>
                          <button
                            onClick={() => handleEditTemplate(template)}
                            className="inline-flex items-center px-2 py-1 text-xs font-medium text-green-600 bg-green-100 rounded-md hover:bg-green-200 focus:outline-none focus:ring-2 focus:ring-green-500"
                          >
                            <PencilIcon className="h-4 w-4 mr-1" />
                            Edit
                          </button>
                          <button
                            onClick={() => handleDeleteTemplate(template)}
                            className="inline-flex items-center px-2 py-1 text-xs font-medium text-red-600 bg-red-100 rounded-md hover:bg-red-200 focus:outline-none focus:ring-2 focus:ring-red-500"
                          >
                            <TrashIcon className="h-4 w-4 mr-1" />
                            Delete
                          </button>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </div>

      {/* Spreadsheet Viewer Modal */}
      {viewingTemplate && (
        <SpreadsheetViewer
          template={viewingTemplate.template}
          data={viewingTemplate.data}
          onClose={() => setViewingTemplate(null)}
          onCreateSamples={handleCreateSamples}
        />
      )}

      {/* Template Edit Modal */}
      {editingTemplate && (
        <TemplateEditModal
          isOpen={!!editingTemplate}
          template={editingTemplate ? { ...editingTemplate, fields: editingTemplate.fields || [] } : null}
          onClose={() => setEditingTemplate(null)}
          onSave={() => {
            queryClient.invalidateQueries({ queryKey: ['templates'] });
            setEditingTemplate(null);
          }}
        />
      )}

      {/* Batch Sample Creation Modal */}
      {creatingFromTemplate && (
        <BatchSampleCreation
          templateData={creatingFromTemplate}
          onClose={() => setCreatingFromTemplate(null)}
          onComplete={() => {
            setCreatingFromTemplate(null);
            // Optionally show success message or redirect to samples page
          }}
        />
      )}

      {/* Delete Confirmation Modal */}
      {deletingTemplate && (
        <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
          <div className="relative top-20 mx-auto p-5 border w-96 shadow-lg rounded-md bg-white">
            <div className="mt-3 text-center">
              <TrashIcon className="mx-auto h-16 w-16 text-red-400" />
              <h3 className="text-lg leading-6 font-medium text-gray-900 mt-4">
                Delete Template
              </h3>
              <div className="mt-2 px-7 py-3">
                <p className="text-sm text-gray-500">
                  Are you sure you want to delete the template "{deletingTemplate.name}"?
                  This action cannot be undone and will permanently remove the template
                  and its associated file.
                </p>
              </div>
              <div className="flex justify-center space-x-4 px-4 py-3">
                <button
                  onClick={cancelDelete}
                  className="px-4 py-2 bg-gray-200 text-gray-800 text-base font-medium rounded-md hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-300"
                >
                  Cancel
                </button>
                <button
                  onClick={confirmDelete}
                  disabled={deleteMutation.isPending}
                  className="px-4 py-2 bg-red-600 text-white text-base font-medium rounded-md hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-300 disabled:opacity-50"
                >
                  {deleteMutation.isPending ? 'Deleting...' : 'Delete'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
} 
