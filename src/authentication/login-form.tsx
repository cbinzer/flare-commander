import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { ComponentPropsWithoutRef, FormEvent, useState } from 'react';
import { useAuth } from '@/authentication/use-auth.ts';
import { LoadingSpinner } from '@/components/ui/loading-spinner.tsx';
import { AuthenticationError, CredentialsType, UserAuthTokenCredentials } from '@/authentication/auth-models.ts';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert.tsx';
import { AlertCircle, GalleryVerticalEnd } from 'lucide-react';

export function LoginForm({ className, ...props }: ComponentPropsWithoutRef<'div'>) {
  const { verifyCredentials } = useAuth();
  const [accountIdErrorMessage, setAccountIdErrorMessage] = useState<string | null>(null);
  const [tokenErrorMessage, setTokenErrorMessage] = useState<string | null>(null);
  const [unknownErrorMessage, setUnknownErrorMessage] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const doLogin = async (event: FormEvent) => {
    setLoading(true);
    event.preventDefault();

    setAccountIdErrorMessage(null);
    setTokenErrorMessage(null);
    setUnknownErrorMessage(null);

    const formData = new FormData(event.target as HTMLFormElement);
    const { accountId, apiToken } = Object.fromEntries(formData.entries()) as {
      accountId: string;
      apiToken: string;
    };

    try {
      const credentials: UserAuthTokenCredentials = {
        type: CredentialsType.UserAuthToken,
        account_id: accountId,
        token: apiToken,
      };
      await verifyCredentials(credentials);
    } catch (error) {
      const authError = error as AuthenticationError;
      switch (authError.kind) {
        case 'InvalidAccountId':
          setAccountIdErrorMessage(authError.message);
          break;
        case 'Unknown':
          setUnknownErrorMessage(authError.message);
          console.error(authError);
          break;
        default:
          setTokenErrorMessage(authError.message);
          console.error(authError);
          break;
      }
    }

    setLoading(false);
  };

  return (
    <div className={cn('flex flex-col gap-6', className)} {...props}>
      <div className="flex w-full max-w-sm flex-col gap-6">
        <div className="flex items-center gap-2 self-center font-medium text-lg">
          <div className="flex h-6 w-6 items-center justify-center rounded-md bg-primary text-primary-foreground">
            <GalleryVerticalEnd className="size-4" />
          </div>
          FlareCommander
        </div>
      </div>

      <Card>
        <CardHeader>
          {unknownErrorMessage ? (
            <Alert variant="destructive">
              <AlertCircle className="h-4 w-4" />
              <AlertTitle>Error</AlertTitle>
              <AlertDescription>{unknownErrorMessage}</AlertDescription>
            </Alert>
          ) : null}

          <CardTitle className="text-2xl">Login</CardTitle>
          <CardDescription>Enter your Account ID below to login</CardDescription>
        </CardHeader>

        <CardContent>
          <form onSubmit={doLogin}>
            <div className="flex flex-col gap-6">
              <div className="grid gap-2">
                <div className="flex items-center">
                  <Label htmlFor="account-id">Account ID</Label>
                  <a
                    href="https://developers.cloudflare.com/fundamentals/setup/find-account-and-zone-ids/"
                    className="ml-auto inline-block text-sm underline-offset-4 hover:underline"
                    target="_blank"
                  >
                    Find your Account ID
                  </a>
                </div>
                <Input id="account-id" name="accountId" type="text" required={true} />
                <p className={cn('text-[0.8rem] font-medium text-destructive', accountIdErrorMessage ? '' : 'hidden')}>
                  {accountIdErrorMessage}
                </p>
              </div>

              <div className="grid gap-2">
                <div className="flex items-center">
                  <Label htmlFor="api-token">API Token</Label>
                  <a
                    href="https://developers.cloudflare.com/fundamentals/api/get-started/create-token/"
                    className="ml-auto inline-block text-sm underline-offset-4 hover:underline"
                    target="_blank"
                  >
                    Create an API token
                  </a>
                </div>
                <Input id="api-token" name="apiToken" type="password" required={true} />
                <p className={cn('text-[0.8rem] font-medium text-destructive', tokenErrorMessage ? '' : 'hidden')}>
                  {tokenErrorMessage}
                </p>
              </div>
              <Button type="submit" className="w-full" disabled={loading}>
                {loading ? (
                  <>
                    <LoadingSpinner /> Login...
                  </>
                ) : (
                  'Login'
                )}
              </Button>
            </div>
            <div className="mt-4 text-center text-sm">
              Don&apos;t have an account?{' '}
              <a href="https://dash.cloudflare.com/sign-up" className="underline underline-offset-4" target="_blank">
                Sign up
              </a>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
