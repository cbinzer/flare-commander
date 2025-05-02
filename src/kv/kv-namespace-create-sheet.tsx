import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { LoadingSpinner } from '@/components/ui/loading-spinner';
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetFooter,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from '@/components/ui/sheet';
import { ChangeEvent, FunctionComponent, ReactNode, useEffect, useRef, useState } from 'react';
import { useNamespaces } from './kv-hooks';
import { KvNamespace } from './kv-models';
import { Save } from 'lucide-react';
import { cn } from '@/lib/utils.ts';

export interface KvNamespaceCreateSheetProps {
  open?: boolean;
  onCreate?: (namespace: KvNamespace) => Promise<void>;
  children?: ReactNode;
  onOpenChange?: (open: boolean) => void;
}

const KvNamespaceCreateSheet: FunctionComponent<KvNamespaceCreateSheetProps> = ({
  open = false,
  children,
  onCreate = () => Promise.resolve(),
  onOpenChange = () => {},
}) => {
  const { createNamespace, namespace, error } = useNamespaces();

  const titleInputRef = useRef<HTMLInputElement>(null);

  const [isOpen, setIsOpen] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const [title, setTitle] = useState<string | undefined>(undefined);
  const [errors, setErrors] = useState<{ title?: Error }>({});

  const isSaveButtonDisabled = isCreating || !title || !!errors.title;

  const setFocusOnOpenChange = (open: boolean) => {
    onOpenChange(open);

    setIsOpen(open);
    if (!open) {
      return;
    }

    setTimeout(() => titleInputRef.current?.focus(), 100);
  };

  const handleTitleChange = (e: ChangeEvent<HTMLInputElement>) => {
    setTitle(e.target.value);
    setErrors((prev) => ({ ...prev, title: undefined }));
  };

  const handleSaveClick = async () => {
    setIsCreating(true);

    try {
      await createNamespace(title ?? '');
    } catch (e) {
      setErrors((prev) => ({ ...prev, title: e as Error }));
    }
  };

  const createOnEnter = async (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && !isSaveButtonDisabled && title) {
      await handleSaveClick();
    }
  };

  useEffect(() => {
    if (namespace) {
      onCreate(namespace)
        .then(() => {
          setIsOpen(false);
          onOpenChange(false);
        })
        .finally(() => setIsCreating(false));
    }
  }, [namespace]);

  useEffect(() => {
    setTitle(undefined);
    setErrors({});
  }, [isOpen]);

  useEffect(() => {
    if (error) {
      setIsCreating(false);

      if (error.kind === 'NamespaceAlreadyExists') {
        setErrors((prev) => ({
          ...prev,
          title: error,
        }));
      } else {
        console.error('Error creating KV namespace:', error);
      }
    }
  }, [error]);

  useEffect(() => setFocusOnOpenChange(open), [open]);

  return (
    <Sheet open={isOpen} onOpenChange={setFocusOnOpenChange}>
      <SheetTrigger asChild>{children}</SheetTrigger>
      <SheetContent closeDisabled={isCreating} className="w-[500px] sm:max-w-[500px]">
        <SheetHeader>
          <SheetTitle>Create KV Namespace</SheetTitle>
          <SheetDescription>Set title to create a namespace</SheetDescription>
        </SheetHeader>

        <div className="grid gap-4 py-4">
          <div className="grid grid-cols-12 items-start gap-4">
            <Label htmlFor="title" className="col-span-2 text-right pt-3">
              Title *
            </Label>
            <div className="col-span-10 space-y-2">
              <Input
                id="title"
                value={title}
                ref={titleInputRef}
                disabled={isCreating}
                onChange={handleTitleChange}
                onKeyDown={createOnEnter}
                className={cn(errors.title && 'border-red-500 focus-visible:ring-red-500')}
              />
              {errors.title && (
                <p className={cn('text-[0.8rem] font-medium text-destructive')}>
                  A namespace with this title already exists
                </p>
              )}
            </div>
          </div>
        </div>

        <SheetFooter>
          <Button type="submit" disabled={isSaveButtonDisabled} onClick={handleSaveClick}>
            {isCreating ? (
              <>
                <LoadingSpinner /> Creating...
              </>
            ) : (
              <>
                <Save /> Create
              </>
            )}
          </Button>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  );
};

export default KvNamespaceCreateSheet;
