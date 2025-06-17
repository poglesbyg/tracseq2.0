import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useState } from 'react';
import Layout from './components/Layout';
import { ChatBot } from './components/ChatBot';
import { ChatBotFloat } from './components/ChatBotFloat';
import { AuthProvider } from './auth/AuthContext';
import { ProtectedRoute } from './auth/ProtectedRoute';
import { LoginModal } from './auth/LoginModal';
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

const queryClient = new QueryClient();

export default function App() {
  const [isChatOpen, setIsChatOpen] = useState(false);

  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <Router>
          <ProtectedRoute fallback={<LoginModal isOpen={true} onClose={() => {}} />}>
            <Layout>
              <Routes>
                <Route path="/" element={<Dashboard />} />
                <Route path="/templates" element={<Templates />} />
                <Route path="/samples" element={<Samples />} />
                <Route path="/rag-submissions" element={<RagSubmissions />} />
                <Route path="/rag-samples" element={<RagSamples />} />
                <Route path="/sequencing" element={<Sequencing />} />
                <Route path="/spreadsheets" element={<Spreadsheets />} />
                <Route path="/storage" element={<Storage />} />
                <Route path="/reports" element={<Reports />} />
                <Route path="/profile" element={<Profile />} />
                <Route 
                  path="/users" 
                  element={
                    <ProtectedRoute requireRoles="lab_administrator">
                      <Users />
                    </ProtectedRoute>
                  } 
                />
              </Routes>
              
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
        </Router>
      </AuthProvider>
    </QueryClientProvider>
  );
}
