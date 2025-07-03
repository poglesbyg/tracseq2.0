use async_trait::async_trait;
use calamine::{open_workbook_auto, DataType, Range, Reader};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

use crate::{
    models::template::{CreateTemplate, SheetData, SpreadsheetData, Template, UpdateTemplate},
    repositories::Repository,
    services::{HealthCheck, HealthStatus, Service, ServiceConfig, ServiceHealth},
};

pub struct TemplateService<R>
where
    R: Repository<Template, CreateTemplate, UpdateTemplate> + Send + Sync,
{
    repository: R,
}

impl<R> TemplateService<R>
where
    R: Repository<Template, CreateTemplate, UpdateTemplate> + Send + Sync,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn create_template(&self, template: CreateTemplate) -> Result<Template, R::Error> {
        self.repository.create(template).await
    }

    pub async fn get_template(&self, template_id: Uuid) -> Result<Option<Template>, R::Error> {
        self.repository.find_by_id(template_id).await
    }

    pub async fn list_templates(&self) -> Result<Vec<Template>, R::Error> {
        self.repository.list(None, None).await
    }

    pub async fn update_template(
        &self,
        template_id: Uuid,
        updates: UpdateTemplate,
    ) -> Result<Template, R::Error> {
        self.repository.update(template_id, updates).await
    }

    pub async fn delete_template(&self, template_id: Uuid) -> Result<(), R::Error> {
        self.repository.delete(template_id).await
    }

    // New method to parse spreadsheet data
    pub async fn parse_spreadsheet(
        &self,
        file_path: &str,
    ) -> Result<SpreadsheetData, anyhow::Error> {
        let path = Path::new(file_path);

        // Check file extension to determine parsing method
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            match extension.to_lowercase().as_str() {
                "csv" => self.parse_csv_file(path).await,
                "xlsx" | "xls" => self.parse_excel_file(path).await,
                _ => Err(anyhow::anyhow!("Unsupported file format: {}", extension)),
            }
        } else {
            Err(anyhow::anyhow!("Could not determine file type"))
        }
    }

    async fn parse_csv_file(&self, path: &Path) -> Result<SpreadsheetData, anyhow::Error> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;

        if lines.is_empty() {
            return Ok(SpreadsheetData {
                sheet_names: vec!["Sheet1".to_string()],
                sheets: vec![SheetData {
                    name: "Sheet1".to_string(),
                    headers: vec![],
                    rows: vec![],
                    total_rows: 0,
                    total_columns: 0,
                }],
            });
        }

        // Parse CSV (simple comma-separated parsing)
        let headers: Vec<String> = lines[0]
            .split(',')
            .map(|h| h.trim().trim_matches('"').to_string())
            .collect();

        let mut rows = Vec::new();
        for line in lines.iter().skip(1) {
            let row: Vec<String> = line
                .split(',')
                .map(|cell| cell.trim().trim_matches('"').to_string())
                .collect();
            rows.push(row);
        }

        let sheet_data = SheetData {
            name: "Sheet1".to_string(),
            headers: headers.clone(),
            rows,
            total_rows: lines.len(),
            total_columns: headers.len(),
        };

        Ok(SpreadsheetData {
            sheet_names: vec!["Sheet1".to_string()],
            sheets: vec![sheet_data],
        })
    }

    async fn parse_excel_file(&self, path: &Path) -> Result<SpreadsheetData, anyhow::Error> {
        let mut workbook = open_workbook_auto(path)?;
        let sheet_names = workbook.sheet_names().to_vec();
        let mut sheets = Vec::new();

        for sheet_name in &sheet_names {
            match workbook.worksheet_range(sheet_name) {
                Some(Ok(range)) => {
                    let sheet_data = self.parse_sheet_range(&range, sheet_name);
                    sheets.push(sheet_data);
                }
                Some(Err(_)) | None => {
                    // Skip sheets that can't be read
                    continue;
                }
            }
        }

        Ok(SpreadsheetData {
            sheet_names,
            sheets,
        })
    }

    fn parse_sheet_range(&self, range: &Range<DataType>, sheet_name: &str) -> SheetData {
        let mut headers = Vec::new();
        let mut rows = Vec::new();

        let height = range.height();
        let width = range.width();

        if height == 0 {
            return SheetData {
                name: sheet_name.to_string(),
                headers,
                rows,
                total_rows: 0,
                total_columns: 0,
            };
        }

        // Extract headers from the first row
        if let Some(first_row) = range.rows().next() {
            for cell in first_row {
                headers.push(self.cell_to_string(cell));
            }
        }

        // Extract data rows (skip the header row)
        for (index, row) in range.rows().enumerate() {
            if index == 0 {
                continue; // Skip header row
            }

            let mut row_data = Vec::new();
            for cell in row {
                row_data.push(self.cell_to_string(cell));
            }
            rows.push(row_data);
        }

        SheetData {
            name: sheet_name.to_string(),
            headers,
            rows,
            total_rows: height,
            total_columns: width,
        }
    }

    fn cell_to_string(&self, cell: &DataType) -> String {
        match cell {
            DataType::Empty => String::new(),
            DataType::String(s) => s.clone(),
            DataType::Float(f) => f.to_string(),
            DataType::Int(i) => i.to_string(),
            DataType::Bool(b) => b.to_string(),
            DataType::Error(e) => format!("Error: {:?}", e),
            DataType::DateTime(dt) => dt.to_string(),
            DataType::Duration(d) => d.to_string(),
            DataType::DateTimeIso(dt) => dt.clone(),
            DataType::DurationIso(d) => d.clone(),
        }
    }
}

#[async_trait]
impl<R> Service for TemplateService<R>
where
    R: Repository<Template, CreateTemplate, UpdateTemplate> + Send + Sync,
{
    fn name(&self) -> &'static str {
        "template_service"
    }

    async fn health_check(&self) -> ServiceHealth {
        let mut checks = HashMap::new();

        // Test repository connectivity by listing templates
        let start = std::time::Instant::now();
        let db_check = match self.list_templates().await {
            Ok(_) => HealthCheck {
                status: HealthStatus::Healthy,
                duration_ms: start.elapsed().as_millis() as u64,
                details: Some("Repository connection successful".to_string()),
            },
            Err(_) => HealthCheck {
                status: HealthStatus::Unhealthy,
                duration_ms: start.elapsed().as_millis() as u64,
                details: Some("Repository error".to_string()),
            },
        };

        checks.insert("repository".to_string(), db_check.clone());

        ServiceHealth {
            status: db_check.status,
            message: Some("Template service health check".to_string()),
            checks,
        }
    }

    fn config(&self) -> ServiceConfig {
        ServiceConfig {
            name: "template_service".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["repository".to_string()],
            settings: HashMap::new(),
        }
    }
}
