import * as React from 'react';
import { cn } from '@/lib/utils.ts';
import { Textarea } from '@/components/ui/textarea.tsx';

const TextFileInput = React.forwardRef<HTMLTextAreaElement, React.ComponentProps<'textarea'>>(
  ({ className, ...props }, ref) => {
    return <Textarea className={cn(className)} ref={ref} {...props} />;
  },
);

export default TextFileInput;
