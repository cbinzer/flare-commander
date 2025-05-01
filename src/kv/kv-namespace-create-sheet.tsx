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
  onCreate?: (namespace: KvNamespace) => Promise<void>;
  children?: ReactNode;
}

const KvNamespaceCreateSheet: FunctionComponent<KvNamespaceCreateSheetProps> = ({
  children,
  onCreate = () => Promise.resolve(),
}) => {
  const { createNamespace, namespace, error } = useNamespaces();

  const titleInputRef = useRef<HTMLInputElement>(null);

  const [isOpen, setIsOpen] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [title, setTitle] = useState<string | undefined>(undefined);
  const [errors, setErrors] = useState<{ title?: Error }>({});

  const isSaveButtonDisabled = isSaving || !title || !!errors.title;

  const setFocusOnOpenChange = (open: boolean) => {
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
    setIsSaving(true);

    try {
      await createNamespace(title ?? '');
    } catch (e) {
      setErrors((prev) => ({ ...prev, title: e as Error }));
    }
  };

  useEffect(() => {
    if (namespace) {
      onCreate(namespace)
        .then(() => setIsOpen(false))
        .finally(() => setIsSaving(false));
    }
  }, [namespace]);

  useEffect(() => {
    setTitle(undefined);
    setErrors({});
  }, [isOpen]);

  useEffect(() => {
    if (error) {
      setIsSaving(false);

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

  return (
    <Sheet open={isOpen} onOpenChange={setFocusOnOpenChange}>
      <SheetTrigger asChild>{children}</SheetTrigger>
      <SheetContent closeDisabled={isSaving} className="w-[500px] sm:max-w-[500px]">
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
                disabled={isSaving}
                onChange={handleTitleChange}
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
            {isSaving ? (
              <>
                <LoadingSpinner /> Saving...
              </>
            ) : (
              <>
                <Save /> Save
              </>
            )}
          </Button>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  );
};

export default KvNamespaceCreateSheet;
