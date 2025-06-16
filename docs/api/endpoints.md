# API Endpoints

## Authentication

All API endpoints require authentication using JWT tokens. Include the token in the Authorization header:

```bash
Authorization: Bearer <your-token>
```

## Sample Endpoints

### Create Sample

Creates a new sample using a template.

```http
POST /api/samples
Content-Type: application/json
```

#### Request Body

```typescript
interface CreateSampleRequest {
  template_id: string;
  metadata: {
    [key: string]: any;  // Template-defined fields
  };
  source: {
    type: string;        // Source type (patient, study, etc.)
    identifier: string;  // Source identifier
  };
  collection: {
    date: string;        // Collection date (ISO 8601)
    location: string;    // Collection location
    collector: string;   // Collector identifier
  };
}
```

#### Example Request

```json
{
  "template_id": "DNA-001",
  "metadata": {
    "sample_type": "Blood",
    "volume": "5ml",
    "collection_date": "2024-03-20"
  },
  "source": {
    "type": "patient",
    "identifier": "P123"
  },
  "collection": {
    "date": "2024-03-20T10:00:00Z",
    "location": "Main Lab",
    "collector": "JOHN_DOE"
  }
}
```

#### Response

```typescript
interface CreateSampleResponse {
  id: string;
  barcode: string;
  status: SampleStatus;
  created_at: string;
  template: {
    id: string;
    name: string;
    version: string;
  };
}
```

#### Status Codes

- `201 Created`: Sample created successfully
- `400 Bad Request`: Invalid request data
- `401 Unauthorized`: Missing or invalid token
- `404 Not Found`: Template not found
- `422 Unprocessable Entity`: Validation failed

### Get Sample

Retrieves a sample by ID or barcode.

```http
GET /api/samples/{id}
```

#### Path Parameters

- `id`: Sample ID or barcode

#### Response

```typescript
interface GetSampleResponse {
  id: string;
  barcode: string;
  status: SampleStatus;
  metadata: {
    [key: string]: any;
  };
  source: {
    type: string;
    identifier: string;
  };
  collection: {
    date: string;
    location: string;
    collector: string;
  };
  storage?: {
    location: Location;
    placed_at: string;
    placed_by: string;
  };
  created_at: string;
  updated_at: string;
  template: {
    id: string;
    name: string;
    version: string;
  };
}
```

#### Status Codes

- `200 OK`: Sample found
- `401 Unauthorized`: Missing or invalid token
- `404 Not Found`: Sample not found

### List Samples

Retrieves a paginated list of samples with filtering options.

```http
GET /api/samples
```

#### Query Parameters

- `page`: Page number (default: 1)
- `per_page`: Items per page (default: 20, max: 100)
- `status`: Filter by status
- `template_id`: Filter by template
- `source_type`: Filter by source type
- `source_id`: Filter by source identifier
- `created_after`: Filter by creation date
- `created_before`: Filter by creation date
- `sort_by`: Sort field (default: created_at)
- `sort_order`: Sort order (asc/desc, default: desc)

#### Response

```typescript
interface ListSamplesResponse {
  items: GetSampleResponse[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}
```

#### Status Codes

- `200 OK`: Samples retrieved successfully
- `401 Unauthorized`: Missing or invalid token
- `400 Bad Request`: Invalid query parameters

### Update Sample

Updates a sample's metadata or status.

```http
PATCH /api/samples/{id}
Content-Type: application/json
```

#### Path Parameters

- `id`: Sample ID or barcode

#### Request Body

```typescript
interface UpdateSampleRequest {
  metadata?: {
    [key: string]: any;
  };
  status?: SampleStatus;
  storage?: {
    location: Location;
  };
}
```

#### Example Request

```json
{
  "status": "stored",
  "storage": {
    "location": {
      "facility": "Main Lab",
      "building": "Science Wing",
      "room": "B-101",
      "storage_unit": "Freezer-A",
      "position": "Shelf-1"
    }
  }
}
```

#### Response

```typescript
interface UpdateSampleResponse {
  id: string;
  status: SampleStatus;
  updated_at: string;
  storage?: {
    location: Location;
    placed_at: string;
    placed_by: string;
  };
}
```

#### Status Codes

- `200 OK`: Sample updated successfully
- `400 Bad Request`: Invalid request data
- `401 Unauthorized`: Missing or invalid token
- `404 Not Found`: Sample not found
- `422 Unprocessable Entity`: Validation failed

### Batch Create Samples

Creates multiple samples in a single request.

```http
POST /api/samples/batch
Content-Type: application/json
```

#### Request Body

```typescript
interface BatchCreateSamplesRequest {
  samples: CreateSampleRequest[];
}
```

#### Response

```typescript
interface BatchCreateSamplesResponse {
  created: number;
  failed: number;
  results: {
    success: CreateSampleResponse[];
    errors: {
      index: number;
      error: string;
    }[];
  };
}
```

#### Status Codes

- `201 Created`: All samples created successfully
- `207 Multi-Status`: Some samples created, some failed
- `400 Bad Request`: Invalid request data
- `401 Unauthorized`: Missing or invalid token

### Get Sample History

Retrieves the history of changes for a sample.

```http
GET /api/samples/{id}/history
```

#### Path Parameters

- `id`: Sample ID or barcode

#### Query Parameters

- `page`: Page number (default: 1)
- `per_page`: Items per page (default: 20, max: 100)

#### Response

```typescript
interface SampleHistoryResponse {
  items: {
    timestamp: string;
    action: string;
    user: string;
    changes: {
      field: string;
      old_value: any;
      new_value: any;
    }[];
  }[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}
```

#### Status Codes

- `200 OK`: History retrieved successfully
- `401 Unauthorized`: Missing or invalid token
- `404 Not Found`: Sample not found

## Error Responses

All endpoints return errors in a consistent format:

```typescript
interface ErrorResponse {
  error: {
    code: string;
    message: string;
    details?: {
      [key: string]: string[];
    };
  };
}
```

### Common Error Codes

- `INVALID_REQUEST`: Invalid request data
- `UNAUTHORIZED`: Missing or invalid token
- `FORBIDDEN`: Insufficient permissions
- `NOT_FOUND`: Resource not found
- `VALIDATION_FAILED`: Request validation failed
- `INTERNAL_ERROR`: Server error

## Rate Limiting

API requests are rate-limited to:
- 100 requests per minute per IP
- 1000 requests per hour per user

Rate limit headers are included in all responses:
- `X-RateLimit-Limit`: Request limit per window
- `X-RateLimit-Remaining`: Remaining requests
- `X-RateLimit-Reset`: Time until limit resets

## Related Topics

- [Authentication](authentication.md)
- [Error Handling](error-handling.md)
- [Sample Processing](../user-guide/sample-processing.md) 
