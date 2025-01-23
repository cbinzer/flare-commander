import { useContext } from 'react';
import { AuthContext } from '@/authentication/auth-provider.tsx';

export const useAuth = () => {
  return useContext(AuthContext);
};
