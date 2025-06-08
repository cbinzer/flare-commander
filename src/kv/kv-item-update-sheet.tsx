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
import { KvKeyPairWriteInput, KvMetadata } from './kv-models';
import { parseMetadataJSON, stringifyMetadataJSON, validateExpirationTTL, validateMetadata } from '@/kv/kv-utils.ts';
import { Save } from 'lucide-react';
import { cn } from '@/lib/utils.ts';

export interface KvItemUpdateSheetProps {
  namespaceId: string;
  itemKey: string;
  itemMetadata: KvMetadata;
  open?: boolean;
  onUpdate?: () => Promise<void>;
  onOpenChange?: (open: boolean) => void;
  children?: ReactNode;
}

const KvItemUpdateSheet: FunctionComponent<KvItemUpdateSheetProps> = ({
  namespaceId,
  itemKey,
  itemMetadata,
  open = false,
  children,
  onUpdate = () => Promise.resolve(),
  onOpenChange = () => {},
}) => {
  const { kvItem, getKvPair, writeKvPair, isLoading } = useKvItem();
  const valueInputRef = useRef<HTMLTextAreaElement>(null);
  const nameInputRef = useRef<HTMLInputElement>(null);

  const [sheetContainer, setSheetContainer] = useState<HTMLElement | null>(null);
  const [isOpen, setIsOpen] = useState(false);
  const [key, setKey] = useState(kvItem?.key);
  const [value, setValue] = useState(kvItem?.value);
  const [metadata, setMetadata] = useState(stringifyMetadataJSON(itemMetadata));
  const [isSaving, setIsSaving] = useState(false);
  const [expiration, setExpiration] = useState(kvItem?.expiration);
  const [expirationTTL, setExpirationTTL] = useState('0');
  const [errors, setErrors] = useState<{ metadata?: Error; expirationTTL?: Error }>({});

  const isSaveButtonDisabled = isLoading || isSaving || !key || !!errors.metadata;

  const loadKvItemOnOpenChange = (open: boolean) => {
    onOpenChange(open);
    setIsOpen(open);

    if (!open) {
      return;
    }

    getKvPair(namespaceId, itemKey).then(() =>
      setSheetContainer(document.querySelector('[role="dialog"]') as HTMLElement),
    );
  };

  const validateAndSetMetadata = (value: string) => {
    setMetadata(value);
    setErrors((prev) => ({ ...prev, metadata: validateMetadata(value) ? undefined : new Error('Invalid JSON') }));
  };

  const validateAndSetExpirationTTL = (value: string) => {
    setExpirationTTL(value);
    setErrors((prev) => ({
      ...prev,
      expirationTTL: validateExpirationTTL(value)
        ? undefined
        : new Error('Expiration TTL must be 0 or at least 60 seconds'),
    }));
  };

  const handleSaveClick = async () => {
    setIsSaving(true);

    try {
      const parsedMetadata = parseMetadataJSON(metadata);
      const upsertInput: Omit<KvKeyPairWriteInput, 'account_id'> = {
        namespace_id: namespaceId,
        key: key ?? '',
        value,
        expiration,
        expiration_ttl: Number(expirationTTL),
        metadata: parsedMetadata,
      };
      await writeKvPair(upsertInput);
      await onUpdate();

      setIsOpen(false);
      onOpenChange(false);
    } catch (e) {
      console.error('Error parsing metadata:', e);
      setErrors((prevState) => ({ ...prevState, metadata: e as Error }));
    } finally {
      setIsSaving(false);
    }
  };

  useEffect(() => {
    valueInputRef.current?.focus();
    // Set cursor at the end
    valueInputRef.current?.setSelectionRange(valueInputRef.current.value.length, valueInputRef.current.value.length);
  }, [sheetContainer]);

  useEffect(() => {
    setKey(kvItem?.key);
    setValue(kvItem?.value);
    setExpiration(kvItem?.expiration);
    setExpirationTTL('0');
  }, [kvItem]);

  useEffect(() => loadKvItemOnOpenChange(open), [open]);

  return (
    <Sheet open={isOpen} onOpenChange={loadKvItemOnOpenChange}>
      <SheetTrigger asChild>{children}</SheetTrigger>

      <SheetContent closeDisabled={isSaving} className="w-[500px] sm:max-w-[500px]">
        <SheetHeader>
          <SheetTitle>Edit KV Item</SheetTitle>
          <SheetDescription>Edit value, metadata and expiration date</SheetDescription>
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
                disabled={true}
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
                  container={sheetContainer}
                  value={expiration}
                  disabled={isSaving}
                  onChange={(date) => setExpiration(date)}
                />
              )}
            </div>
          </div>

          <div className="grid grid-cols-12 items-start gap-4">
            <Label htmlFor="expirationTTL" className="col-span-2 text-right pt-3">
              Expiration TTL
            </Label>
            <div className="col-span-10 space-y-2">
              {isLoading ? (
                <Skeleton className="w-full h-[36px] rounded-md" />
              ) : (
                <>
                  <Input
                    id="expirationTTL"
                    value={expirationTTL}
                    disabled={isSaving}
                    onChange={(e) => validateAndSetExpirationTTL(e.target.value)}
                    className={cn(errors.expirationTTL && 'border-red-500 focus-visible:ring-red-500')}
                  />
                  {errors.expirationTTL && (
                    <p className={cn('text-[0.8rem] font-medium text-destructive')}>{errors.expirationTTL.message}</p>
                  )}
                </>
              )}
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

export default KvItemUpdateSheet;
