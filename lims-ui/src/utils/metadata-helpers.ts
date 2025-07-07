interface SampleMetadata {
  id: string;
  barcode: string;
  status: string;
  location: string;
  sampleType?: string;
  concentration?: number;
  project?: string;
  description?: string;
}

interface TemplateMetadata {
  id: string;
  version: string;
  isActive: boolean;
  description?: string;
}

interface ProjectMetadata {
  id: string;
  name: string;
  projectCode: string;
  projectType: string;
  status: string;
  priority: string;
  department: string;
  budgetApproved?: number;
  budgetUsed?: number;
  description?: string;
}

interface ReportMetadata {
  id: string;
  format: string;
  status: string;
  filePath: string;
  description?: string;
}

export type ItemMetadata = SampleMetadata | TemplateMetadata | ProjectMetadata | ReportMetadata | Record<string, unknown>;

// Type guards
export function isSampleMetadata(metadata: ItemMetadata | undefined): metadata is SampleMetadata {
  return !!metadata && 'barcode' in metadata && typeof metadata.barcode === 'string';
}

export function isTemplateMetadata(metadata: ItemMetadata | undefined): metadata is TemplateMetadata {
  return !!metadata && 'version' in metadata && typeof metadata.version === 'string';
}

export function isProjectMetadata(metadata: ItemMetadata | undefined): metadata is ProjectMetadata {
  return !!metadata && 'projectCode' in metadata && typeof metadata.projectCode === 'string';
}

export function isReportMetadata(metadata: ItemMetadata | undefined): metadata is ReportMetadata {
  return !!metadata && 'format' in metadata && typeof metadata.format === 'string';
}

// Safe property accessors
export function getMetadataProperty<T extends ItemMetadata, K extends keyof T>(
  metadata: ItemMetadata | undefined,
  property: K,
  defaultValue: string = ''
): string {
  if (!metadata || !(property in metadata)) {
    return defaultValue;
  }
  const value = (metadata as any)[property];
  return value != null ? String(value) : defaultValue;
}

export function getMetadataId(metadata: ItemMetadata | undefined): string | undefined {
  if (!metadata || !('id' in metadata)) return undefined;
  return (metadata as any).id;
}

export function getMetadataDescription(metadata: ItemMetadata | undefined): string {
  if (!metadata || !('description' in metadata)) return '';
  return String((metadata as any).description || '');
} 