import { cn } from '@/lib/utils.ts';
import { Button } from '@/components/ui/button.tsx';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card.tsx';
import { Input } from '@/components/ui/input.tsx';
import { Label } from '@/components/ui/label.tsx';
import { ComponentPropsWithoutRef, FormEvent, FunctionComponent, useState } from 'react';
import { useAuth } from '@/features/authentication/hooks/use-auth.ts';
import { AuthenticationError, Credentials, CredentialsType } from '@/features/authentication/auth-models.ts';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert.tsx';
import { AlertCircle, Loader2Icon } from 'lucide-react';
import logoUrl from '../../../assets/logo.svg';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs.tsx';

export function LoginForms({ className, ...props }: ComponentPropsWithoutRef<'div'>) {
  return (
    <div className={cn('flex flex-col gap-5', className)} {...props}>
      <div className="grid grid-cols-1 grid-rows-2 justify-items-center font-medium text-lg">
        <div className="h-11 w-11 justify-center">
          <img alt="logo" src={logoUrl} />
        </div>
        <div className="text-center">FlareCommander</div>
      </div>

      <Tabs defaultValue="account">
        <TabsList className="w-full">
          <TabsTrigger value="account">Account</TabsTrigger>
          <TabsTrigger value="user">User</TabsTrigger>
        </TabsList>

        <TabsContent value="account">
          <LoginForm credentialsType={CredentialsType.AccountAuthToken} />
        </TabsContent>
        <TabsContent value="user">
          <LoginForm credentialsType={CredentialsType.UserAuthToken} />
        </TabsContent>
      </Tabs>
    </div>
  );
}

interface LoginFormProps {
  credentialsType: CredentialsType;
}

const loginFormMetadata: Record<string, Record<CredentialsType, string>> = {
  cardDescription: {
    [CredentialsType.AccountAuthToken]: 'Enter your Account ID and Account API Token',
    [CredentialsType.UserAuthToken]: 'Enter your Account ID and User API Token',
    [CredentialsType.UserAuthKey]: '',
    [CredentialsType.Service]: '',
  },
  tokenLabel: {
    [CredentialsType.AccountAuthToken]: 'Account API Token',
    [CredentialsType.UserAuthToken]: 'User API Token',
    [CredentialsType.UserAuthKey]: '',
    [CredentialsType.Service]: '',
  },
  tokenLink: {
    [CredentialsType.AccountAuthToken]:
      'https://developers.cloudflare.com/fundamentals/api/get-started/account-owned-tokens/',
    [CredentialsType.UserAuthToken]: 'https://developers.cloudflare.com/fundamentals/api/get-started/create-token/',
    [CredentialsType.UserAuthKey]: '',
    [CredentialsType.Service]: '',
  },
};

const LoginForm: FunctionComponent<LoginFormProps> = ({ credentialsType }) => {
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
      const credentials = createCredentials(credentialsType, apiToken);
      await verifyCredentials(accountId, credentials);
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
        <CardDescription>{loginFormMetadata.cardDescription[credentialsType]}</CardDescription>
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
              <Input id="account-id" name="accountId" type="text" required={true} disabled={loading} />
              <p className={cn('text-[0.8rem] font-medium text-destructive', accountIdErrorMessage ? '' : 'hidden')}>
                {accountIdErrorMessage}
              </p>
            </div>

            <div className="grid gap-2">
              <div className="flex items-center">
                <Label htmlFor="api-token">{loginFormMetadata.tokenLabel[credentialsType]}</Label>
                <a
                  href={loginFormMetadata.tokenLink[credentialsType]}
                  className="ml-auto inline-block text-sm underline-offset-4 hover:underline"
                  target="_blank"
                >
                  Create an API token
                </a>
              </div>
              <Input id="api-token" name="apiToken" type="password" required={true} disabled={loading} />
              <p className={cn('text-[0.8rem] font-medium text-destructive', tokenErrorMessage ? '' : 'hidden')}>
                {tokenErrorMessage}
              </p>
            </div>
            <Button type="submit" className="w-full" disabled={loading}>
              {loading ? (
                <>
                  <Loader2Icon className="animate-spin" /> Login...
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
  );
};

function createCredentials(type: CredentialsType, token: string): Credentials {
  switch (type) {
    case CredentialsType.UserAuthToken:
      return { type, token };
    case CredentialsType.AccountAuthToken:
      return { type, token };
    default:
      throw new Error(`Unsupported credentials type: ${type}`);
  }
}
