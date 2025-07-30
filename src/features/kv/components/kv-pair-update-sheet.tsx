import { Button } from '@/components/ui/button.tsx';
import DateTimePicker from '@/components/ui/date-time-picker.tsx';
import { Input } from '@/components/ui/input.tsx';
import { Label } from '@/components/ui/label.tsx';
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
import { KvPairWriteInput } from '../kv-models.ts';
import {
  parseMetadataJSON,
  stringifyMetadataJSON,
  validateExpirationTTL,
  validateMetadata,
} from '@/features/kv/lib/kv-utils.ts';
import { Loader2Icon, Save } from 'lucide-react';
import { cn } from '@/lib/utils.ts';
import { useError } from '@/hooks/use-error.ts';
import TextFileInput from '@/components/ui/text-file-input.tsx';
import { ScrollArea } from '@/components/ui/scroll-area.tsx';

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
  const { kvPair, getKvPair, writeKvPair, isLoading, isWriting, error } = useKvPair();
  const { handleError } = useError();
  const valueInputRef = useRef<HTMLTextAreaElement>(null);
  const nameInputRef = useRef<HTMLInputElement>(null);

  const [sheetContainer, setSheetContainer] = useState<HTMLElement | null>(null);
  const [isOpen, setIsOpen] = useState(false);
  const [key, setKey] = useState(kvPair?.key ?? '');
  const [value, setValue] = useState(kvPair?.value);
  const [metadata, setMetadata] = useState<string>('');
  const [isSaving, setIsSaving] = useState(false);
  const [expiration, setExpiration] = useState(kvPair?.expiration);
  const [expirationTTL, setExpirationTTL] = useState('0');
  const [errors, setErrors] = useState<{ value?: Error; metadata?: Error; expiration?: Error; expirationTTL?: Error }>(
    {},
  );

  const isSaveButtonDisabled =
    isLoading ||
    isSaving ||
    !key ||
    !!errors.value ||
    !!errors.metadata ||
    !!errors.expiration ||
    !!errors.expirationTTL;

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

    setTimeout(async () => {
      const parsedMetadata = parseMetadataJSON(metadata);
      const writeInput: Omit<KvPairWriteInput, 'account_id'> = {
        namespace_id: namespaceId,
        key: key ?? '',
        value,
        expiration,
        expiration_ttl: Number(expirationTTL),
        metadata: parsedMetadata,
      };
      await writeKvPair(writeInput);
    }, 20);
  };

  const changeExpiration = (date: Date | undefined) => {
    setExpiration(date);
    setErrors((prevState) => ({ ...prevState, expiration: undefined }));
  };

  const handleChangeValue = (val: Uint8Array) => {
    setValue(val);
    setErrors((prevState) => ({ ...prevState, value: undefined }));
  };

  useEffect(() => {
    if (!isWriting && error) {
      setIsSaving(false);
      return;
    }

    // If writing is done and there is no error, close the sheet
    if (!isWriting && !error) {
      onUpdate().then(() => {
        setIsSaving(false);
        setIsOpen(false);
        onOpenChange(false);
        setErrors({});
      });
    }
  }, [isWriting]);

  useEffect(() => {
    valueInputRef.current?.focus();
    // Set cursor at the end
    valueInputRef.current?.setSelectionRange(valueInputRef.current.value.length, valueInputRef.current.value.length);
  }, [sheetContainer]);

  useEffect(() => {
    setKey(kvPair?.key ?? '');
    setValue(kvPair?.value);
    setExpiration(kvPair?.expiration);
    setMetadata(stringifyMetadataJSON(kvPair?.metadata ?? null));
    setExpirationTTL('0');
    setErrors({});
  }, [kvPair]);

  useEffect(() => loadKvPairOnOpenChange(open), [open]);

  useEffect(() => {
    if (error?.kind === 'InvalidMetadata') {
      setErrors((prevState) => ({ ...prevState, metadata: error }));
    } else if (error?.kind === 'InvalidExpiration') {
      setErrors((prevState) => ({ ...prevState, expiration: error }));
    } else if (error) {
      handleError(error);
    }
  }, [error]);

  return (
    <Sheet open={isOpen} onOpenChange={loadKvPairOnOpenChange}>
      <SheetTrigger asChild>{children}</SheetTrigger>

      <SheetContent closeDisabled={isSaving} className="grid grid-rows-[auto_1fr_auto] w-[550px] sm:max-w-[550px]">
        <SheetHeader>
          <SheetTitle>Edit KV Pair</SheetTitle>
          <SheetDescription>Edit value, metadata and expiration date</SheetDescription>
        </SheetHeader>

        <ScrollArea className="min-h-0">
          <div className="grid gap-4 p-4">
            <div className="grid grid-cols-[100px_1fr] items-center gap-4">
              <Label htmlFor="key" className="text-right">
                Key *
              </Label>
              {isLoading ? (
                <Skeleton className="w-full h-[36px] rounded-md" />
              ) : (
                <Input
                  id="key"
                  value={key}
                  disabled={true}
                  ref={nameInputRef}
                  onChange={(e) => setKey(e.target.value)}
                />
              )}
            </div>

            <div className="grid grid-cols-[100px_1fr] items-center gap-4">
              <Label htmlFor="value" className="self-start text-right mt-3">
                Value
              </Label>
              {isLoading ? (
                <div>
                  <Skeleton id="value" className="w-full h-[200px] rounded-md" />
                  <div className="grid grid-cols-[1fr_auto_auto] mt-2">
                    <div />
                    <Skeleton id="value" className="h-9 w-9 mr-2 rounded-md" />
                    <Skeleton id="value" className="h-9 w-9 rounded-md" />
                  </div>
                </div>
              ) : (
                <TextFileInput
                  id="value"
                  value={kvPair?.value}
                  onChangeValue={handleChangeValue}
                  onValueError={(error) => setErrors((prevState) => ({ ...prevState, value: error }))}
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
                  <div className="space-y-2">
                    <DateTimePicker
                      value={expiration}
                      disabled={isSaving}
                      onChange={changeExpiration}
                      className={cn(errors.expiration && 'border-red-500 focus-visible:ring-red-500')}
                    />
                    {errors.expiration && (
                      <p className={cn('text-[0.8rem] font-medium text-destructive')}>
                        Invalid expiration date. Date has to be in the future.
                      </p>
                    )}
                  </div>
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
        </ScrollArea>

        <SheetFooter>
          <Button type="submit" disabled={isSaveButtonDisabled} onClick={handleSaveClick} className="w-fit self-end">
            {isSaving ? (
              <>
                <Loader2Icon className="animate-spin" /> Saving...
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
