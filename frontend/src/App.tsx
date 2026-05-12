import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { ServerProvider } from './contexts/ServerContext';
import { ToastProvider } from './components/Toast';
import { ErrorBoundary } from './components/ErrorBoundary';
import { Layout } from './components/Layout';
import { Dashboard } from './pages/Dashboard';
import { Terminal } from './pages/Terminal';
import { Files } from './pages/Files';

function App() {
  return (
    <ErrorBoundary>
      <ToastProvider>
        <ServerProvider>
          <Router>
            <Layout>
              <Routes>
                <Route path="/" element={<Dashboard />} />
                <Route path="/terminal" element={<Terminal />} />
                <Route path="/files" element={<Files />} />
              </Routes>
            </Layout>
          </Router>
        </ServerProvider>
      </ToastProvider>
    </ErrorBoundary>
  );
}

export default App;
