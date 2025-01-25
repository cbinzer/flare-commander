import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { ComponentPropsWithoutRef, FormEvent } from 'react';
import { useAuth } from '@/authentication/use-auth.ts';
import {
  AuthenticationError,
  DisabledTokenError,
  ExpiredTokenError,
  InvalidAccountIdError,
  InvalidTokenError,
} from '@/authentication/auth-errors.ts';

export function LoginForm({
  className,
  ...props
}: ComponentPropsWithoutRef<'div'>) {
  const { login } = useAuth();

  const doLogin = async (event: FormEvent) => {
    event.preventDefault();

    try {
      await login('fake-account-id', 'fake-token');
    } catch (error) {
      const authError = error as AuthenticationError;
      console.error(authError.message);
      if (authError instanceof InvalidTokenError) {
        return;
      } else if (authError instanceof ExpiredTokenError) {
        return;
      } else if (authError instanceof DisabledTokenError) {
        return;
      } else if (authError instanceof InvalidAccountIdError) {
        return;
      } else {
        return;
      }
    }
  };

  return (
    <div className={cn('flex flex-col gap-6', className)} {...props}>
      <Card>
        <CardHeader>
          <CardTitle className="text-2xl">Login</CardTitle>
          <CardDescription>
            Enter your Account ID below to login
          </CardDescription>
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
                <Input id="account-id" type="text" required />
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
                <Input id="api-token" type="password" required />
              </div>
              <Button type="submit" className="w-full">
                Login
              </Button>
            </div>
            <div className="mt-4 text-center text-sm">
              Don&apos;t have an account?{' '}
              <a
                href="https://dash.cloudflare.com/sign-up"
                className="underline underline-offset-4"
                target="_blank"
              >
                Sign up
              </a>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
