import { createContext, FunctionComponent, ReactNode, useState } from 'react';
import { useNavigate } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import {
  DisabledTokenError,
  ExpiredTokenError,
  InvalidAccountIdError,
  InvalidTokenError,
  UnknownAuthenticationError,
} from '@/authentication/auth-errors.ts';
import { APIError } from '@/common/common-errors.ts';

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
    // const token = await fakeAuth();
    await authenticate(accountId, token);
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

async function authenticate(accountId: string, token: string) {
  try {
    await invoke('login', { accountId, token });
  } catch (error) {
    const apiError = error as APIError;
    switch (apiError.kind) {
      case 'InvalidToken':
        throw new InvalidTokenError();
      case 'ExpiredToken':
        throw new ExpiredTokenError();
      case 'DisabledToken':
        throw new DisabledTokenError();
      case 'InvalidAccountId':
        throw new InvalidAccountIdError();
      case 'Unknown':
        throw new UnknownAuthenticationError(apiError.message);
    }
  }
}

export default AuthProvider;
