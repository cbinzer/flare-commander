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
  verifyCredentials: (credentials: Credentials) => Promise<void>;
  resetCredentials: () => void;
}

export const AuthContext = createContext<AuthContextValue>({
  account: null,
  verifyCredentials: async () => {},
  resetCredentials: () => {},
});

interface AuthProviderProps {
  children: ReactNode;
}

const AuthProvider: FunctionComponent<AuthProviderProps> = ({ children }) => {
  const navigate = useNavigate();
  const [account, setAccount] =
    useLocalStorage<AccountWithCredentials>('account');

  const verifyCredentials = async (credentials: Credentials) => {
    const accountWithCredentials = await invokeVerifyCredentials(credentials);
    setAccount(accountWithCredentials);
    navigate('/');
  };
  const logout = async () => setAccount(null);

  const value: AuthContextValue = {
    account,
    verifyCredentials: verifyCredentials,
    resetCredentials: logout,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};

async function invokeVerifyCredentials(
  credentials: Credentials,
): Promise<AccountWithCredentials> {
  try {
    return await invoke<AccountWithCredentials>('verify_credentials', {
      credentials,
    });
  } catch (e) {
    const error = e as AuthenticationError;
    throw new AuthenticationError(error.message, error.kind);
  }
}

export default AuthProvider;
