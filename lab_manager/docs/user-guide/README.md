# 👥 User Guide - Lab Manager

Welcome to the Lab Manager user guide! This section provides role-specific documentation for all types of users in the laboratory management system.

## 🎯 Quick Start by Role

### **🔬 Research Scientist**
You primarily submit samples and track their progress through the system.

**Getting Started:**
1. [Login & Dashboard Overview](#login--dashboard)
2. [Submit Samples via Document Upload](#sample-submission)
3. [Track Sample Status](#sample-tracking)
4. [Search & Filter Data](#data-search)

### **🧪 Lab Technician** 
You process samples, manage storage, and perform quality control.

**Getting Started:**
1. [Process Sample Submissions](#sample-processing)
2. [Manage Storage Locations](#storage-management)
3. [Update Sample States](#sample-state-management)
4. [Generate Reports](#reporting)

### **👨‍💼 Principal Investigator**
You oversee projects, approve samples, and generate reports.

**Getting Started:**
1. [Project Overview Dashboard](#project-dashboard)
2. [Approve Sample State Changes](#sample-approval)
3. [Generate Project Reports](#project-reporting)
4. [Manage Team Access](#team-management)

### **⚙️ Lab Administrator**
You manage users, configure system settings, and monitor operations.

**Getting Started:**
1. [User Management](#user-management)
2. [System Configuration](#system-configuration)
3. [Storage Location Setup](#storage-setup)
4. [System Monitoring](#monitoring)

## 🔐 Login & Dashboard

### First Time Login
1. Navigate to the Lab Manager application
2. Enter your credentials provided by your lab administrator
3. Complete any required profile setup
4. Familiarize yourself with the dashboard layout

### Dashboard Overview
The dashboard provides role-specific widgets:

**Research Scientist Dashboard:**
- Recent sample submissions
- Sample status overview
- Quick submission links
- Personal sample statistics

**Lab Technician Dashboard:**
- Pending processing queue
- Storage capacity alerts
- Recent sample movements
- Quality control metrics

**Principal Investigator Dashboard:**
- Project sample overview
- Approval queue
- Team activity summary
- Compliance reports

**Lab Administrator Dashboard:**
- System health status
- User activity metrics
- Storage utilization
- System alerts

## 🧪 Sample Management

### Sample Submission

#### **Method 1: Document Upload (Recommended)**
1. Navigate to **Samples** → **Submit Samples**
2. Click **Upload Document**
3. Select your spreadsheet or document (CSV, XLS, XLSX supported)
4. Review the AI-extracted data
5. Correct any low-confidence extractions
6. Click **Submit Samples**

#### **Method 2: Manual Entry**
1. Navigate to **Samples** → **Create Sample**
2. Fill in required fields:
   - Sample Name (minimum 3 characters)
   - Sample Type (Blood, Saliva, Tissue, etc.)
   - Volume and Concentration
   - Temperature Requirements
3. Add optional metadata
4. Click **Create Sample**

#### **Method 3: Template-Based Upload**
1. Navigate to **Templates** → **Select Template**
2. Download the template spreadsheet
3. Fill in your sample data following the template format
4. Upload the completed template
5. Review and submit

### Sample Tracking

#### **View Sample Status**
1. Navigate to **Samples** → **My Samples** or **All Samples**
2. Use the search bar to find specific samples
3. Click on any sample to view detailed information
4. Check the **Status** field for current state:
   - **Pending**: Awaiting validation
   - **Validated**: Ready for storage
   - **In Storage**: Located in storage system
   - **In Sequencing**: Being processed
   - **Completed**: Processing finished

#### **Sample State Transitions**
Sample states follow a defined workflow:
```
Pending → Validated → In Storage → In Sequencing → Completed
```

- **Pending to Validated**: Lab technician or admin approval
- **Validated to In Storage**: Assignment to storage location
- **In Storage to In Sequencing**: Removal for processing
- **In Sequencing to Completed**: Processing finished

### Data Search & Filtering

#### **Basic Search**
1. Use the main search bar at the top of any sample list
2. Enter sample names, barcodes, or metadata
3. Results update in real-time

#### **Advanced Filtering**
1. Click **Filters** button on sample list pages
2. Available filters include:
   - **Sample Type**: Blood, Saliva, Tissue, etc.
   - **Status**: Current processing state
   - **Date Range**: Creation or processing dates
   - **Storage Location**: Physical location
   - **Temperature Zone**: Storage temperature
   - **Department**: Submitting department
   - **Created By**: Sample submitter

#### **Exporting Data**
1. Apply desired filters to your sample list
2. Click **Export** button
3. Choose format: CSV, JSON, or Excel
4. File will download automatically

## 🏪 Storage Management

### Storage Location Overview

The storage system is organized hierarchically:
```
Building → Room → Equipment → Container → Position
```

**Temperature Zones:**
- **-80°C**: Long-term storage (deep freezers)
- **-20°C**: Medium-term storage (freezers)
- **4°C**: Short-term storage (refrigerators)
- **RT**: Room temperature storage
- **37°C**: Incubator storage

### Assigning Samples to Storage

1. Navigate to sample details page
2. Click **Assign to Storage**
3. Select appropriate location based on:
   - Temperature compatibility
   - Available capacity
   - Container type requirements
4. Confirm assignment
5. Sample status updates to "In Storage"

### Managing Storage Capacity

#### **Capacity Monitoring**
- Green: <80% capacity (optimal)
- Yellow: 80-95% capacity (warning)
- Red: >95% capacity (critical)

#### **Capacity Alerts**
The system automatically generates alerts when:
- Storage location reaches 80% capacity
- Critical capacity (95%) is reached
- Temperature monitoring detects issues

### Moving Samples Between Locations

1. Go to sample details or storage overview
2. Click **Move Sample**
3. Select new location
4. Enter reason for movement
5. System logs the movement in audit trail
6. Chain of custody is automatically maintained

## 📊 Data Analysis & Reporting

### Spreadsheet Data Viewer

#### **Viewing Uploaded Data**
1. Navigate to **Data** → **Spreadsheets**
2. Select a dataset from the list
3. Use the data viewer to:
   - Search across all columns
   - Filter by specific values
   - Sort by any column
   - Export filtered results

#### **Data Viewer Features**
- **Real-time Search**: Type to filter results instantly
- **Column Filters**: Click column headers for specific filters
- **Pagination**: Navigate large datasets efficiently
- **Export Options**: CSV, JSON export of filtered data
- **Full-screen Mode**: Maximize viewing area

### Generating Reports

#### **Standard Reports**
1. Navigate to **Reports** → **Standard Reports**
2. Choose from available reports:
   - Sample Statistics
   - Storage Utilization
   - User Activity
   - Audit Trail
3. Select date range and other parameters
4. Click **Generate Report**

#### **Custom SQL Reports** (Advanced Users)
1. Navigate to **Reports** → **Custom Reports**
2. Write SQL query using available tables
3. Test query with **Preview** button
4. Save report for future use
5. Schedule automatic generation if needed

## 🔧 Administrative Functions

### User Management (Lab Administrators)

#### **Creating Users**
1. Navigate to **Admin** → **Users**
2. Click **Create User**
3. Fill in user information:
   - Name and email
   - Department and role
   - Access permissions
4. Send invitation email to user
5. User completes setup on first login

#### **User Roles**
- **Lab Administrator**: Full system access
- **Principal Investigator**: Project oversight and approvals
- **Lab Technician**: Sample processing and storage
- **Research Scientist**: Sample submission and data access
- **Data Analyst**: Read-only access to data and reports
- **Guest**: Limited read access

### System Configuration

#### **Storage Location Setup**
1. Navigate to **Admin** → **Storage**
2. Click **Add Location**
3. Configure:
   - Location hierarchy (Building/Room/Equipment)
   - Temperature zone
   - Container types supported
   - Capacity limits
4. Save configuration

#### **Template Management**
1. Navigate to **Admin** → **Templates**
2. Create custom templates for your lab's needs
3. Define required and optional fields
4. Set validation rules
5. Make available to users

### Monitoring & Maintenance

#### **System Health**
1. Navigate to **Admin** → **System Health**
2. Monitor:
   - Service status (Backend, RAG, Database)
   - Performance metrics
   - Error rates
   - Storage utilization

#### **Audit Trail**
1. Navigate to **Admin** → **Audit Trail**
2. Review all system activities:
   - User logins and actions
   - Sample state changes
   - Data modifications
   - System configuration changes

## 🆘 Troubleshooting

### Common Issues

#### **Cannot Login**
- Verify username/email and password
- Check with lab administrator for account status
- Clear browser cache and cookies
- Try different browser

#### **Sample Upload Fails**
- Check file format (CSV, XLS, XLSX supported)
- Verify file size (maximum 10MB)
- Ensure required columns are present
- Check for special characters in data

#### **Cannot Assign Sample to Storage**
- Verify temperature compatibility
- Check storage capacity
- Ensure container type is supported
- Contact lab technician for assistance

#### **Data Not Appearing**
- Check filters and search criteria
- Verify permissions for the data
- Refresh the page
- Contact administrator if issue persists

### Getting Help

#### **In-App Help**
- Look for **?** icons throughout the interface
- Click for context-specific help

#### **Contact Support**
- **Email**: support@lab-manager.dev
- **Internal Help**: Contact your lab administrator
- **Documentation**: Reference this user guide

#### **Reporting Issues**
1. Note the exact error message
2. Record steps to reproduce the issue
3. Take screenshots if helpful
4. Contact your lab administrator or IT support

## 📚 Best Practices

### **For All Users**
- Log out when finished with session
- Use strong, unique passwords
- Report any unusual system behavior
- Keep sample data accurate and up-to-date

### **For Sample Submission**
- Use standardized naming conventions
- Include all required metadata
- Double-check data before submission
- Use templates when available

### **For Data Management**
- Regular backup of important data
- Proper categorization of samples
- Consistent metadata entry
- Follow laboratory SOPs

### **For Storage Management**
- Monitor capacity regularly
- Document all sample movements
- Maintain temperature logs
- Follow chain of custody procedures

---

**Need additional help?** Contact your lab administrator or refer to the [technical documentation](../README.md) for more detailed information.

*Context improved by Giga AI* 
