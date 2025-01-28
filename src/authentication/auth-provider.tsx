import { createContext, FunctionComponent, ReactNode } from 'react';
import { useNavigate } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import {
  AccountWithToken,
  AuthenticationError,
} from '@/authentication/auth-models.ts';
import { useLocalStorage } from '@/common/common-hooks.ts';

interface AuthContextValue {
  account: AccountWithToken | null;
  login: (accountId: string, token: string) => Promise<void>;
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
  const [account, setAccount] = useLocalStorage<AccountWithToken>('account');

  const login = async (accountId: string, token: string) => {
    const accountWithToken = await authenticate(accountId, token);
    setAccount(accountWithToken);
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
  accountId: string,
  token: string,
): Promise<AccountWithToken> {
  try {
    return await invoke<AccountWithToken>('login', { accountId, token });
  } catch (e) {
    const error = e as AuthenticationError;
    throw new AuthenticationError(error.message, error.kind);
  }
}

export default AuthProvider;
