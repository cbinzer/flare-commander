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
import { ChangeEvent, FunctionComponent, ReactNode, useEffect, useRef, useState } from 'react';
import { useKvItem } from './kv-hooks';
import { KvItem } from './kv-models';
import { parseMetadataJSON, validateMetadata } from '@/kv/kv-utils.ts';
import { Save } from 'lucide-react';
import { cn } from '@/lib/utils.ts';

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
  const { kvItem, error, createKvItem } = useKvItem();

  const valueInputRef = useRef<HTMLTextAreaElement>(null);
  const nameInputRef = useRef<HTMLInputElement>(null);

  const [sheetContainer, setSheetContainer] = useState<HTMLElement | null>(null);
  const [isOpen, setIsOpen] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [key, setKey] = useState<string | undefined>(undefined);
  const [value, setValue] = useState<string | undefined>(undefined);
  const [metadata, setMetadata] = useState('');
  const [errors, setErrors] = useState<{ key?: Error; metadata?: Error }>({});
  const [expiration, setExpiration] = useState<Date | undefined>(undefined);

  const isSaveButtonDisabled = isSaving || !key || !!errors.key || !!errors.metadata;

  const setContainerOnOpenChange = (open: boolean) => {
    setIsOpen(open);
    if (!open) {
      return;
    }

    setTimeout(() => {
      setSheetContainer(document.querySelector('[role="dialog"]') as HTMLElement);
      nameInputRef.current?.focus();
    }, 100);
  };

  const handleKeyChange = (e: ChangeEvent<HTMLInputElement>) => {
    setKey(e.target.value);
    setErrors((prev) => ({ ...prev, key: undefined }));
  };

  const validateAndSetMetadata = (value: string) => {
    setMetadata(value);
    setErrors((prev) => ({ ...prev, metadata: validateMetadata(value) ? undefined : new Error('Invalid JSON') }));
  };

  const handleSaveClick = async () => {
    setIsSaving(true);

    try {
      const parsedMetadata = parseMetadataJSON(metadata);
      const item: KvItem = {
        key: key ?? '',
        value,
        expiration,
        metadata: parsedMetadata,
      };
      await createKvItem({
        namespaceId,
        ...item,
      });
    } catch (e) {
      console.error('Error parsing metadata:', e);
      setErrors((prev) => ({ ...prev, metadata: e as Error }));
    }
  };

  useEffect(() => {
    if (kvItem) {
      onCreate(kvItem)
        .then(() => setIsOpen(false))
        .finally(() => setIsSaving(false));
    }
  }, [kvItem]);

  useEffect(() => {
    setKey(undefined);
    setValue(undefined);
    setMetadata('');
    setExpiration(undefined);
    setErrors({});
  }, [isOpen]);

  useEffect(() => {
    if (error) {
      setIsSaving(false);

      if (error.kind === 'KeyAlreadyExists') {
        setErrors((prev) => ({
          ...prev,
          key: error,
        }));
      } else {
        console.error('Error creating KV item:', error);
      }
    }
  }, [error]);

  return (
    <Sheet open={isOpen} onOpenChange={setContainerOnOpenChange}>
      <SheetTrigger asChild>{children}</SheetTrigger>
      <SheetContent closeDisabled={isSaving} className="w-[500px] sm:max-w-[500px]">
        <SheetHeader>
          <SheetTitle>Create KV Item</SheetTitle>
          <SheetDescription>Set key, value, metadata and expiration date</SheetDescription>
        </SheetHeader>

        <div className="grid gap-4 py-4">
          <div className="grid grid-cols-12 items-start gap-4">
            <Label htmlFor="key" className="col-span-2 text-right pt-3">
              Key *
            </Label>
            <div className="col-span-10 space-y-2">
              <Input
                id="key"
                value={key}
                ref={nameInputRef}
                disabled={isSaving}
                onChange={handleKeyChange}
                className={cn(errors.key && 'border-red-500 focus-visible:ring-red-500')}
              />
              {errors.key && (
                <p className={cn('text-[0.8rem] font-medium text-destructive')}>An item with this key already exists</p>
              )}
            </div>
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
            <div className="col-span-10 space-y-2">
              <Textarea
                id="metadata"
                value={metadata}
                onChange={(e) => validateAndSetMetadata(e.target.value)}
                className={cn('min-h-[200px]', errors.metadata && 'border-red-500 focus-visible:ring-red-500')}
                disabled={isSaving}
              />
              {errors.metadata && (
                <p className={cn('text-[0.8rem] font-medium text-destructive')}>Must be a valid JSON</p>
              )}
            </div>
          </div>

          <div className="grid grid-cols-12 items-start gap-4">
            <Label htmlFor="expiration" className="col-span-2 text-right pt-3">
              Expiration
            </Label>
            <div className="col-span-10 w-full">
              <DateTimePicker
                container={sheetContainer}
                value={expiration}
                disabled={isSaving}
                onChange={(date) => setExpiration(date)}
              />
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

export default KvItemCreateSheet;
