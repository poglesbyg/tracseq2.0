#!/usr/bin/env python3
"""
Development API Gateway for TracSeq 2.0
Provides basic API endpoints for testing upload functionality
"""

import json
import os
import sqlite3
import uuid
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List

from fastapi import FastAPI, File, Form, HTTPException, UploadFile
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from pydantic import BaseModel

app = FastAPI(title="TracSeq 2.0 Dev API Gateway", version="1.0.0")

# Enable CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:5173", "http://localhost:3000"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Database connection
DATABASE_PATH = "./dev_database.db"
UPLOAD_DIR = Path("./uploads")

def get_db_connection():
    return sqlite3.connect(DATABASE_PATH)

def dict_factory(cursor, row):
    d = {}
    for idx, col in enumerate(cursor.description):
        d[col[0]] = row[idx]
    return d

class UploadResponse(BaseModel):
    success: bool
    data: List[Dict[str, Any]] = None
    message: str

@app.get("/health")
async def health_check():
    return {"status": "healthy", "service": "dev-api-gateway"}

@app.get("/api/dashboard/stats")
async def get_dashboard_stats():
    conn = get_db_connection()
    conn.row_factory = dict_factory
    cursor = conn.cursor()
    
    # Get counts
    cursor.execute("SELECT COUNT(*) as count FROM samples")
    samples_count = cursor.fetchone()["count"]
    
    cursor.execute("SELECT COUNT(*) as count FROM templates")
    templates_count = cursor.fetchone()["count"]
    
    cursor.execute("SELECT COUNT(*) as count FROM spreadsheet_datasets")
    datasets_count = cursor.fetchone()["count"]
    
    cursor.execute("SELECT COUNT(*) as count FROM rag_submissions")
    rag_count = cursor.fetchone()["count"]
    
    conn.close()
    
    return {
        "samples": {"total": samples_count, "active": samples_count},
        "templates": {"total": templates_count},
        "datasets": {"total": datasets_count},
        "rag_submissions": {"total": rag_count},
        "storage": {"locations": 5, "capacity": 85.5}
    }

@app.get("/api/samples")
async def get_samples():
    conn = get_db_connection()
    conn.row_factory = dict_factory
    cursor = conn.cursor()
    
    cursor.execute("SELECT * FROM samples ORDER BY created_at DESC")
    samples = cursor.fetchall()
    conn.close()
    
    return samples

@app.get("/api/templates")
async def get_templates():
    conn = get_db_connection()
    conn.row_factory = dict_factory
    cursor = conn.cursor()
    
    cursor.execute("SELECT * FROM templates ORDER BY created_at DESC")
    templates = cursor.fetchall()
    conn.close()
    
    return templates

@app.get("/api/spreadsheets/datasets")
async def get_datasets():
    conn = get_db_connection()
    conn.row_factory = dict_factory
    cursor = conn.cursor()
    
    cursor.execute("SELECT * FROM spreadsheet_datasets ORDER BY created_at DESC")
    datasets = cursor.fetchall()
    conn.close()
    
    return datasets

@app.get("/api/rag/submissions")
async def get_rag_submissions():
    conn = get_db_connection()
    conn.row_factory = dict_factory
    cursor = conn.cursor()
    
    cursor.execute("SELECT * FROM rag_submissions ORDER BY created_at DESC")
    submissions = cursor.fetchall()
    conn.close()
    
    return submissions

@app.post("/api/spreadsheets/upload-multiple")
async def upload_spreadsheet(
    file: UploadFile = File(...),
    uploaded_by: str = Form(None),
    selected_sheets: str = Form(None)
):
    """Upload and process spreadsheet files"""
    try:
        # Validate file type
        allowed_types = ['.csv', '.xlsx', '.xls']
        file_ext = '.' + file.filename.split('.')[-1].lower()
        
        if file_ext not in allowed_types:
            raise HTTPException(status_code=400, detail=f"Unsupported file type: {file_ext}")
        
        # Create upload directory
        UPLOAD_DIR.mkdir(exist_ok=True)
        spreadsheet_dir = UPLOAD_DIR / "spreadsheets"
        spreadsheet_dir.mkdir(exist_ok=True)
        
        # Save file
        file_id = str(uuid.uuid4())
        saved_filename = f"{file_id}_{file.filename}"
        file_path = spreadsheet_dir / saved_filename
        
        content = await file.read()
        with open(file_path, "wb") as f:
            f.write(content)
        
        # Parse selected sheets
        sheets = []
        if selected_sheets:
            try:
                sheets = json.loads(selected_sheets)
            except:
                sheets = [selected_sheets]
        
        # Create database entries
        conn = get_db_connection()
        cursor = conn.cursor()
        
        datasets = []
        for i, sheet in enumerate(sheets if sheets else [""]):
            dataset_id = str(uuid.uuid4())
            
            cursor.execute("""
                INSERT INTO spreadsheet_datasets 
                (id, filename, original_filename, file_type, file_size, uploaded_by, metadata)
                VALUES (?, ?, ?, ?, ?, ?, ?)
            """, (
                dataset_id,
                saved_filename,
                file.filename,
                file_ext.replace('.', ''),
                len(content),
                uploaded_by or "anonymous",
                json.dumps({"sheet_name": sheet} if sheet else {})
            ))
            
            datasets.append({
                "id": dataset_id,
                "filename": saved_filename,
                "original_filename": file.filename,
                "file_type": file_ext.replace('.', ''),
                "sheet_name": sheet
            })
        
        conn.commit()
        conn.close()
        
        return UploadResponse(
            success=True,
            data=datasets,
            message=f"Successfully uploaded {file.filename}"
        )
        
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/api/rag/process")
async def process_rag_document(file: UploadFile = File(...)):
    """Process RAG documents"""
    try:
        # Create upload directory
        UPLOAD_DIR.mkdir(exist_ok=True)
        docs_dir = UPLOAD_DIR / "documents"
        docs_dir.mkdir(exist_ok=True)
        
        # Save file
        file_id = str(uuid.uuid4())
        saved_filename = f"{file_id}_{file.filename}"
        file_path = docs_dir / saved_filename
        
        content = await file.read()
        with open(file_path, "wb") as f:
            f.write(content)
        
        # Create database entry
        conn = get_db_connection()
        cursor = conn.cursor()
        
        submission_id = f"RAG-{str(uuid.uuid4())[:8]}"
        
        cursor.execute("""
            INSERT INTO rag_submissions 
            (submission_id, filename, file_path, extracted_data, confidence_score)
            VALUES (?, ?, ?, ?, ?)
        """, (
            submission_id,
            file.filename,
            str(file_path),
            json.dumps({"status": "processed", "samples_found": 1}),
            0.85
        ))
        
        conn.commit()
        conn.close()
        
        return {
            "success": True,
            "submission_id": submission_id,
            "confidence_score": 0.85,
            "samples_found": 1,
            "message": "Document processed successfully"
        }
        
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/api/templates/upload")
async def upload_template(file: UploadFile = File(...)):
    """Upload template files"""
    try:
        # Create upload directory
        UPLOAD_DIR.mkdir(exist_ok=True)
        templates_dir = UPLOAD_DIR / "templates"
        templates_dir.mkdir(exist_ok=True)
        
        # Save file
        file_id = str(uuid.uuid4())
        saved_filename = f"{file_id}_{file.filename}"
        file_path = templates_dir / saved_filename
        
        content = await file.read()
        with open(file_path, "wb") as f:
            f.write(content)
        
        # Create database entry
        conn = get_db_connection()
        cursor = conn.cursor()
        
        template_id = str(uuid.uuid4())
        
        cursor.execute("""
            INSERT INTO templates 
            (id, name, description, template_data)
            VALUES (?, ?, ?, ?)
        """, (
            template_id,
            file.filename.replace('.xlsx', '').replace('.csv', ''),
            f"Uploaded template: {file.filename}",
            json.dumps({"file_path": str(file_path), "original_name": file.filename})
        ))
        
        conn.commit()
        conn.close()
        
        return {
            "success": True,
            "id": template_id,
            "name": file.filename,
            "message": "Template uploaded successfully"
        }
        
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8089)