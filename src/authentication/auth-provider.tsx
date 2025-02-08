import { createContext, FunctionComponent, ReactNode } from 'react';
import { useNavigate } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import {
  AccountWithCredentials,
  AuthenticationError,
  Credentials,
} from '@/authentication/auth-models.ts';
import { useLocalStorage } from '@/common/common-hooks.ts';

interface AuthContextValue {
  account: AccountWithCredentials | null;
  login: (credentials: Credentials) => Promise<void>;
  logout: () => void;
}

export const AuthContext = createContext<AuthContextValue>({
  account: null,
  login: async () => {},
  logout: () => {},
});

interface AuthProviderProps {
  children: ReactNode;
}

const AuthProvider: FunctionComponent<AuthProviderProps> = ({ children }) => {
  const navigate = useNavigate();
  const [account, setAccount] =
    useLocalStorage<AccountWithCredentials>('account');

  const login = async (credentials: Credentials) => {
    const accountWithCredentials = await authenticate(credentials);
    setAccount(accountWithCredentials);
    navigate('/');
  };
  const logout = async () => setAccount(null);

  const value: AuthContextValue = {
    account,
    login,
    logout,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};

export async function authenticate(
  credentials: Credentials,
): Promise<AccountWithCredentials> {
  try {
    return await invoke<AccountWithCredentials>('login', { credentials });
  } catch (e) {
    const error = e as AuthenticationError;
    throw new AuthenticationError(error.message, error.kind);
  }
}

export default AuthProvider;
