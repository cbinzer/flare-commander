import { FunctionComponent, ReactNode } from 'react';
import { Navigate } from 'react-router';
import { useAuth } from '@/authentication/use-auth.ts';

const ProtectedPage: FunctionComponent<ProtectedPageProps> = ({ children }) => {
  const { token } = useAuth();
  if (!token) {
    return <Navigate to="/login" replace={true} />;
  }

  return children;
};

type ProtectedPageProps = {
  children: ReactNode;
};

export default ProtectedPage;
