import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useState } from 'react';
import Layout from './components/Layout';
import { ChatBot } from './components/ChatBot';
import { ChatBotFloat } from './components/ChatBotFloat';
import { AuthProvider } from './auth/AuthContext';
import { ProtectedRoute } from './auth/ProtectedRoute';
import Login from './pages/Login';
import ForgotPassword from './pages/ForgotPassword';
import Dashboard from './pages/Dashboard';
import Templates from './pages/Templates';
import Samples from './pages/Samples';
import RagSubmissions from './pages/RagSubmissions';
import RagSamples from './pages/RagSamples';
import Sequencing from './pages/Sequencing';
import Spreadsheets from './pages/Spreadsheets';
import Storage from './pages/Storage';
import Reports from './pages/Reports';
import Profile from './pages/Profile';
import Users from './pages/Users';
import LibraryPrep from './pages/LibraryPrep';
import QualityControl from './pages/QualityControl';
import ProjectManagement from './pages/ProjectManagement';
import FlowCellDesign from './pages/FlowCellDesign';
import ServicesStatus from './pages/ServicesStatus';
import { NotificationCenter } from './components/notifications/NotificationCenter';
import { EventMonitorDashboard } from './components/events/EventMonitorDashboard';
import { QAQCWorkflowInterface } from './components/qaqc/QAQCWorkflowInterface';

const queryClient = new QueryClient();

export default function App() {
  const [isChatOpen, setIsChatOpen] = useState(false);

  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <Router future={{ v7_startTransition: true, v7_relativeSplatPath: true }}>
          <Routes>
            {/* Public routes */}
            <Route path="/login" element={<Login />} />
            <Route path="/forgot-password" element={<ForgotPassword />} />
            
            {/* Protected routes */}
            <Route path="/dashboard" element={
              <ProtectedRoute>
                <Layout>
                  <Dashboard />
                  {/* RAG Chatbot */}
                  {!isChatOpen && (
                    <ChatBotFloat onClick={() => setIsChatOpen(true)} />
                  )}
                  <ChatBot 
                    isOpen={isChatOpen} 
                    onToggle={() => setIsChatOpen(!isChatOpen)} 
                  />
                </Layout>
              </ProtectedRoute>
            } />
            
            <Route path="/templates" element={
              <ProtectedRoute>
                <Layout>
                  <Templates />
                  {!isChatOpen && <ChatBotFloat onClick={() => setIsChatOpen(true)} />}
                  <ChatBot isOpen={isChatOpen} onToggle={() => setIsChatOpen(!isChatOpen)} />
                </Layout>
              </ProtectedRoute>
            } />
            
            <Route path="/samples" element={
              <ProtectedRoute>
                <Layout>
                  <Samples />
                  {!isChatOpen && <ChatBotFloat onClick={() => setIsChatOpen(true)} />}
                  <ChatBot isOpen={isChatOpen} onToggle={() => setIsChatOpen(!isChatOpen)} />
                </Layout>
              </ProtectedRoute>
            } />
            
            <Route path="/rag-submissions" element={
              <ProtectedRoute>
                <Layout>
                  <RagSubmissions />
                  {!isChatOpen && <ChatBotFloat onClick={() => setIsChatOpen(true)} />}
                  <ChatBot isOpen={isChatOpen} onToggle={() => setIsChatOpen(!isChatOpen)} />
                </Layout>
              </ProtectedRoute>
            } />
            
            <Route path="/rag-samples" element={
              <ProtectedRoute>
                <Layout>
                  <RagSamples />
                  {!isChatOpen && <ChatBotFloat onClick={() => setIsChatOpen(true)} />}
                  <ChatBot isOpen={isChatOpen} onToggle={() => setIsChatOpen(!isChatOpen)} />
                </Layout>
              </ProtectedRoute>
            } />
            
            <Route path="/library-prep" element={
              <ProtectedRoute>
                <Layout>
                  <LibraryPrep />
                  {!isChatOpen && <ChatBotFloat onClick={() => setIsChatOpen(true)} />}
                  <ChatBot isOpen={isChatOpen} onToggle={() => setIsChatOpen(!isChatOpen)} />
                </Layout>
              </ProtectedRoute>
            } />
            
            <Route path="/quality-control" element={
              <ProtectedRoute>
                <Layout>
                  <QualityControl />
                  {!isChatOpen && <ChatBotFloat onClick={() => setIsChatOpen(true)} />}
                  <ChatBot isOpen={isChatOpen} onToggle={() => setIsChatOpen(!isChatOpen)} />
                </Layout>
              </ProtectedRoute>
            } />
            
            <Route path="/projects" element={
              <ProtectedRoute>
                <Layout>
                  <ProjectManagement />
                  {!isChatOpen && <ChatBotFloat onClick={() => setIsChatOpen(true)} />}
                  <ChatBot isOpen={isChatOpen} onToggle={() => setIsChatOpen(!isChatOpen)} />
                </Layout>
              </ProtectedRoute>
            } />
            
            <Route path="/flow-cell-design" element={
              <ProtectedRoute>
                <Layout>
                  <FlowCellDesign />
                  {!isChatOpen && <ChatBotFloat onClick={() => setIsChatOpen(true)} />}
                  <ChatBot isOpen={isChatOpen} onToggle={() => setIsChatOpen(!isChatOpen)} />
                </Layout>
              </ProtectedRoute>
            } />
            
            <Route path="/sequencing" element={
              <ProtectedRoute>
                <Layout>
                  <Sequencing />
                  {!isChatOpen && <ChatBotFloat onClick={() => setIsChatOpen(true)} />}
                  <ChatBot isOpen={isChatOpen} onToggle={() => setIsChatOpen(!isChatOpen)} />
                </Layout>
              </ProtectedRoute>
            } />
            
            <Route path="/spreadsheets" element={
              <ProtectedRoute>
                <Layout>
                  <Spreadsheets />
                  {!isChatOpen && <ChatBotFloat onClick={() => setIsChatOpen(true)} />}
                  <ChatBot isOpen={isChatOpen} onToggle={() => setIsChatOpen(!isChatOpen)} />
                </Layout>
              </ProtectedRoute>
            } />
            
            <Route path="/storage" element={
              <ProtectedRoute>
                <Layout>
                  <Storage />
                  {!isChatOpen && <ChatBotFloat onClick={() => setIsChatOpen(true)} />}
                  <ChatBot isOpen={isChatOpen} onToggle={() => setIsChatOpen(!isChatOpen)} />
                </Layout>
              </ProtectedRoute>
            } />
            
            <Route path="/reports" element={
              <ProtectedRoute>
                <Layout>
                  <Reports />
                  {!isChatOpen && <ChatBotFloat onClick={() => setIsChatOpen(true)} />}
                  <ChatBot isOpen={isChatOpen} onToggle={() => setIsChatOpen(!isChatOpen)} />
                </Layout>
              </ProtectedRoute>
            } />
            
            <Route path="/profile" element={
              <ProtectedRoute>
                <Layout>
                  <Profile />
                  {!isChatOpen && <ChatBotFloat onClick={() => setIsChatOpen(true)} />}
                  <ChatBot isOpen={isChatOpen} onToggle={() => setIsChatOpen(!isChatOpen)} />
                </Layout>
              </ProtectedRoute>
            } />
            
            <Route path="/users" element={
              <ProtectedRoute requireRoles="lab_administrator">
                <Layout>
                  <Users />
                  {!isChatOpen && <ChatBotFloat onClick={() => setIsChatOpen(true)} />}
                  <ChatBot isOpen={isChatOpen} onToggle={() => setIsChatOpen(!isChatOpen)} />
                </Layout>
              </ProtectedRoute>
            } />
            
            <Route path="/services" element={
              <ProtectedRoute>
                <Layout>
                  <ServicesStatus />
                  {!isChatOpen && <ChatBotFloat onClick={() => setIsChatOpen(true)} />}
                  <ChatBot isOpen={isChatOpen} onToggle={() => setIsChatOpen(!isChatOpen)} />
                </Layout>
              </ProtectedRoute>
            } />
            
            <Route path="/notifications" element={
              <ProtectedRoute>
                <Layout>
                  <NotificationCenter />
                  {!isChatOpen && <ChatBotFloat onClick={() => setIsChatOpen(true)} />}
                  <ChatBot isOpen={isChatOpen} onToggle={() => setIsChatOpen(!isChatOpen)} />
                </Layout>
              </ProtectedRoute>
            } />
            
            <Route path="/events" element={
              <ProtectedRoute>
                <Layout>
                  <EventMonitorDashboard />
                  {!isChatOpen && <ChatBotFloat onClick={() => setIsChatOpen(true)} />}
                  <ChatBot isOpen={isChatOpen} onToggle={() => setIsChatOpen(!isChatOpen)} />
                </Layout>
              </ProtectedRoute>
            } />
            
            <Route path="/qaqc" element={
              <ProtectedRoute>
                <Layout>
                  <QAQCWorkflowInterface />
                  {!isChatOpen && <ChatBotFloat onClick={() => setIsChatOpen(true)} />}
                  <ChatBot isOpen={isChatOpen} onToggle={() => setIsChatOpen(!isChatOpen)} />
                </Layout>
              </ProtectedRoute>
            } />
            
            {/* Admin redirect route */}
            <Route path="/admin/users" element={<Navigate to="/users" replace />} />
            
            {/* Redirect root to dashboard */}
            <Route path="/" element={<Navigate to="/dashboard" replace />} />
            
            {/* Catch all route - redirect to dashboard */}
            <Route path="*" element={<Navigate to="/dashboard" replace />} />
          </Routes>
        </Router>
      </AuthProvider>
    </QueryClientProvider>
  );
}
