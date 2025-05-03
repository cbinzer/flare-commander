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
import { Textarea } from '@/components/ui/textarea';
import { FunctionComponent, ReactNode, useEffect, useRef, useState } from 'react';
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
  const { namespace, getNamespace, updateNamespace, isLoadingOne, isUpdating } = useNamespaces();
  const titleInputRef = useRef<HTMLTextAreaElement>(null);

  const [isOpen, setIsOpen] = useState(false);
  const [title, setTitle] = useState(namespace?.title);
  const [errors, setErrors] = useState<{ title?: Error }>({});

  const isSaveButtonDisabled = isLoadingOne || isUpdating || !title || title === namespace?.title || !!errors.title;

  const loadKvNamespaceOnOpenChange = (open: boolean) => {
    onOpenChange(open);
    setIsOpen(open);

    if (!open) {
      return;
    }

    getNamespace(namespaceId).then(() => {
      titleInputRef.current?.focus();
      // Set cursor at the end
      titleInputRef.current?.setSelectionRange(titleInputRef.current.value.length, titleInputRef.current.value.length);
    });
  };

  const handleSaveClick = async () => {
    try {
      const namespaceUpdateInput: KvNamespaceUpdateInput = {
        id: namespaceId,
        title: title ?? '',
      };
      await updateNamespace(namespaceUpdateInput);

      setIsOpen(false);
      onOpenChange(false);
    } catch (e) {
      setErrors((prevState) => ({ ...prevState, title: e as Error }));
    }
  };

  useEffect(() => {
    setTitle(namespace?.title);
  }, [namespace]);

  useEffect(() => {
    if (namespace) {
      onUpdate(namespace);
    }
  }, [namespace]);

  useEffect(() => loadKvNamespaceOnOpenChange(open), [open]);

  return (
    <Sheet open={isOpen} onOpenChange={loadKvNamespaceOnOpenChange}>
      <SheetTrigger asChild>{children}</SheetTrigger>

      <SheetContent closeDisabled={isUpdating} className="w-[500px] sm:max-w-[500px]">
        <SheetHeader>
          <SheetTitle>Edit KV Namespace</SheetTitle>
          <SheetDescription>Edit the title</SheetDescription>
        </SheetHeader>

        <div className="grid gap-4 py-4">
          <div className="grid grid-cols-12 items-center gap-4">
            <Label htmlFor="id" className="col-span-2 text-right">
              Id *
            </Label>
            {isLoadingOne ? (
              <Skeleton id="id" className="w-full h-[36px] rounded-md col-span-10" />
            ) : (
              <Input id="id" value={namespace?.id} className="col-span-10" disabled={true} />
            )}
          </div>

          <div className="grid grid-cols-12 items-start gap-4">
            <Label htmlFor="title" className="col-span-2 text-right pt-2">
              Title *
            </Label>
            {isLoadingOne ? (
              <Skeleton id="title" className="w-full h-[200px] rounded-md col-span-10" />
            ) : (
              <Textarea
                id="title"
                value={title}
                className="col-span-10 min-h-[200px]"
                onChange={(e) => setTitle(e.target.value)}
                ref={titleInputRef}
                disabled={isUpdating}
              />
            )}
          </div>
        </div>

        <SheetFooter>
          <Button type="submit" disabled={isSaveButtonDisabled} onClick={handleSaveClick}>
            {isUpdating ? (
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
