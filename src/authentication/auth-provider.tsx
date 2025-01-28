import { createContext, FunctionComponent, ReactNode, useState } from 'react';
import { useNavigate } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import {
  AccountWithToken,
  AuthenticationError,
} from '@/authentication/auth-models.ts';

interface AuthContextValue {
  token: string | undefined;
  login: (accountId: string, token: string) => Promise<void>;
  logout: () => void;
}

export const AuthContext = createContext<AuthContextValue>({
  token: undefined,
  login: async () => {},
  logout: () => {},
});

interface AuthProviderProps {
  children: ReactNode;
}

const AuthProvider: FunctionComponent<AuthProviderProps> = ({ children }) => {
  const navigate = useNavigate();
  const [token, setToken] = useState<string>();

  const login = async (accountId: string, token: string) => {
    const account = await authenticate(accountId, token);
    console.log(account);
    setToken('fake-token');
    navigate('/');
  };
  const logout = async () => setToken(undefined);

  const value: AuthContextValue = {
    token,
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
