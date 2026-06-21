import * as SelectPrimitive from "@radix-ui/react-select";
import { cn } from "../../lib/utils";

interface SelectOption {
  value: string;
  label: string;
}

interface SelectProps {
  options: SelectOption[];
  value?: string;
  onValueChange?: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  className?: string;
}

export default function Select({
  options,
  value,
  onValueChange,
  placeholder = "Select...",
  disabled = false,
  className,
}: SelectProps) {
  return (
    <SelectPrimitive.Root
      value={value}
      onValueChange={onValueChange}
      disabled={disabled}
    >
      <SelectPrimitive.Trigger
        className={cn(
          "inline-flex w-full items-center justify-between gap-2 border border-edge bg-black px-4 py-2.5 font-mono text-sm text-white transition-colors",
          "hover:border-muted hover:bg-surface-hover",
          "focus:outline-none focus:ring-2 focus:ring-accent-lime focus:ring-offset-2 focus:ring-offset-black",
          "disabled:cursor-not-allowed disabled:opacity-40",
          className
        )}
      >
        <SelectPrimitive.Value placeholder={placeholder} />
        <SelectPrimitive.Icon>
          <svg
            width="12"
            height="12"
            viewBox="0 0 12 12"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              d="M2 4L6 8L10 4"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="square"
            />
          </svg>
        </SelectPrimitive.Icon>
      </SelectPrimitive.Trigger>
      <SelectPrimitive.Portal>
        <SelectPrimitive.Content
          className={cn(
            "relative z-50 min-w-[8rem] overflow-hidden border border-edge-strong bg-black",
            "shadow-card-md",
            "data-[state=open]:animate-in data-[state=closed]:animate-out",
            "data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0",
            "data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95"
          )}
          position="popper"
          sideOffset={4}
        >
          <SelectPrimitive.Viewport className="p-1">
            {options.map((option) => (
              <SelectPrimitive.Item
                key={option.value}
                value={option.value}
                className={cn(
                  "relative flex cursor-pointer select-none items-center px-4 py-2.5 font-mono text-sm text-muted outline-none transition-colors",
                  "hover:bg-accent-lime hover:text-black",
                  "focus:bg-accent-lime focus:text-black",
                  "data-[state=checked]:bg-accent-lime data-[state=checked]:text-black data-[state=checked]:font-bold"
                )}
              >
                <SelectPrimitive.ItemText>{option.label}</SelectPrimitive.ItemText>
              </SelectPrimitive.Item>
            ))}
          </SelectPrimitive.Viewport>
        </SelectPrimitive.Content>
      </SelectPrimitive.Portal>
    </SelectPrimitive.Root>
  );
}

export { SelectPrimitive };
