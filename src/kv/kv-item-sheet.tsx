import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Sheet,
  SheetClose,
  SheetContent,
  SheetDescription,
  SheetFooter,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from '@/components/ui/sheet';
import { Skeleton } from '@/components/ui/skeleton';
import { Textarea } from '@/components/ui/textarea';
import { FunctionComponent, useEffect, useRef, useState } from 'react';
import { useKvItem } from './kv-hooks';
import { KvItem } from './kv-models';
import DateTimePicker from '@/components/ui/date-time-picker';
import { LoadingSpinner } from '@/components/ui/loading-spinner';

export interface KvItemSheetProps {
  namespaceId: string;
  itemKey: string;
}

const KvItemSheet: FunctionComponent<KvItemSheetProps> = ({ namespaceId, itemKey }) => {
  const { kvItem, loadKvItem, updateKvItem, isUpdating } = useKvItem();
  const [sheetContainer, setSheetContainer] = useState<HTMLElement | null>(null);
  const [isOpen, setIsOpen] = useState(false);

  const loadKvItemOnOpenChange = (open: boolean) => {
    setIsOpen(open);

    if (open) {
      loadKvItem(namespaceId, itemKey).then(() =>
        setSheetContainer(document.querySelector('[role="dialog"]') as HTMLElement),
      );
    }
  };

  const updateKvItemOnSave = async () => {
    await updateKvItem('', new Date());
    setIsOpen(false);
  };

  return (
    <Sheet open={isOpen} onOpenChange={loadKvItemOnOpenChange}>
      <SheetTrigger asChild>
        <Button variant="link" className="w-fit h-fit p-0 text-left text-foreground">
          {itemKey}
        </Button>
      </SheetTrigger>

      <KvItemSheetContent
        item={kvItem}
        container={sheetContainer}
        onSaveClick={updateKvItemOnSave}
        isSaving={isUpdating}
      />
    </Sheet>
  );
};

interface KvItemSheetContentProps {
  item: KvItem | null;
  container?: HTMLElement | null;
  isSaving?: boolean;
  onSaveClick?: () => void;
}

const KvItemSheetContent: FunctionComponent<KvItemSheetContentProps> = ({
  item,
  container,
  isSaving = false,
  onSaveClick = () => {},
}) => {
  const valueInputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    const textarea = valueInputRef.current;
    if (textarea) {
      textarea.focus();
      // Set cursor at the end
      textarea.setSelectionRange(textarea.value.length, textarea.value.length);
    }
  }, [item, container]);

  return (
    <SheetContent className="w-[500px] sm:max-w-[500px]">
      <SheetHeader>
        <SheetTitle>Edit KV Item</SheetTitle>
        <SheetDescription>Edit the value and expiration date of the KV item</SheetDescription>
      </SheetHeader>

      <div className="grid gap-4 py-4">
        <div className="grid grid-cols-12 items-center gap-4">
          <Label htmlFor="name" className="col-span-2 text-right">
            Name
          </Label>
          {item ? (
            <Input id="name" value={item?.key} className="col-span-10" disabled={true} />
          ) : (
            <Skeleton className="w-full h-[36px] rounded-md col-span-10" />
          )}
        </div>
        <div className="grid grid-cols-12 items-start gap-4">
          <Label htmlFor="value" className="col-span-2 text-right pt-2">
            Value
          </Label>
          {item ? (
            <Textarea
              id="value"
              value={item?.value}
              className="col-span-10 min-h-[200px]"
              ref={valueInputRef}
              disabled={isSaving}
            />
          ) : (
            <Skeleton id="value" className="w-full h-[200px] rounded-md col-span-10" />
          )}
        </div>
        <div className="grid grid-cols-12 items-center gap-4">
          <Label htmlFor="expiration" className="col-span-2 text-right">
            Expiration
          </Label>
          <div className="col-span-10 w-full">
            {item ? (
              <DateTimePicker container={container} value={item?.expiration} disabled={isSaving} />
            ) : (
              <Skeleton className="w-full h-[36px] rounded-md" />
            )}
          </div>
        </div>
      </div>

      <SheetFooter>
        <Button type="submit" disabled={!item || isSaving} onClick={onSaveClick}>
          {isSaving ? <LoadingSpinner /> : 'Save'}
        </Button>
      </SheetFooter>
    </SheetContent>
  );
};

export default KvItemSheet;
