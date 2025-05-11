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
import { Skeleton } from '@/components/ui/skeleton';
import { FunctionComponent, KeyboardEvent, ReactNode, useEffect, useRef, useState } from 'react';
import { useNamespaces } from './kv-hooks';
import { KvNamespace, KvNamespaceUpdateInput } from './kv-models';
import { Save } from 'lucide-react';

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
  const { namespace, getNamespace, updateNamespace, isLoadingOne } = useNamespaces();
  const titleInputRef = useRef<HTMLInputElement>(null);

  const [isSaving, setIsSaving] = useState(false);
  const [isOpen, setIsOpen] = useState(false);
  const [title, setTitle] = useState(namespace?.title);
  const [errors, setErrors] = useState<{ title?: Error }>({});

  const isSaveButtonDisabled = isLoadingOne || isSaving || !title || title === namespace?.title || !!errors.title;

  const loadKvNamespaceOnOpenChange = (open: boolean) => {
    onOpenChange(open);
    setIsOpen(open);

    if (!open) {
      return;
    }

    getNamespace(namespaceId).then(() => {
      setTimeout(() => {
        titleInputRef.current?.focus();
        titleInputRef.current?.setSelectionRange(0, titleInputRef.current.value.length);
      }, 100);
    });
  };

  const handleSaveClick = async () => {
    setIsSaving(true);

    try {
      const namespaceUpdateInput: KvNamespaceUpdateInput = {
        id: namespaceId,
        title: title ?? '',
      };
      await updateNamespace(namespaceUpdateInput);
      await onUpdate({ ...(namespace as KvNamespace), title: title ?? '' });

      setIsOpen(false);
      onOpenChange(false);
    } catch (e) {
      setErrors((prevState) => ({ ...prevState, title: e as Error }));
    } finally {
      setIsSaving(false);
    }
  };

  const saveOnEnter = async (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && !isSaveButtonDisabled && title) {
      await handleSaveClick();
    }
  };

  useEffect(() => {
    setTitle(namespace?.title);
  }, [namespace]);

  useEffect(() => loadKvNamespaceOnOpenChange(open), [open]);

  return (
    <Sheet open={isOpen} onOpenChange={loadKvNamespaceOnOpenChange}>
      <SheetTrigger asChild>{children}</SheetTrigger>

      <SheetContent closeDisabled={isSaving} className="w-[500px] sm:max-w-[500px]">
        <SheetHeader>
          <SheetTitle>Edit KV Namespace</SheetTitle>
          <SheetDescription>Edit the title</SheetDescription>
        </SheetHeader>

        <div className="grid gap-4 py-4">
          <div className="grid grid-cols-[95px_1fr] items-center gap-4">
            <Label htmlFor="id" className="text-right">
              Id *
            </Label>
            {isLoadingOne ? (
              <Skeleton id="id" className="w-full h-[36px] rounded-md" />
            ) : (
              <Input id="id" value={namespace?.id} disabled={true} />
            )}
          </div>

          <div className="grid grid-cols-[95px_1fr] items-center gap-4">
            <Label htmlFor="title" className="text-right">
              Title *
            </Label>
            {isLoadingOne ? (
              <Skeleton id="title" className="w-full h-[36px] rounded-md" />
            ) : (
              <Input
                id="title"
                value={title}
                disabled={isSaving}
                onKeyDown={saveOnEnter}
                onChange={(e) => setTitle(e.target.value)}
                ref={titleInputRef}
              />
            )}
          </div>

          <div className="grid grid-cols-[95px_1fr] items-center gap-4">
            <Label htmlFor="beta" className="text-right">
              Beta
            </Label>
            {isLoadingOne ? (
              <Skeleton id="beta" className="w-full h-[36px] rounded-md" />
            ) : (
              <Input id="beta" value={namespace?.beta?.toString()} disabled={true} />
            )}
          </div>

          <div className="grid grid-cols-[95px_1fr] items-center gap-4">
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

export default KvNamespaceUpdateSheet;
