import { useToast } from '@/hooks/use-toast.ts';
import { useAuth } from '@/features/authentication/hooks/use-auth.ts';

export function useError() {
  const { toast } = useToast();
  const { resetCredentials } = useAuth();

  const handleError = (error: Error) => {
    console.error(error);
    if ('kind' in error && error.kind === 'Authentication') {
      resetCredentials();
      return;
    }

    toast({
      variant: 'destructive',
      title: 'Something went wrong.',
      description: error.message,
    });
  };

  return {
    handleError,
  };
}
