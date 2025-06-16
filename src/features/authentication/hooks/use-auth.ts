import { useContext } from 'react';
import { AuthContext } from '@/features/authentication/components/auth-provider.tsx';

export const useAuth = () => {
  return useContext(AuthContext);
};
