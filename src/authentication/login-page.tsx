import { FunctionComponent } from 'react';
import { LoginForm } from '@/authentication/login-form.tsx';

const LoginPage: FunctionComponent = () => {
  return (
    <div className="flex min-h-svh w-full items-center justify-center p-6 md:p-10 bg-muted">
      <div className="w-full max-w-sm">
        <LoginForm />
      </div>
    </div>
  );
};

export default LoginPage;
