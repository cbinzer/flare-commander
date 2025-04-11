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
import { Textarea } from '@/components/ui/textarea';
import { FunctionComponent, ReactNode, useEffect, useRef, useState } from 'react';
import { useKvItem } from './kv-hooks';
import { KvItem } from './kv-models';
import { parseMetadataJSON, validateMetadata } from '@/kv/kv-utils.ts';

export interface KvItemCreateSheetProps {
  namespaceId: string;
  onCreate?: (item: KvItem) => Promise<void>;
  children?: ReactNode;
}

const KvItemCreateSheet: FunctionComponent<KvItemCreateSheetProps> = ({
  namespaceId,
  children,
  onCreate = () => Promise.resolve(),
}) => {
  const { kvItem, writeKvItem, isWriting } = useKvItem();
  const [sheetContainer, setSheetContainer] = useState<HTMLElement | null>(null);
  const [isOpen, setIsOpen] = useState(false);

  const setContainerOnOpenChange = (open: boolean) => {
    setIsOpen(open);
    if (!open) {
      return;
    }

    setTimeout(() => setSheetContainer(document.querySelector('[role="dialog"]') as HTMLElement), 100);
  };

  const writeKvItemOnSave = async (item: KvItem) => {
    await writeKvItem({
      namespaceId,
      ...item,
    });
  };

  useEffect(() => {
    if (kvItem) {
      onCreate(kvItem).then(() => setIsOpen(false));
    }
  }, [kvItem]);

  return (
    <Sheet open={isOpen} onOpenChange={setContainerOnOpenChange}>
      <SheetTrigger asChild>{children}</SheetTrigger>
      <KvItemCreateSheetContent
        open={isOpen}
        container={sheetContainer}
        isSaving={isWriting}
        onSaveClick={writeKvItemOnSave}
      />
    </Sheet>
  );
};

interface KvItemCreateSheetContentProps {
  open: boolean;
  container?: HTMLElement | null;
  isSaving?: boolean;
  onSaveClick?: (item: KvItem) => void;
}

const KvItemCreateSheetContent: FunctionComponent<KvItemCreateSheetContentProps> = ({
  open,
  container,
  isSaving = false,
  onSaveClick = () => {},
}) => {
  const valueInputRef = useRef<HTMLTextAreaElement>(null);
  const nameInputRef = useRef<HTMLInputElement>(null);

  const [key, setKey] = useState<string | undefined>(undefined);
  const [value, setValue] = useState<string | undefined>(undefined);
  const [metadata, setMetadata] = useState('');
  const [isMetadataValid, setIsMetadataValid] = useState(true);
  const [expiration, setExpiration] = useState<Date | undefined>(undefined);
  const isSaveButtonDisabled = isSaving || !key || !isMetadataValid;

  const validateAndSetMetadata = (value: string) => {
    setMetadata(value);
    setIsMetadataValid(validateMetadata(value));
  };

  const handleSaveClick = () => {
    try {
      const parsedMetadata = parseMetadataJSON(metadata);
      const item: KvItem = {
        key: key ?? '',
        value,
        expiration,
        metadata: parsedMetadata,
      };
      onSaveClick(item);
    } catch (e) {
      console.error('Error parsing metadata:', e);
      setIsMetadataValid(false);
    }
  };

  useEffect(() => nameInputRef.current?.focus(), [container]);
  useEffect(() => {
    setKey(undefined);
    setValue(undefined);
    setMetadata('');
    setExpiration(undefined);
  }, [open]);

  return (
    <SheetContent closeDisabled={isSaving} className="w-[500px] sm:max-w-[500px]">
      <SheetHeader>
        <SheetTitle>Create KV Item</SheetTitle>
        <SheetDescription>Set key, value, metadata and expiration date</SheetDescription>
      </SheetHeader>

      <div className="grid gap-4 py-4">
        <div className="grid grid-cols-12 items-center gap-4">
          <Label htmlFor="key" className="col-span-2 text-right">
            Key *
          </Label>
          <Input
            id="key"
            value={key}
            className="col-span-10"
            ref={nameInputRef}
            disabled={isSaving}
            onChange={(e) => setKey(e.target.value)}
          />
        </div>
        <div className="grid grid-cols-12 items-start gap-4">
          <Label htmlFor="value" className="col-span-2 text-right pt-2">
            Value
          </Label>
          <Textarea
            id="value"
            value={value}
            onChange={(e) => setValue(e.target.value)}
            className="col-span-10 min-h-[200px]"
            ref={valueInputRef}
            disabled={isSaving}
          />
        </div>
        <div className="grid grid-cols-12 items-start gap-4">
          <Label htmlFor="metadata" className="col-span-2 text-right pt-2">
            Metadata
          </Label>
          <Textarea
            id="metadata"
            value={metadata}
            onChange={(e) => validateAndSetMetadata(e.target.value)}
            className="col-span-10 min-h-[200px]"
            disabled={isSaving}
          />
        </div>
        <div className="grid grid-cols-12 items-center gap-4">
          <Label htmlFor="expiration" className="col-span-2 text-right">
            Expiration
          </Label>
          <div className="col-span-10 w-full">
            <DateTimePicker
              container={container}
              value={expiration}
              disabled={isSaving}
              onChange={(date) => setExpiration(date)}
            />
          </div>
        </div>
      </div>

      <SheetFooter>
        <Button type="submit" disabled={isSaveButtonDisabled} onClick={handleSaveClick}>
          {isSaving ? <LoadingSpinner /> : 'Save'}
        </Button>
      </SheetFooter>
    </SheetContent>
  );
};

export default KvItemCreateSheet;
