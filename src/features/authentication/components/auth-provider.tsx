import { createContext, FunctionComponent, ReactNode } from 'react';
import { useNavigate } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import { AccountWithCredentials, AuthenticationError, Credentials } from '@/features/authentication/auth-models.ts';
import { useLocalStorage } from '@/hooks/use-local-storage.ts';

interface AuthContextValue {
  account: AccountWithCredentials | null;
  verifyCredentials: (accountId: string, credentials: Credentials) => Promise<void>;
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
  const [account, setAccount] = useLocalStorage<AccountWithCredentials>('account');

  const verifyCredentials = async (accountId: string, credentials: Credentials) => {
    const accountWithCredentials = await invokeVerifyAccountAndCredentials(accountId, credentials);
    setAccount(accountWithCredentials);
    navigate('/');
  };
  const logout = async () => setAccount(null);

  const value: AuthContextValue = {
    account,
    verifyCredentials,
    resetCredentials: logout,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};

async function invokeVerifyAccountAndCredentials(
  accountId: string,
  credentials: Credentials,
): Promise<AccountWithCredentials> {
  try {
    return await invoke<AccountWithCredentials>('verify_account_and_credentials', {
      accountId,
      credentials,
    });
  } catch (e) {
    console.error('An error occurred on verifying account and credentials', e);
    const error = e as AuthenticationError;
    throw new AuthenticationError(error.message, error.kind);
  }
}

export default AuthProvider;
