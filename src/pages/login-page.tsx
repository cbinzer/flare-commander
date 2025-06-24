import { FunctionComponent } from 'react';
import { LoginForms } from '@/features/authentication/components/login-forms.tsx';

const LoginPage: FunctionComponent = () => {
  return (
    <div className="flex min-h-svh w-full items-center justify-center p-6 md:p-10 bg-muted">
      <div className="w-full max-w-sm">
        <LoginForms />
      </div>
    </div>
  );
};

export default LoginPage;
