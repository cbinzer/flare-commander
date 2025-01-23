import { createContext, FunctionComponent, ReactNode, useState } from 'react';
import { useNavigate } from 'react-router';

export const AuthContext = createContext<{
  token: string | undefined;
  login: () => Promise<void>;
  logout: () => void;
}>({
  token: undefined,
  login: async () => {},
  logout: () => {},
});

type AuthProviderProps = {
  children: ReactNode;
};

const AuthProvider: FunctionComponent<AuthProviderProps> = ({ children }) => {
  const navigate = useNavigate();
  const [token, setToken] = useState<string>();

  const login = async () => {
    // const token = await fakeAuth();
    setToken('fake-token');
    navigate('/');
  };
  const logout = async () => setToken(undefined);

  const value = {
    token,
    login,
    logout,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};

export default AuthProvider;
