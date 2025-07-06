import { cn } from '@/lib/utils.ts';
import { Textarea } from '@/components/ui/textarea.tsx';
import { ChangeEvent, ComponentProps, forwardRef, RefObject, useEffect, useState } from 'react';
import { BinaryIcon, FileDown, FileUp, XIcon } from 'lucide-react';
import { Button } from '@/components/ui/button.tsx';
import { open, save, SaveDialogOptions } from '@tauri-apps/plugin-dialog';
import { readFile, writeFile } from '@tauri-apps/plugin-fs';
import { fileTypeFromBuffer } from 'file-type';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip.tsx';

interface TextFileInputProps extends Omit<ComponentProps<'textarea'>, 'value'> {
  value?: Uint8Array;
  onChangeValue?: (value: Uint8Array) => void;
  onValueError?: (error: Error) => void;
}

interface ValueType {
  type: string;
  isBinary: boolean;
  extension?: string;
}

const TextFileInput = forwardRef<HTMLTextAreaElement, TextFileInputProps>(
  ({ className, disabled, value, onChange, onChangeValue, onValueError, ...props }, ref) => {
    const [texValue, setTexValue] = useState<string>('');
    const [valueType, setValueType] = useState<ValueType>({
      type: 'unknown',
      isBinary: true,
    });
    const [error, setError] = useState<Error | undefined>();

    const saveFile = async () => {
      const dialogOptions: SaveDialogOptions = {
        filters: [],
      };
      if (valueType.extension) {
        dialogOptions.filters = [
          {
            name: valueType.type,
            extensions: [valueType.extension],
          },
        ];
      }

      const path = await save(dialogOptions);
      if (path) {
        await writeFile(path, encodeValue(texValue ?? ''));
      }
    };

    const loadFile = async () => {
      const path = await open();
      if (path) {
        const fileContent = await readFile(path);
        const valueType = await determineValueType(fileContent);
        setValueType(valueType);

        if (valueType.isBinary) {
          setTexValue('');
        } else {
          setTexValue(uint8ArrayToString(fileContent));
        }

        if (onChangeValue) {
          onChangeValue(fileContent);
        }

        const error = validateValue(fileContent);
        if (error) {
          setError(error);
          if (onValueError) {
            onValueError(error);
          }
        } else {
          setError(undefined);
        }
      }
    };

    const changeValue = (changeEvent: ChangeEvent<HTMLTextAreaElement>) => {
      setTexValue(changeEvent.target.value);

      if (onChange) {
        onChange(changeEvent);
      }

      const encodedValue = encodeValue(changeEvent.target.value);
      if (onChangeValue) {
        onChangeValue(encodedValue);
      }

      const error = validateValue(encodedValue);
      if (error) {
        setError(error);
        if (onValueError) {
          onValueError(error);
        }
      } else {
        setError(undefined);
      }
    };

    const resetValue = () => {
      setTexValue('');
      setValueType({ type: 'unknown', isBinary: false });

      if (onChangeValue) {
        onChangeValue(new Uint8Array());
      }

      setError(undefined);

      setTimeout(() => (ref as RefObject<HTMLTextAreaElement>)?.current?.focus());
    };

    useEffect(() => {
      determineValueType(value ?? new Uint8Array()).then((newValueType) => {
        setValueType(newValueType);

        if (!value || newValueType.isBinary) {
          setTexValue('');
        } else {
          setTexValue(uint8ArrayToString(value));
        }
      });
    }, [value]);

    return (
      <div className="space-y-2">
        {valueType.isBinary ? (
          <div
            className={cn(
              className,
              'grid rounded-md border border-input bg-transparent px-3 py-2 shadow-sm relative',
              disabled ? 'opacity-50' : '',
              error ? 'border-red-500 focus-visible:ring-red-500' : '',
            )}
          >
            <Button onClick={resetValue} className="absolute top-0 right-0" disabled={disabled} variant="blank">
              <XIcon className="h-4 w-4" />
            </Button>
            <div className="grid grid-cols-[auto_auto_auto] m-auto font-weight items-center">
              <BinaryIcon className="inline size-[18px]" />
              <div>Binary data</div>
              <div>{valueType.extension ? <>&nbsp;({valueType.extension})</> : ''}</div>
            </div>
          </div>
        ) : (
          <Textarea
            className={cn(className)}
            ref={ref}
            disabled={valueType.isBinary || disabled}
            value={texValue}
            onChange={changeValue}
            {...props}
          />
        )}

        {error && <p className={cn('text-[0.8rem] font-medium text-destructive')}>{error.message}</p>}

        <div className="mt-2 text-right">
          <Tooltip delayDuration={1500}>
            <TooltipTrigger asChild>
              <Button className="mr-2" variant="outline" size="icon" disabled={disabled} onClick={loadFile}>
                <FileUp />
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              <p>Upload File</p>
            </TooltipContent>
          </Tooltip>
          <Tooltip delayDuration={1500}>
            <TooltipTrigger asChild>
              <Button variant="outline" size="icon" disabled={disabled} onClick={saveFile}>
                <FileDown />
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              <p>Download File</p>
            </TooltipContent>
          </Tooltip>
        </div>
      </div>
    );
  },
);

function uint8ArrayToString(uint8array: Uint8Array): string {
  let result = '';
  const chunkSize = 0x8000; // 32k

  for (let i = 0; i < uint8array.length; i += chunkSize) {
    result += String.fromCharCode(...uint8array.subarray(i, i + chunkSize));
  }

  return result;
}

function encodeValue(val: string): Uint8Array {
  const uint8array = new Uint8Array(val.length);
  for (let i = 0; i < val.length; i++) {
    uint8array[i] = val?.charCodeAt(i) & 0xff;
  }

  return uint8array;
}

async function determineValueType(value: Uint8Array): Promise<ValueType> {
  const fileType = await fileTypeFromBuffer(value);
  if (fileType) {
    return {
      type: fileType.mime,
      isBinary: true,
      extension: fileType.ext,
    };
  }

  return {
    type: 'unknown',
    isBinary: hasUnreadableSigns(value),
  };
}

function hasUnreadableSigns(uint8array: Uint8Array): boolean {
  return uint8array.some((byte) => {
    // Allow: tab (9), newline (10), carriage return (13), space (32) to tilde (126)
    if (byte < 32 && byte !== 9 && byte !== 10 && byte !== 13) {
      return true;
    }

    if (byte > 126) {
      return true;
    }
  });
}

function validateValue(value: Uint8Array): Error | undefined {
  if (value.length > 26214400) {
    return new Error(`Value length of ${value.length} exceeds limit of 26214400.`);
  }

  return undefined;
}

export default TextFileInput;
