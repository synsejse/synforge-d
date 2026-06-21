import { useId } from "react";
import { cn } from "../../lib/utils";

export type SegmentedTone = "lime" | "success" | "orange" | "white";

export interface SegmentedItem<TValue extends string> {
  value: TValue;
  label: string;
  /** Optional override for the active fill color. Defaults to "lime". */
  tone?: SegmentedTone;
  disabled?: boolean;
}

export interface SegmentedControlProps<TValue extends string> {
  value: TValue;
  onChange: (next: TValue) => void;
  items: ReadonlyArray<SegmentedItem<TValue>>;
  /** Visual density. */
  size?: "sm" | "md" | "lg";
  /** Force every segment to share the available width. */
  fullWidth?: boolean;
  /** Accessible label describing what the group selects. */
  ariaLabel: string;
  className?: string;
}

const sizeClasses: Record<NonNullable<SegmentedControlProps<string>["size"]>, string> = {
  sm: "px-3 py-1.5 text-xs tracking-[0.16em]",
  // md matches form-input height (~44px) so a kind/status segmented sits
  // level with sibling textboxes in a filter row.
  md: "px-4 py-3 text-xs tracking-[0.18em]",
  lg: "px-5 py-3 text-sm tracking-[0.18em]",
};

const toneActiveClasses: Record<SegmentedTone, string> = {
  lime: "bg-accent-lime text-black",
  success: "bg-success text-black",
  orange: "bg-accent-orange text-black",
  white: "bg-white text-black",
};

/**
 * Brutalist segmented toggle group: bordered container, full-width fill on
 * the selected segment. Replaces hand-rolled `<button>` groups for a
 * mutually-exclusive choice.
 */
export default function SegmentedControl<TValue extends string>({
  value,
  onChange,
  items,
  size = "md",
  fullWidth = false,
  ariaLabel,
  className,
}: SegmentedControlProps<TValue>) {
  const groupId = useId();
  return (
    <div
      role="group"
      aria-label={ariaLabel}
      className={cn(
        "border border-edge",
        fullWidth ? "flex w-full" : "inline-flex",
        className,
      )}
    >
      {items.map((item, idx) => {
        const isActive = item.value === value;
        const tone = item.tone ?? "lime";
        return (
          <button
            key={`${groupId}-${item.value}`}
            type="button"
            onClick={() => onChange(item.value)}
            aria-pressed={isActive}
            disabled={item.disabled}
            className={cn(
              "font-mono font-bold uppercase transition-colors",
              "focus-visible:outline-none focus-visible:relative focus-visible:z-10 focus-visible:ring-2 focus-visible:ring-accent-lime focus-visible:ring-offset-2 focus-visible:ring-offset-black",
              "disabled:pointer-events-none disabled:opacity-40",
              sizeClasses[size],
              fullWidth ? "flex-1" : "",
              idx > 0 ? "border-l border-edge-strong" : "",
              isActive
                ? toneActiveClasses[tone]
                : "bg-black text-muted hover:text-white",
            )}
          >
            {item.label}
          </button>
        );
      })}
    </div>
  );
}
