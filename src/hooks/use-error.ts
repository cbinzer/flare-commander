import { useAuth } from '@/features/authentication/hooks/use-auth.ts';
import { toast } from 'sonner';

export function useError() {
  const { resetCredentials } = useAuth();

  const handleError = (error: Error, title = 'An unknown error occurred') => {
    console.error(error);
    if ('kind' in error && error.kind === 'Authentication') {
      resetCredentials();
      return;
    }

    toast.error(title, {
      position: 'top-center',
      description: error.message,
    });
  };

  return {
    handleError,
  };
}
