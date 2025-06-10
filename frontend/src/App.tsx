import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useState } from 'react';
import Layout from './components/Layout';
import { ChatBot } from './components/ChatBot';
import { ChatBotFloat } from './components/ChatBotFloat';
import Dashboard from './pages/Dashboard';
import Templates from './pages/Templates';
import Samples from './pages/Samples';
import RagSubmissions from './pages/RagSubmissions';
import Sequencing from './pages/Sequencing';
import Storage from './pages/Storage';
import Reports from './pages/Reports';

const queryClient = new QueryClient();

export default function App() {
  const [isChatOpen, setIsChatOpen] = useState(false);

  return (
    <QueryClientProvider client={queryClient}>
      <Router>
        <Layout>
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/templates" element={<Templates />} />
            <Route path="/samples" element={<Samples />} />
            <Route path="/rag-submissions" element={<RagSubmissions />} />
            <Route path="/sequencing" element={<Sequencing />} />
            <Route path="/storage" element={<Storage />} />
            <Route path="/reports" element={<Reports />} />
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
      </Router>
    </QueryClientProvider>
  );
}
