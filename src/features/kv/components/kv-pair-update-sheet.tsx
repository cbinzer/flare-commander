import { Button } from '@/components/ui/button.tsx';
import DateTimePicker from '@/components/ui/date-time-picker.tsx';
import { Input } from '@/components/ui/input.tsx';
import { Label } from '@/components/ui/label.tsx';
import { LoadingSpinner } from '@/components/ui/loading-spinner.tsx';
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
import { Textarea } from '@/components/ui/textarea.tsx';
import { FunctionComponent, ReactNode, useEffect, useRef, useState } from 'react';
import { useKvPair } from '../hooks/use-kv-pair.ts';
import { KvError, KvKeyPairWriteInput } from '../kv-models.ts';
import {
  parseMetadataJSON,
  stringifyMetadataJSON,
  validateExpirationTTL,
  validateMetadata,
} from '@/features/kv/lib/kv-utils.ts';
import { Save } from 'lucide-react';
import { cn } from '@/lib/utils.ts';
import { useError } from '@/hooks/use-error.ts';

export interface KvPairUpdateSheetProps {
  namespaceId: string;
  itemKey: string;
  open?: boolean;
  onUpdate?: () => Promise<void>;
  onOpenChange?: (open: boolean) => void;
  children?: ReactNode;
}

const KvPairUpdateSheet: FunctionComponent<KvPairUpdateSheetProps> = ({
  namespaceId,
  itemKey,
  open = false,
  children,
  onUpdate = () => Promise.resolve(),
  onOpenChange = () => {},
}) => {
  const { kvPair, getKvPair, writeKvPair, isLoading } = useKvPair();
  const { handleError } = useError();
  const valueInputRef = useRef<HTMLTextAreaElement>(null);
  const nameInputRef = useRef<HTMLInputElement>(null);

  const [sheetContainer, setSheetContainer] = useState<HTMLElement | null>(null);
  const [isOpen, setIsOpen] = useState(false);
  const [key, setKey] = useState(kvPair?.key);
  const [value, setValue] = useState(kvPair?.value);
  const [metadata, setMetadata] = useState<string>('');
  const [isSaving, setIsSaving] = useState(false);
  const [expiration, setExpiration] = useState(kvPair?.expiration);
  const [expirationTTL, setExpirationTTL] = useState('0');
  const [errors, setErrors] = useState<{ metadata?: Error; expirationTTL?: Error }>({});

  const isSaveButtonDisabled = isLoading || isSaving || !key || !!errors.metadata;

  const loadKvPairOnOpenChange = (open: boolean) => {
    onOpenChange(open);
    setIsOpen(open);

    if (!open) {
      return;
    }

    getKvPair(namespaceId, itemKey)
      .then(() => setSheetContainer(document.querySelector('[role="dialog"]') as HTMLElement))
      .catch(handleError);
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
      const error = e as KvError;
      if (error.kind === 'InvalidMetadata') {
        setErrors((prevState) => ({ ...prevState, metadata: error }));
      } else {
        handleError(error);
      }
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
    console.log(kvPair);
    setKey(kvPair?.key);
    setValue(kvPair?.value);
    setExpiration(kvPair?.expiration);
    setMetadata(stringifyMetadataJSON(kvPair?.metadata ?? null));
    setExpirationTTL('0');
  }, [kvPair]);

  useEffect(() => loadKvPairOnOpenChange(open), [open]);

  return (
    <Sheet open={isOpen} onOpenChange={loadKvPairOnOpenChange}>
      <SheetTrigger asChild>{children}</SheetTrigger>

      <SheetContent closeDisabled={isSaving} className="w-[550px] sm:max-w-[550px]">
        <SheetHeader>
          <SheetTitle>Edit KV Pair</SheetTitle>
          <SheetDescription>Edit value, metadata and expiration date</SheetDescription>
        </SheetHeader>

        <div className="grid gap-4 py-4">
          <div className="grid grid-cols-[100px_1fr] items-center gap-4">
            <Label htmlFor="key" className="text-right">
              Key *
            </Label>
            {isLoading ? (
              <Skeleton className="w-full h-[36px] rounded-md" />
            ) : (
              <Input id="key" value={key} disabled={true} ref={nameInputRef} onChange={(e) => setKey(e.target.value)} />
            )}
          </div>

          <div className="grid grid-cols-[100px_1fr] items-center gap-4">
            <Label htmlFor="value" className="self-start text-right mt-3">
              Value
            </Label>
            {isLoading ? (
              <Skeleton id="value" className="w-full h-[200px] rounded-md" />
            ) : (
              <Textarea
                id="value"
                value={value}
                onChange={(e) => setValue(e.target.value)}
                className="min-h-[200px]"
                ref={valueInputRef}
                disabled={isSaving}
              />
            )}
          </div>

          <div className="grid grid-cols-[100px_1fr] items-center gap-4">
            <Label htmlFor="metadata" className="self-start text-right mt-3">
              Metadata
            </Label>
            {isLoading ? (
              <Skeleton id="metadata" className="w-full h-[200px] rounded-md" />
            ) : (
              <div className="space-y-2">
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

          <div className="grid grid-cols-[100px_1fr] items-center gap-4">
            <Label htmlFor="expiration" className="text-right">
              Expiration
            </Label>
            <div className="w-full">
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

          <div className="grid grid-cols-[100px_1fr] items-center gap-4">
            <Label htmlFor="expirationTTL" className="text-right">
              Expiration TTL
            </Label>
            <div className="space-y-2">
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

export default KvPairUpdateSheet;
