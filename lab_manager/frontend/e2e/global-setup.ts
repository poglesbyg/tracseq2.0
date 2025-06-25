import { chromium, FullConfig } from '@playwright/test';

/**
 * Global setup for TracSeq 2.0 Laboratory Management System E2E Tests
 * - Initialize test databases
 * - Seed test data (users, samples, etc.)
 * - Set up authentication tokens
 */
async function globalSetup(config: FullConfig) {
    console.log('🧪 Setting up TracSeq 2.0 E2E test environment...');

    const { baseURL } = config.projects[0].use;
    if (!baseURL) {
        throw new Error('baseURL is required for E2E tests');
    }

    const browser = await chromium.launch();

    try {
        // Create test users and store authentication data
        await setupTestUsers(browser, baseURL);

        // Initialize laboratory test data
        await setupLaboratoryData(browser, baseURL);

        console.log('✅ TracSeq 2.0 E2E test environment ready');
    } catch (error) {
        console.error('❌ Failed to setup test environment:', error);
        throw error;
    } finally {
        await browser.close();
    }
}

/**
 * Create test users with different roles for authentication testing
 */
async function setupTestUsers(browser: any, _baseURL: string) {
    const context = await browser.newContext();
    // Page setup for future user creation logic

    try {
        console.log('👥 Setting up test users...');

        // Test user credentials stored for reuse in tests
        const testUsers = {
            admin: {
                email: 'admin.test@tracseq.com',
                password: 'AdminTest123!',
                firstName: 'Admin',
                lastName: 'Test',
                role: 'LabAdministrator'
            },
            researcher: {
                email: 'researcher.test@tracseq.com',
                password: 'ResearchTest123!',
                firstName: 'Research',
                lastName: 'Scientist',
                role: 'ResearchScientist'
            },
            technician: {
                email: 'tech.test@tracseq.com',
                password: 'TechTest123!',
                firstName: 'Lab',
                lastName: 'Technician',
                role: 'LabTechnician'
            }
        };

        // Store test user data for access in tests
        process.env.TEST_USERS = JSON.stringify(testUsers);

        console.log('✅ Test users configured');
    } catch (error) {
        console.error('❌ Failed to setup test users:', error);
        throw error;
    } finally {
        await context.close();
    }
}

/**
 * Set up laboratory test data (samples, projects, equipment)
 */
async function setupLaboratoryData(browser: any, _baseURL: string) {
    const context = await browser.newContext();
    // Page setup for future data seeding logic

    try {
        console.log('🔬 Setting up laboratory test data...');

        // Laboratory test data
        const testData = {
            projects: [
                {
                    id: 'PROJ-001',
                    name: 'COVID-19 Genomic Surveillance',
                    description: 'Sequencing SARS-CoV-2 samples for variant analysis',
                    status: 'active'
                },
                {
                    id: 'PROJ-002',
                    name: 'Cancer Genomics Study',
                    description: 'Tumor and normal sample sequencing',
                    status: 'active'
                }
            ],
            samples: [
                {
                    id: 'SAM-001',
                    name: 'COVID-Sample-001',
                    type: 'RNA',
                    projectId: 'PROJ-001',
                    status: 'pending'
                },
                {
                    id: 'SAM-002',
                    name: 'Tumor-Sample-001',
                    type: 'DNA',
                    projectId: 'PROJ-002',
                    status: 'processing'
                }
            ],
            equipment: [
                {
                    id: 'SEQ-001',
                    name: 'Illumina NovaSeq 6000',
                    type: 'sequencer',
                    status: 'available'
                },
                {
                    id: 'STOR-001',
                    name: 'Freezer -80°C Unit 1',
                    type: 'storage',
                    status: 'active'
                }
            ]
        };

        // Store test data for access in tests
        process.env.TEST_LAB_DATA = JSON.stringify(testData);

        console.log('✅ Laboratory test data configured');
    } catch (error) {
        console.error('❌ Failed to setup laboratory data:', error);
        throw error;
    } finally {
        await context.close();
    }
}

export default globalSetup; 
