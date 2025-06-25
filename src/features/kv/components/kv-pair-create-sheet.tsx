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
import { Textarea } from '@/components/ui/textarea.tsx';
import { ChangeEvent, FunctionComponent, ReactNode, useEffect, useRef, useState } from 'react';
import { useKvPair } from '../hooks/use-kv-pair.ts';
import { KvPair, KvPairCreateInput } from '../kv-models.ts';
import { parseMetadataJSON, validateExpirationTTL, validateMetadata } from '@/features/kv/lib/kv-utils.ts';
import { cn } from '@/lib/utils.ts';
import { PlusIcon } from 'lucide-react';
import { useError } from '@/hooks/use-error.ts';
import TextFileInput from '@/components/ui/text-file-input.tsx';

export interface KvPairCreateSheetProps {
  namespaceId: string;
  onCreate?: (item: KvPair) => Promise<void>;
  children?: ReactNode;
}

const KvPairCreateSheet: FunctionComponent<KvPairCreateSheetProps> = ({
  namespaceId,
  children,
  onCreate = () => Promise.resolve(),
}) => {
  const { kvPair, error, createKvPair } = useKvPair();
  const { handleError } = useError();

  const valueInputRef = useRef<HTMLTextAreaElement>(null);
  const nameInputRef = useRef<HTMLInputElement>(null);

  const [sheetContainer, setSheetContainer] = useState<HTMLElement | null>(null);
  const [isOpen, setIsOpen] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [key, setKey] = useState<string | undefined>(undefined);
  const [value, setValue] = useState<string | undefined>(undefined);
  const [metadata, setMetadata] = useState('');
  const [expiration, setExpiration] = useState<Date | undefined>(undefined);
  const [expirationTTL, setExpirationTTL] = useState('0');
  const [errors, setErrors] = useState<{ key?: Error; metadata?: Error; expiration?: Error; expirationTTL?: Error }>(
    {},
  );

  const isSaveButtonDisabled =
    isSaving || !key || !!errors.key || !!errors.metadata || !!errors.expiration || !!errors.expirationTTL;

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

    const parsedMetadata = parseMetadataJSON(metadata);
    const createInput: Omit<KvPairCreateInput, 'account_id'> = {
      namespace_id: namespaceId,
      key: key ?? '',
      value,
      expiration,
      expiration_ttl: Number(expirationTTL),
      metadata: parsedMetadata,
    };
    await createKvPair(createInput);
  };

  const changeExpiration = (date: Date | undefined) => {
    setExpiration(date);
    setErrors((prevState) => ({ ...prevState, expiration: undefined }));
  };

  useEffect(() => {
    if (kvPair) {
      onCreate(kvPair)
        .then(() => setIsOpen(false))
        .catch(handleError)
        .finally(() => setIsSaving(false));
    }
  }, [kvPair]);

  useEffect(() => {
    setKey(undefined);
    setValue(undefined);
    setMetadata('');
    setExpiration(undefined);
    setExpirationTTL('0');
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
      } else if (error.kind === 'InvalidMetadata') {
        setErrors((prev) => ({ ...prev, metadata: error }));
      } else if (error.kind === 'InvalidExpiration') {
        setErrors((prev) => ({ ...prev, expiration: error }));
      } else {
        handleError(error);
      }
    }
  }, [error]);

  return (
    <Sheet open={isOpen} onOpenChange={setContainerOnOpenChange}>
      <SheetTrigger asChild>{children}</SheetTrigger>
      <SheetContent closeDisabled={isSaving} className="w-[550px] sm:max-w-[550px]">
        <SheetHeader>
          <SheetTitle>Create KV Pair</SheetTitle>
          <SheetDescription>Set key, value, metadata and expiration date</SheetDescription>
        </SheetHeader>

        <div className="grid gap-4 py-4">
          <div className="grid grid-cols-[100px_1fr] items-center gap-4">
            <Label htmlFor="key" className="text-right">
              Key *
            </Label>
            <div className="space-y-2">
              <Input
                id="key"
                value={key}
                ref={nameInputRef}
                disabled={isSaving}
                onChange={handleKeyChange}
                className={cn(errors.key && 'border-red-500 focus-visible:ring-red-500')}
              />
              {errors.key && (
                <p className={cn('text-[0.8rem] font-medium text-destructive')}>A pair with this key already exists</p>
              )}
            </div>
          </div>

          <div className="grid grid-cols-[100px_1fr] items-center gap-4">
            <Label htmlFor="value" className="self-start text-right mt-3">
              Value
            </Label>
            <TextFileInput
              id="value"
              value={value}
              onChange={(e) => setValue(e.target.value)}
              className="min-h-[200px]"
              ref={valueInputRef}
              disabled={isSaving}
            />
          </div>

          <div className="grid grid-cols-[100px_1fr] items-center gap-4">
            <Label htmlFor="metadata" className="self-start text-right mt-3">
              Metadata
            </Label>
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
          </div>

          <div className="grid grid-cols-[100px_1fr] items-center gap-4">
            <Label htmlFor="expiration" className="text-right">
              Expiration
            </Label>
            <div className="space-y-2">
              <DateTimePicker
                container={sheetContainer}
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
          </div>

          <div className="grid grid-cols-[100px_1fr] items-center gap-4">
            <Label htmlFor="expirationTTL" className="text-right">
              Expiration TTL
            </Label>
            <div className="space-y-2">
              <Input
                id="expirationTTL"
                type="text"
                value={expirationTTL}
                disabled={isSaving}
                onChange={(e) => validateAndSetExpirationTTL(e.target.value)}
                className={cn(errors.expirationTTL && 'border-red-500 focus-visible:ring-red-500')}
              />
              {errors.expirationTTL && (
                <p className={cn('text-[0.8rem] font-medium text-destructive')}>{errors.expirationTTL.message}</p>
              )}
            </div>
          </div>
        </div>

        <SheetFooter>
          <Button type="submit" disabled={isSaveButtonDisabled} onClick={handleSaveClick}>
            {isSaving ? (
              <>
                <LoadingSpinner /> Creating...
              </>
            ) : (
              <>
                <PlusIcon /> Create
              </>
            )}
          </Button>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  );
};

export default KvPairCreateSheet;
