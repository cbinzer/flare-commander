'use client';

import { Button } from '@/components/ui/button';
import { Calendar } from '@/components/ui/calendar';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { ScrollArea, ScrollBar } from '@/components/ui/scroll-area';
import { cn } from '@/lib/utils';
import { format } from 'date-fns';
import { CalendarIcon, XIcon } from 'lucide-react';
import { FunctionComponent, MouseEvent, useEffect, useState } from 'react';

export interface DateTimePickerProps {
  value?: Date;
  disabled?: boolean;
  container?: Element | null | undefined;
  onChange?: (date: Date | undefined) => void;
}

const DateTimePicker: FunctionComponent<DateTimePickerProps> = ({
  value,
  container,
  disabled = false,
  onChange = () => {},
}) => {
  const [date, setDate] = useState<Date>();
  const [isOpen, setIsOpen] = useState(false);

  const hours = Array.from({ length: 24 }, (_, i) => i);
  const handleDateSelect = (selectedDate: Date | undefined) => {
    if (selectedDate) {
      setDate(selectedDate);
    }
  };

  const handleTimeChange = (type: 'hour' | 'minute', value: string) => {
    if (date) {
      const newDate = new Date(date);
      if (type === 'hour') {
        newDate.setHours(parseInt(value));
      } else if (type === 'minute') {
        newDate.setMinutes(parseInt(value));
      }
      setDate(newDate);
    }
  };

  const executeOnChangeAndSetOpenState = (open: boolean) => {
    setIsOpen(open);
    onChange(date);
  };

  const resetDate = (event: MouseEvent) => {
    console.log('resetDate');
    event.preventDefault();
    setDate(undefined);
    onChange(undefined);
  };

  useEffect(() => setDate(value), [value]);

  return (
    <Popover open={isOpen} onOpenChange={executeOnChangeAndSetOpenState}>
      <PopoverTrigger asChild>
        <div className="grid grid-cols-[1fr_auto] w-full border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground rounded-md">
          <Button
            disabled={disabled}
            variant="blank"
            className={cn('justify-start text-left font-normal', !date && 'text-muted-foreground')}
          >
            <CalendarIcon className="mr-2 h-4 w-4" />
            <span>{date ? format(date, 'yyyy-MM-dd HH:mm') : 'yyyy-MM-dd HH:mm'}</span>
          </Button>
          <Button onClick={resetDate} disabled={disabled} variant="blank">
            <XIcon className="h-4 w-4" />
          </Button>
        </div>
      </PopoverTrigger>

      <PopoverContent className="w-auto p-0" container={container}>
        <div className="sm:flex">
          <Calendar mode="single" selected={date} onSelect={handleDateSelect} initialFocus />

          <div className="flex flex-col sm:flex-row sm:h-[300px] divide-y sm:divide-y-0 sm:divide-x">
            <ScrollArea className="w-64 sm:w-auto">
              <div className="flex sm:flex-col p-2">
                {hours.reverse().map((hour) => (
                  <Button
                    key={hour}
                    size="icon"
                    variant={date && date.getHours() === hour ? 'default' : 'ghost'}
                    className="sm:w-full shrink-0 aspect-square"
                    onClick={() => handleTimeChange('hour', hour.toString())}
                  >
                    {hour}
                  </Button>
                ))}
              </div>
              <ScrollBar orientation="horizontal" className="sm:hidden" />
            </ScrollArea>

            <ScrollArea className="w-64 sm:w-auto">
              <div className="flex sm:flex-col p-2">
                {Array.from({ length: 12 }, (_, i) => i * 5).map((minute) => (
                  <Button
                    key={minute}
                    size="icon"
                    variant={date && date.getMinutes() === minute ? 'default' : 'ghost'}
                    className="sm:w-full shrink-0 aspect-square"
                    onClick={() => handleTimeChange('minute', minute.toString())}
                  >
                    {minute.toString().padStart(2, '0')}
                  </Button>
                ))}
              </div>
              <ScrollBar orientation="horizontal" className="sm:hidden" />
            </ScrollArea>
          </div>
        </div>
      </PopoverContent>
    </Popover>
  );
};

export default DateTimePicker;
