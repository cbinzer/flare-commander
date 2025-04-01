import { Button } from '@/components/ui/button';
import DateTimePicker from '@/components/ui/date-time-picker';
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
import { FunctionComponent, useEffect, useRef, useState } from 'react';
import { useKvItem } from './kv-hooks';
import { KvItem } from './kv-models';

export interface KvItemSheetProps {
  namespaceId: string;
  itemKey: string;
}

const KvItemSheet: FunctionComponent<KvItemSheetProps> = ({ namespaceId, itemKey }) => {
  const { kvItem, loadKvItem, writeKvItem, isWriting } = useKvItem();
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

  const writeKvItemOnSave = async (value: string | undefined, expiration: Date | undefined) => {
    await writeKvItem({
      namespaceId,
      key: itemKey,
      value,
      expiration,
    });
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
        onSaveClick={writeKvItemOnSave}
        isSaving={isWriting}
      />
    </Sheet>
  );
};

interface KvItemSheetContentProps {
  item: KvItem | null;
  container?: HTMLElement | null;
  isSaving?: boolean;
  onSaveClick?: (value: string | undefined, expiration: Date | undefined) => void;
}

const KvItemSheetContent: FunctionComponent<KvItemSheetContentProps> = ({
  item,
  container,
  isSaving = false,
  onSaveClick = () => {},
}) => {
  const valueInputRef = useRef<HTMLTextAreaElement>(null);
  const [value, setValue] = useState(item?.value);
  const [expiration, setExpiration] = useState(item?.expiration);

  useEffect(() => {
    const textarea = valueInputRef.current;
    if (textarea) {
      textarea.focus();
      // Set cursor at the end
      textarea.setSelectionRange(textarea.value.length, textarea.value.length);
    }
  }, [item, container]);

  useEffect(() => {
    setValue(item?.value);
    setExpiration(item?.expiration);
  }, [item]);

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
              value={value}
              onChange={(e) => setValue(e.target.value)}
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
              <DateTimePicker
                container={container}
                value={expiration}
                disabled={isSaving}
                onChange={(date) => setExpiration(date)}
              />
            ) : (
              <Skeleton className="w-full h-[36px] rounded-md" />
            )}
          </div>
        </div>
      </div>

      <SheetFooter>
        <Button type="submit" disabled={!item || isSaving} onClick={() => onSaveClick(value, expiration)}>
          {isSaving ? <LoadingSpinner /> : 'Save'}
        </Button>
      </SheetFooter>
    </SheetContent>
  );
};

export default KvItemSheet;
