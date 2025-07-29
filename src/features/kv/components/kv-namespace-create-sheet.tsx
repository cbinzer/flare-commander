import { Button } from '@/components/ui/button.tsx';
import { Input } from '@/components/ui/input.tsx';
import { Label } from '@/components/ui/label.tsx';
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetFooter,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from '@/components/ui/sheet.tsx';
import { ChangeEvent, FunctionComponent, KeyboardEvent, ReactNode, useEffect, useRef, useState } from 'react';

import { KvNamespace } from '../kv-models.ts';
import { Loader2Icon, PlusIcon } from 'lucide-react';
import { cn } from '@/lib/utils.ts';
import { useKvNamespaces } from '@/features/kv/hooks/use-kv-namespaces.ts';
import { useError } from '@/hooks/use-error.ts';
import { ScrollArea } from '@/components/ui/scroll-area.tsx';

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
  const { createNamespace, namespace, error } = useKvNamespaces();
  const { handleError } = useError();

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
    await createNamespace({ title: title ?? '' });
  };

  const createOnEnter = async (e: KeyboardEvent<HTMLInputElement>) => {
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
        handleError(error);
      }
    }
  }, [error]);

  useEffect(() => setFocusOnOpenChange(open), [open]);

  return (
    <Sheet open={isOpen} onOpenChange={setFocusOnOpenChange}>
      <SheetTrigger asChild>{children}</SheetTrigger>
      <SheetContent closeDisabled={isCreating} className="grid grid-rows-[auto_1fr_auto] w-[550px] sm:max-w-[550px]">
        <SheetHeader>
          <SheetTitle>Create KV Namespace</SheetTitle>
          <SheetDescription>Set title to create a namespace</SheetDescription>
        </SheetHeader>

        <ScrollArea className="min-h-0">
          <div className="grid gap-4 py-4 px-4">
            <div className="grid grid-cols-[100px_1fr] items-center gap-4">
              <Label htmlFor="title" className="text-right">
                Title *
              </Label>
              <div className="space-y-2">
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
        </ScrollArea>

        <SheetFooter>
          <Button type="submit" disabled={isSaveButtonDisabled} onClick={handleSaveClick} className="w-fit self-end">
            {isCreating ? (
              <>
                <Loader2Icon className="animate-spin" /> Creating...
              </>
            ) : (
              <>
                <PlusIcon /> Create
              </>
            )}
          </Button>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  );
};

export default KvNamespaceCreateSheet;
