import { Check, ChevronDown } from 'lucide-react';
import * as DropdownMenuPrimitive from '@radix-ui/react-dropdown-menu';
import * as React from 'react';

import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { cn } from '@/lib/utils';

export interface MultiSelectOption {
  value: string;
  label: string;
}

export interface MultiSelectProps {
  options: MultiSelectOption[];
  value: string[];
  onChange: (value: string[]) => void;
  placeholder?: string;
  className?: string;
}

const MultiSelectCheckboxItem = React.forwardRef<
  React.ElementRef<typeof DropdownMenuPrimitive.CheckboxItem>,
  React.ComponentPropsWithoutRef<typeof DropdownMenuPrimitive.CheckboxItem>
>(({ className, children, checked, ...props }, ref) => (
  <DropdownMenuPrimitive.CheckboxItem
    ref={ref}
    checked={checked}
    className={cn('relative flex cursor-default select-none items-center rounded-sm py-1.5 pl-8 pr-2 text-xs outline-none focus:bg-accent focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50', className)}
    {...props}
  >
    <span className="absolute left-2 flex size-3.5 items-center justify-center">
      <DropdownMenuPrimitive.ItemIndicator>
        <Check className="size-4" />
      </DropdownMenuPrimitive.ItemIndicator>
    </span>
    {children}
  </DropdownMenuPrimitive.CheckboxItem>
));
MultiSelectCheckboxItem.displayName = DropdownMenuPrimitive.CheckboxItem.displayName;

export function MultiSelect({ options, value, onChange, placeholder = 'Select...', className }: MultiSelectProps) {
  const [open, setOpen] = React.useState(false);

  const handleSelect = (optionValue: string) => {
    onChange(value.includes(optionValue) ? value.filter((item) => item !== optionValue) : [...value, optionValue]);
  };

  const displayText =
    value.length === 0
      ? placeholder
      : value.length === 1
        ? options.find((option) => option.value === value[0])?.label || placeholder
        : `${value.length} selected`;

  return (
    <DropdownMenu open={open} onOpenChange={setOpen}>
      <DropdownMenuTrigger asChild>
        <button
          type="button"
          className={cn('flex h-8 w-full items-center justify-between rounded-full border border-border bg-card px-3 py-2 text-xs ring-offset-background placeholder:text-muted-foreground transition-colors hover:bg-background/50 focus:outline-none focus:ring-2 focus:ring-ring/70 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50', className)}
        >
          <span className="line-clamp-1">{displayText}</span>
          <ChevronDown className="size-4 opacity-50" />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="max-h-96 overflow-auto" align="start" onCloseAutoFocus={(event) => event.preventDefault()}>
        {options.map((option) => (
          <MultiSelectCheckboxItem
            key={option.value}
            checked={value.includes(option.value)}
            onSelect={(event) => {
              event.preventDefault();
              handleSelect(option.value);
            }}
          >
            {option.label}
          </MultiSelectCheckboxItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
