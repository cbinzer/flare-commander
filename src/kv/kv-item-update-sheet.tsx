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
import { FunctionComponent, ReactNode, useEffect, useRef, useState } from 'react';
import { useKvItem } from './kv-hooks';
import { KvItem, KvMetadata } from './kv-models';

export interface KvItemSheetProps {
  namespaceId: string;
  itemKey: string;
  itemMetadata?: KvMetadata;
  mode?: KvItemWriteMode;
  onChange?: (item: KvItem) => void;
  children?: ReactNode;
}

export enum KvItemWriteMode {
  CREATE = 'CREATE',
  UPDATE = 'UPDATE',
}

const KvItemUpdateSheet: FunctionComponent<KvItemSheetProps> = ({
  namespaceId,
  itemKey,
  itemMetadata,
  mode = KvItemWriteMode.UPDATE,
  children,
  onChange = () => {},
}) => {
  const { kvItem, loadKvItem, writeKvItem, resetKvItem, isLoading, isWriting } = useKvItem();
  const [sheetContainer, setSheetContainer] = useState<HTMLElement | null>(null);
  const [isOpen, setIsOpen] = useState(false);

  const loadKvItemOnOpenChange = (open: boolean) => {
    setIsOpen(open);
    if (!open) {
      return;
    }

    if (mode === KvItemWriteMode.UPDATE) {
      loadKvItem(namespaceId, itemKey).then(() =>
        setSheetContainer(document.querySelector('[role="dialog"]') as HTMLElement),
      );
    } else {
      setTimeout(() => setSheetContainer(document.querySelector('[role="dialog"]') as HTMLElement), 100);
    }
  };

  const writeKvItemOnSave = async (item: KvItem) => {
    await writeKvItem({
      namespaceId,
      ...item,
    });
    setIsOpen(false);
  };

  useEffect(() => {
    if (kvItem) {
      onChange(kvItem);

      if (mode === KvItemWriteMode.CREATE) {
        resetKvItem();
      }
    }
  }, [kvItem]);

  return (
    <Sheet open={isOpen} onOpenChange={loadKvItemOnOpenChange}>
      <SheetTrigger asChild>{children}</SheetTrigger>

      <KvItemUpdateSheetContent
        item={kvItem}
        itemMetadata={itemMetadata}
        container={sheetContainer}
        mode={mode}
        isLoading={isLoading}
        isSaving={isWriting}
        onSaveClick={writeKvItemOnSave}
      />
    </Sheet>
  );
};

interface KvItemSheetContentProps {
  item: KvItem | null;
  itemMetadata?: KvMetadata;
  container?: HTMLElement | null;
  isSaving?: boolean;
  isLoading?: boolean;
  mode?: KvItemWriteMode;
  onSaveClick?: (item: KvItem) => void;
}

const KvItemUpdateSheetContent: FunctionComponent<KvItemSheetContentProps> = ({
  item,
  itemMetadata = null,
  container,
  isSaving = false,
  isLoading = false,
  mode = KvItemWriteMode.UPDATE,
  onSaveClick = () => {},
}) => {
  const isUpdateMode = mode === KvItemWriteMode.UPDATE;
  const title = isUpdateMode ? 'Edit KV Item' : 'Create KV Item';
  const description = isUpdateMode
    ? 'Edit value, metadata and expiration date of the KV item'
    : 'Set key, value, metadata and expiration date of the KV item';
  const valueInputRef = useRef<HTMLTextAreaElement>(null);
  const nameInputRef = useRef<HTMLInputElement>(null);

  const [key, setKey] = useState(item?.key);
  const [value, setValue] = useState(item?.value);
  const [metadata, setMetadata] = useState(stringifyJSON(itemMetadata));
  const [isMetadataValid, setIsMetadataValid] = useState(true);
  const [expiration, setExpiration] = useState(item?.expiration);
  const isSaveButtonDisabled = isLoading || isSaving || !key || !isMetadataValid;

  const validateAndSetMetadata = (value: string) => {
    setMetadata(value);
    setIsMetadataValid(validateMetadata(value));
  };

  const handleSaveClick = () => {
    try {
      const parsedMetadata = parseJSON(metadata);
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

  useEffect(() => {
    if (mode === KvItemWriteMode.CREATE) {
      nameInputRef.current?.focus();
    } else {
      valueInputRef.current?.focus();
      // Set cursor at the end
      valueInputRef.current?.setSelectionRange(valueInputRef.current.value.length, valueInputRef.current.value.length);
    }
  }, [item, container]);

  useEffect(() => {
    setKey(item?.key);
    setValue(item?.value);
    setExpiration(item?.expiration);
  }, [item]);

  return (
    <SheetContent closeDisabled={isSaving} className="w-[500px] sm:max-w-[500px]">
      <SheetHeader>
        <SheetTitle>{title}</SheetTitle>
        <SheetDescription>{description}</SheetDescription>
      </SheetHeader>

      <div className="grid gap-4 py-4">
        <div className="grid grid-cols-12 items-center gap-4">
          <Label htmlFor="key" className="col-span-2 text-right">
            Key *
          </Label>
          {isLoading ? (
            <Skeleton className="w-full h-[36px] rounded-md col-span-10" />
          ) : (
            <Input
              id="key"
              value={key}
              className="col-span-10"
              disabled={isUpdateMode}
              ref={nameInputRef}
              onChange={(e) => setKey(e.target.value)}
            />
          )}
        </div>
        <div className="grid grid-cols-12 items-start gap-4">
          <Label htmlFor="value" className="col-span-2 text-right pt-2">
            Value
          </Label>
          {isLoading ? (
            <Skeleton id="value" className="w-full h-[200px] rounded-md col-span-10" />
          ) : (
            <Textarea
              id="value"
              value={value}
              onChange={(e) => setValue(e.target.value)}
              className="col-span-10 min-h-[200px]"
              ref={valueInputRef}
              disabled={isSaving}
            />
          )}
        </div>
        <div className="grid grid-cols-12 items-start gap-4">
          <Label htmlFor="metadata" className="col-span-2 text-right pt-2">
            Metadata
          </Label>
          {isLoading ? (
            <Skeleton id="metadata" className="w-full h-[200px] rounded-md col-span-10" />
          ) : (
            <Textarea
              id="metadata"
              value={metadata}
              onChange={(e) => validateAndSetMetadata(e.target.value)}
              className="col-span-10 min-h-[200px]"
              disabled={isSaving}
            />
          )}
        </div>
        <div className="grid grid-cols-12 items-center gap-4">
          <Label htmlFor="expiration" className="col-span-2 text-right">
            Expiration
          </Label>
          <div className="col-span-10 w-full">
            {isLoading ? (
              <Skeleton className="w-full h-[36px] rounded-md" />
            ) : (
              <DateTimePicker
                container={container}
                value={expiration}
                disabled={isSaving}
                onChange={(date) => setExpiration(date)}
              />
            )}
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

function stringifyJSON(value: KvMetadata): string {
  if (value === null) {
    return '';
  }

  try {
    return JSON.stringify(value);
  } catch (e) {
    console.error('Error stringifying JSON:', e);
    return '';
  }
}

function parseJSON(value: string): KvMetadata {
  if (value === '') {
    return null;
  }

  try {
    return JSON.parse(value);
  } catch (e) {
    console.error('Error parsing JSON:', e);
    return null;
  }
}

function validateMetadata(value: string): boolean {
  if (value === '') {
    return true;
  }

  try {
    JSON.parse(value);
    return true;
  } catch (e) {
    return false;
  }
}

export default KvItemUpdateSheet;
