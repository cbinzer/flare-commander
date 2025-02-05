import './App.css';
import MainPage from '@/main/main-page.tsx';
import { BrowserRouter, Route, Routes } from 'react-router';
import LoginPage from '@/authentication/login-page.tsx';
import ProtectedPage from '@/authentication/protected-page.tsx';
import AuthProvider from '@/authentication/auth-provider.tsx';
import { Toaster } from '@/components/ui/toaster.tsx';

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
            index={true}
          >
            {/*<Route index element={<RecentActivity />} />*/}
            {/*<Route path="project/:id" element={<Project />} />*/}
          </Route>
        </Routes>
      </AuthProvider>
    </BrowserRouter>
  );
}

export default App;
