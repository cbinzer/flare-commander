import AuthProvider from '@/authentication/auth-provider.tsx';
import LoginPage from '@/authentication/login-page.tsx';
import ProtectedPage from '@/authentication/protected-page.tsx';
import { Toaster } from '@/components/ui/toaster.tsx';
import KvNamespaceDetails from '@/kv/kv-namespace-details.tsx';
import MainPage from '@/main/main-page.tsx';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { BrowserRouter, Route, Routes } from 'react-router';
import './App.css';

const queryClient = new QueryClient();

function App() {
  return (
    <QueryClientProvider client={queryClient}>
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
    </QueryClientProvider>
  );
}

export default App;
