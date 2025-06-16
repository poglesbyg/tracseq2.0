use reqwest::{multipart, Client};
use serde_json::Value;
use std::error::Error;
use tokio::fs;

/// Demo of RAG Integration with Lab Manager
///
/// This example shows how to:
/// 1. Process laboratory documents using the RAG system
/// 2. Extract sample information automatically
/// 3. Create samples from AI-extracted data
/// 4. Query the system for submission information
///
/// Prerequisites:
/// 1. Lab Manager server running on http://localhost:3000
/// 2. RAG LLM system running on http://localhost:8000
/// 3. Sample laboratory documents in supported formats (PDF, DOCX)

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🧬🤖 Laboratory RAG Integration Demo");
    println!("=====================================\n");

    let client = Client::new();
    let lab_manager_url = "http://localhost:3000";
    let rag_system_url = "http://localhost:8000";

    // Step 1: Check system health
    println!("🏥 Checking system health...");
    check_system_health(&client, lab_manager_url, rag_system_url).await?;

    // Step 2: Process a laboratory document
    println!("\n📄 Processing laboratory document with RAG...");
    process_lab_document(&client, lab_manager_url).await?;

    // Step 3: Preview document extraction
    println!("\n👁️ Previewing document extraction...");
    preview_document_extraction(&client, lab_manager_url).await?;

    // Step 4: Query submission information
    println!("\n❓ Querying submission information...");
    query_submission_info(&client, lab_manager_url).await?;

    // Step 5: Demonstrate batch processing workflow
    println!("\n📚 Batch processing workflow...");
    demonstrate_batch_workflow(&client, lab_manager_url).await?;

    println!("\n✅ RAG Integration demo completed successfully!");
    println!("\n💡 Next steps:");
    println!("   - Upload your own laboratory documents");
    println!("   - Customize extraction confidence thresholds");
    println!("   - Integrate with existing lab workflows");
    println!("   - Set up automated document processing pipelines");

    Ok(())
}

async fn check_system_health(
    client: &Client,
    lab_manager_url: &str,
    rag_system_url: &str,
) -> Result<(), Box<dyn Error>> {
    // Check Lab Manager health
    let lab_health = client
        .get(&format!("{}/health", lab_manager_url))
        .send()
        .await?;

    if lab_health.status().is_success() {
        println!("  ✅ Lab Manager: Healthy");
    } else {
        println!("  ❌ Lab Manager: Unhealthy");
        return Err("Lab Manager is not healthy".into());
    }

    // Check RAG system health through Lab Manager
    let rag_health = client
        .get(&format!("{}/api/samples/rag/status", lab_manager_url))
        .send()
        .await?;

    if rag_health.status().is_success() {
        println!("  ✅ RAG System: Healthy");
        let health_data: Value = rag_health.json().await?;
        if let Some(status) = health_data.get("status") {
            println!("     Status: {}", status);
        }
    } else {
        println!("  ⚠️ RAG System: Connection issues");
        println!(
            "     Make sure the RAG system is running on {}",
            rag_system_url
        );
    }

    Ok(())
}

async fn process_lab_document(
    client: &Client,
    lab_manager_url: &str,
) -> Result<(), Box<dyn Error>> {
    // Create a sample lab document (in real usage, you'd have actual lab forms)
    let sample_document_content = create_sample_lab_document();
    let temp_file = "temp_lab_form.txt";

    fs::write(temp_file, sample_document_content).await?;

    // Create multipart form with document
    let file_part = multipart::Part::bytes(fs::read(temp_file).await?)
        .file_name("lab_submission_form.txt")
        .mime_str("text/plain")?;

    let form = multipart::Form::new()
        .part("document", file_part)
        .text("confidence_threshold", "0.7");

    // Process document
    let response = client
        .post(&format!(
            "{}/api/samples/rag/process-document",
            lab_manager_url
        ))
        .multipart(form)
        .send()
        .await?;

    if response.status().is_success() {
        let result: Value = response.json().await?;

        println!("  ✅ Document processed successfully!");
        println!(
            "     Confidence Score: {:.2}",
            result["confidence_score"].as_f64().unwrap_or(0.0)
        );
        println!(
            "     Samples Extracted: {}",
            result["samples"].as_array().map(|s| s.len()).unwrap_or(0)
        );

        if let Some(warnings) = result["validation_warnings"].as_array() {
            if !warnings.is_empty() {
                println!("     Validation Warnings:");
                for warning in warnings {
                    println!("       - {}", warning.as_str().unwrap_or("Unknown warning"));
                }
            }
        }

        // Display extracted samples
        if let Some(samples) = result["samples"].as_array() {
            println!("\n     📋 Extracted Samples:");
            for (i, sample) in samples.iter().enumerate() {
                println!(
                    "       {}. Name: {}",
                    i + 1,
                    sample["name"].as_str().unwrap_or("Unknown")
                );
                println!(
                    "          Barcode: {}",
                    sample["barcode"].as_str().unwrap_or("Unknown")
                );
                println!(
                    "          Location: {}",
                    sample["location"].as_str().unwrap_or("Unknown")
                );
            }
        }
    } else {
        println!("  ❌ Document processing failed: {}", response.status());
        let error_text = response.text().await?;
        println!("     Error: {}", error_text);
    }

    // Clean up
    let _ = fs::remove_file(temp_file).await;

    Ok(())
}

async fn preview_document_extraction(
    client: &Client,
    lab_manager_url: &str,
) -> Result<(), Box<dyn Error>> {
    // Create another sample document for preview
    let preview_document_content = create_complex_lab_document();
    let temp_file = "temp_preview_form.txt";

    fs::write(temp_file, preview_document_content).await?;

    let file_part = multipart::Part::bytes(fs::read(temp_file).await?)
        .file_name("complex_lab_form.txt")
        .mime_str("text/plain")?;

    let form = multipart::Form::new().part("document", file_part);

    let response = client
        .post(&format!("{}/api/samples/rag/preview", lab_manager_url))
        .multipart(form)
        .send()
        .await?;

    if response.status().is_success() {
        let result: Value = response.json().await?;

        println!("  🔍 Document preview completed!");
        println!(
            "     Confidence Score: {:.2}",
            result["confidence_score"].as_f64().unwrap_or(0.0)
        );
        println!(
            "     Processing Time: {:.2}s",
            result["processing_time"].as_f64().unwrap_or(0.0)
        );

        if let Some(extraction) = result["extraction_result"].as_object() {
            if let Some(submission) = extraction["submission"].as_object() {
                println!("\n     📊 Extracted Information Categories:");

                // Administrative info
                if let Some(admin) = submission["administrative_info"].as_object() {
                    println!(
                        "       👤 Administrative: {} {} ({})",
                        admin["submitter_first_name"].as_str().unwrap_or("Unknown"),
                        admin["submitter_last_name"].as_str().unwrap_or("Unknown"),
                        admin["submitter_email"].as_str().unwrap_or("Unknown")
                    );
                }

                // Source material
                if let Some(source) = submission["source_material"].as_object() {
                    println!(
                        "       🧪 Source Material: {}",
                        source["source_type"].as_str().unwrap_or("Unknown")
                    );
                }

                // Sequencing info
                if let Some(sequencing) = submission["sequence_generation"].as_object() {
                    if let Some(platform) = sequencing["sequencing_platform"].as_str() {
                        println!("       🔬 Sequencing Platform: {}", platform);
                    }
                }
            }
        }
    } else {
        println!("  ❌ Preview failed: {}", response.status());
    }

    let _ = fs::remove_file(temp_file).await;
    Ok(())
}

async fn query_submission_info(
    client: &Client,
    lab_manager_url: &str,
) -> Result<(), Box<dyn Error>> {
    let queries = vec![
        "What sequencing platform is being used?",
        "Who is the submitter for this project?",
        "What type of sample analysis is requested?",
        "What are the storage requirements?",
        "What is the sample priority level?",
    ];

    for query in queries {
        let request_body = serde_json::json!({
            "query": query
        });

        let response = client
            .post(&format!("{}/api/samples/rag/query", lab_manager_url))
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let result: Value = response.json().await?;
            println!("  ❓ Q: {}", query);
            println!(
                "     A: {}",
                result["answer"].as_str().unwrap_or("No answer available")
            );
            println!();
        }
    }

    Ok(())
}

async fn demonstrate_batch_workflow(
    client: &Client,
    lab_manager_url: &str,
) -> Result<(), Box<dyn Error>> {
    println!("  📁 Simulating batch document processing workflow...");

    // In a real scenario, you would:
    // 1. Monitor a directory for new documents
    // 2. Process multiple documents in parallel
    // 3. Validate and create samples in batches
    // 4. Handle errors and retries

    println!("     1. Document monitoring: ✅ (Would scan upload directory)");
    println!("     2. Parallel processing: ✅ (Would process multiple docs)");
    println!("     3. Batch validation: ✅ (Would validate sample data)");
    println!("     4. Error handling: ✅ (Would retry failed extractions)");
    println!("     5. Sample creation: ✅ (Would create validated samples)");

    println!("\n  💡 Workflow Benefits:");
    println!("     - Automated data extraction from lab forms");
    println!("     - Reduced manual data entry errors");
    println!("     - Consistent sample metadata formatting");
    println!("     - Intelligent validation and error detection");
    println!("     - Scalable processing for high-volume labs");

    Ok(())
}

fn create_sample_lab_document() -> String {
    r#"
LABORATORY SAMPLE SUBMISSION FORM

ADMINISTRATIVE INFORMATION:
Submitter: Dr. Sarah Johnson
Email: sarah.johnson@research.edu
Phone: (555) 123-4567
Project: PROJ-2024-WGS-001
Department: Molecular Biology
Institution: Research University

SOURCE MATERIAL:
Sample Type: Genomic DNA
Source: Blood samples
Collection Date: 2024-01-15
Organism: Homo sapiens
Tissue Type: Whole blood
Preservation: EDTA tubes, -80°C storage

SEQUENCING REQUIREMENTS:
Platform: Illumina NovaSeq 6000
Read Type: Paired-end
Read Length: 150 bp
Target Coverage: 30x
Library Prep: TruSeq DNA PCR-Free

SAMPLE DETAILS:
Sample ID: SAMPLE-WGS-001
Patient ID: PT-001
Priority: High
Quality Score: 9.2/10
Volume: 5.0 mL
Concentration: 250 ng/μL

ANALYSIS REQUIREMENTS:
Analysis Type: Whole Genome Sequencing (WGS)
Reference Genome: GRCh38
Pipeline: GATK Best Practices
Special Instructions: Focus on rare variants
"#
    .to_string()
}

fn create_complex_lab_document() -> String {
    r#"
MULTI-SAMPLE LABORATORY SUBMISSION

ADMINISTRATIVE:
PI: Prof. Michael Chen
Contact: m.chen@university.edu
Phone: (555) 987-6543
Project: RNA-SEQ-2024-CANCER
Grant: NIH-R01-123456

POOLING INFORMATION:
Pooled Samples: Yes
Pool ID: POOL-RNA-001
Samples in Pool: RNA-001, RNA-002, RNA-003, RNA-004
Barcode Strategy: Unique dual indexing
Multiplex Level: 4-plex

SEQUENCING PARAMETERS:
Platform: Illumina NextSeq 2000
Read Configuration: Single-end
Read Length: 75 bp
Target Depth: 50M reads per sample
Library Kit: TruSeq Stranded mRNA

SAMPLE DETAILS:
Sample 1: RNA-001, Tumor tissue, High priority
Sample 2: RNA-002, Normal tissue, High priority  
Sample 3: RNA-003, Tumor tissue, Medium priority
Sample 4: RNA-004, Normal tissue, Medium priority

INFORMATICS:
Analysis: RNA-seq differential expression
Reference: GRCh38 with Ensembl v104
Pipeline: STAR alignment + DESeq2
Comparisons: Tumor vs Normal pairs
"#
    .to_string()
}
