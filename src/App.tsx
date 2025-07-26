import AuthProvider from '@/features/authentication/components/auth-provider.tsx';
import LoginPage from '@/pages/login-page.tsx';
import ProtectedPage from '@/features/authentication/components/protected-page.tsx';
import { Toaster } from '@/components/ui/sonner.tsx';
import KvNamespaceDetails from '@/features/kv/components/kv-namespace-details.tsx';
import MainPage from '@/pages/main-page.tsx';
import { BrowserRouter, Route, Routes } from 'react-router';
import './App.css';

function App() {
  return (
    <BrowserRouter>
      <AuthProvider>
        <Toaster />

        <Routes>
          <Route path="/login" element={<LoginPage />} />
          <Route
            path="/"
            element={
              <ProtectedPage>
                <MainPage />
              </ProtectedPage>
            }
          >
            <Route path="namespaces/:id" element={<KvNamespaceDetails />} />
          </Route>
        </Routes>
      </AuthProvider>
    </BrowserRouter>
  );
}

export default App;
