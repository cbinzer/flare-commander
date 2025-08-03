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
import { Skeleton } from '@/components/ui/skeleton.tsx';
import { FunctionComponent, KeyboardEvent, ReactNode, useEffect, useRef, useState } from 'react';
import { KvError, KvNamespace, KvNamespaceUpdateInput } from '../kv-models.ts';
import { Loader2Icon, Save } from 'lucide-react';
import { cn } from '@/lib/utils.ts';
import { useKvNamespaces } from '@/features/kv/hooks/use-kv-namespaces.ts';
import { useError } from '@/hooks/use-error.tsx';
import { ScrollArea } from '@/components/ui/scroll-area.tsx';

export interface KvNamespaceUpdateSheetProps {
  namespaceId: string;
  open?: boolean;
  onUpdate?: (item: KvNamespace) => Promise<void>;
  onOpenChange?: (open: boolean) => void;
  children?: ReactNode;
}

const KvNamespaceUpdateSheet: FunctionComponent<KvNamespaceUpdateSheetProps> = ({
  namespaceId,
  open = false,
  children,
  onUpdate = () => {},
  onOpenChange = () => {},
}) => {
  const { namespace, getNamespace, updateNamespace, isLoadingOne } = useKvNamespaces();
  const { handleError } = useError();
  const titleInputRef = useRef<HTMLInputElement>(null);

  const [isSaving, setIsSaving] = useState(false);
  const [isOpen, setIsOpen] = useState(false);
  const [title, setTitle] = useState(namespace?.title ?? '');
  const [errors, setErrors] = useState<{ title?: Error }>({});

  const isSaveButtonDisabled = isLoadingOne || isSaving || !title || title === namespace?.title || !!errors.title;

  const loadKvNamespaceOnOpenChange = (open: boolean) => {
    onOpenChange(open);
    setIsOpen(open);

    if (!open) {
      return;
    }

    setErrors({});

    getNamespace(namespaceId)
      .then(() => {
        setTimeout(() => {
          titleInputRef.current?.focus();
          titleInputRef.current?.setSelectionRange(0, titleInputRef.current.value.length);
        }, 100);
      })
      .catch(handleError);
  };

  const handleSaveClick = async () => {
    setIsSaving(true);

    try {
      const namespaceUpdateInput: Omit<KvNamespaceUpdateInput, 'account_id'> = {
        namespace_id: namespaceId,
        title: title ?? '',
      };
      await updateNamespace(namespaceUpdateInput);
      await onUpdate({ ...(namespace as KvNamespace), title: title ?? '' });

      setIsOpen(false);
      onOpenChange(false);
    } catch (e) {
      const error = e as KvError;
      if (error.kind === 'NamespaceAlreadyExists') {
        setErrors((prevState) => ({ ...prevState, title: error }));
      } else {
        handleError(error);
      }
    } finally {
      setIsSaving(false);
    }
  };

  const changeTitle = (newTitle: string) => {
    setErrors((prev) => ({ ...prev, title: undefined }));
    setTitle(newTitle);
  };

  const saveOnEnter = async (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && !isSaveButtonDisabled && title) {
      await handleSaveClick();
    }
  };

  useEffect(() => {
    setTitle(namespace?.title ?? '');
  }, [namespace]);

  useEffect(() => loadKvNamespaceOnOpenChange(open), [open]);

  return (
    <Sheet open={isOpen} onOpenChange={loadKvNamespaceOnOpenChange}>
      <SheetTrigger asChild>{children}</SheetTrigger>

      <SheetContent closeDisabled={isSaving} className="grid grid-rows-[auto_1fr_auto] w-[550px] sm:max-w-[550px]">
        <SheetHeader>
          <SheetTitle>Edit KV Namespace</SheetTitle>
          <SheetDescription>Edit the title</SheetDescription>
        </SheetHeader>

        <ScrollArea className="min-h-0">
          <div className="grid gap-4 p-4">
            <div className="grid grid-cols-[100px_1fr] items-center gap-4">
              <Label htmlFor="id" className="text-right">
                Id *
              </Label>
              {isLoadingOne ? (
                <Skeleton id="id" className="w-full h-[36px] rounded-md" />
              ) : (
                <Input id="id" value={namespace?.id} disabled={true} />
              )}
            </div>

            <div className="grid grid-cols-[100px_1fr] items-center gap-4">
              <Label htmlFor="title" className="text-right">
                Title *
              </Label>
              {isLoadingOne ? (
                <Skeleton id="title" className="w-full h-[36px] rounded-md" />
              ) : (
                <div className="space-y-2">
                  <Input
                    id="title"
                    value={title}
                    disabled={isSaving}
                    onKeyDown={saveOnEnter}
                    onChange={(e) => changeTitle(e.target.value)}
                    ref={titleInputRef}
                  />
                  {errors.title && (
                    <p className={cn('text-[0.8rem] font-medium text-destructive')}>
                      A namespace with this title already exists
                    </p>
                  )}
                </div>
              )}
            </div>

            <div className="grid grid-cols-[100px_1fr] items-center gap-4">
              <Label htmlFor="beta" className="text-right">
                Beta
              </Label>
              {isLoadingOne ? (
                <Skeleton id="beta" className="w-full h-[36px] rounded-md" />
              ) : (
                <Input id="beta" value={namespace?.beta?.toString()} disabled={true} />
              )}
            </div>

            <div className="grid grid-cols-[100px_1fr] items-center gap-4">
              <Label htmlFor="supportsUrlEncoding" className=" text-right">
                URL Encoding
              </Label>
              {isLoadingOne ? (
                <Skeleton id="supportsUrlEncoding" className="w-full h-[36px] rounded-md" />
              ) : (
                <Input id="supportsUrlEncoding" value={namespace?.supports_url_encoding?.toString()} disabled={true} />
              )}
            </div>
          </div>
        </ScrollArea>

        <SheetFooter>
          <Button type="submit" disabled={isSaveButtonDisabled} onClick={handleSaveClick} className="w-fit self-end">
            {isSaving ? (
              <>
                <Loader2Icon className="animate-spin" /> Saving...
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

export default KvNamespaceUpdateSheet;
