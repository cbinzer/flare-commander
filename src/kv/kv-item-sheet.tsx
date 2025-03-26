import { Button } from '@/components/ui/button';
import DateTimePicker from '@/components/ui/date-time-picker';
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
import { Textarea } from '@/components/ui/textarea';
import { FunctionComponent, useEffect, useRef, useState } from 'react';
import { useKvItem } from './kv-hooks';
import { KvItem } from './kv-models';

export interface KvItemSheetProps {
  namespaceId: string;
  itemKey: string;
}

const KvItemSheet: FunctionComponent<KvItemSheetProps> = ({ namespaceId, itemKey }) => {
  const { kvItem, loadKvItem } = useKvItem();
  const [sheetContainer, setSheetContainer] = useState<HTMLElement | null>(null);

  const focusValueInputOnOpenChange = (open: boolean) => {
    if (open) {
      loadKvItem(namespaceId, itemKey).then(() =>
        setSheetContainer(document.querySelector('[role="dialog"]') as HTMLElement),
      );
    }
  };

  return (
    <Sheet onOpenChange={focusValueInputOnOpenChange}>
      <SheetTrigger asChild>
        <Button variant="link" className="w-fit px-0 text-left text-foreground">
          {itemKey}
        </Button>
      </SheetTrigger>

      <KvItemSheetContent item={kvItem} container={sheetContainer} />
    </Sheet>
  );
};

interface KvItemSheetContentProps {
  item: KvItem | null;
  container?: HTMLElement | null;
}

const KvItemSheetContent: FunctionComponent<KvItemSheetContentProps> = ({ item, container }) => {
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
          <Input id="name" value={item?.key} className="col-span-10" disabled={true} />
        </div>
        <div className="grid grid-cols-12 items-start gap-4">
          <Label htmlFor="value" className="col-span-2 text-right pt-2">
            Value
          </Label>
          <Textarea id="value" value={item?.value} className="col-span-10 min-h-[200px]" ref={valueInputRef} />
        </div>
        <div className="grid grid-cols-12 items-center gap-4">
          <Label htmlFor="expiration" className="col-span-2 text-right">
            Expiration
          </Label>
          <div className="col-span-10">
            <DateTimePicker container={container} />
          </div>
        </div>
      </div>

      <SheetFooter>
        <SheetClose asChild>
          <Button type="submit">Save</Button>
        </SheetClose>
      </SheetFooter>
    </SheetContent>
  );
};

export default KvItemSheet;
