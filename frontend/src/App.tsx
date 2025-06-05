import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import Templates from './pages/Templates';
import Samples from './pages/Samples';
import Sequencing from './pages/Sequencing';
import Storage from './pages/Storage';
import Reports from './pages/Reports';

const queryClient = new QueryClient();

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <Router>
        <Layout>
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/templates" element={<Templates />} />
            <Route path="/samples" element={<Samples />} />
            <Route path="/sequencing" element={<Sequencing />} />
            <Route path="/storage" element={<Storage />} />
            <Route path="/reports" element={<Reports />} />
          </Routes>
        </Layout>
      </Router>
    </QueryClientProvider>
  );
}
