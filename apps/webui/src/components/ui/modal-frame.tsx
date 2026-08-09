import * as DialogPrimitive from "@radix-ui/react-dialog";
import type { ReactNode } from "react";
import { cn } from "../../lib/utils";

interface ModalFrameProps {
  /** Controlled open state. The modal is always rendered controlled. */
  open: boolean;
  /** Called with `false` when Radix wants to close (Escape / outside click). */
  onOpenChange: (open: boolean) => void;
  /** Disable Escape-to-close and outside-click-to-close. Focus stays trapped. */
  dismissable?: boolean;
  /** z-index for the overlay + content (the app stacks a few modals). */
  zIndex?: number;
  /** Extra classes for the content frame (border, width, layout). */
  className?: string;
  /** Extra classes for the overlay (background tint, alignment, padding). */
  overlayClassName?: string;
  /** Must contain a <ModalTitle> for screen-reader labelling. */
  children: ReactNode;
}

/**
 * Headless brutalist modal frame on top of Radix Dialog. Gives every modal a
 * focus trap, Escape-to-close, outside-click-to-close, and focus restore for
 * free, while leaving the inner layout/styling entirely to the caller.
 *
 * Unlike the higher-level `Dialog`, this imposes no header band or fixed
 * width — callers compose their own chrome inside `children` and supply a
 * `<ModalTitle>` (and optional `<ModalDescription>`) for a11y.
 */
export default function ModalFrame({
  open,
  onOpenChange,
  dismissable = true,
  zIndex = 50,
  className,
  overlayClassName,
  children,
}: ModalFrameProps) {
  return (
    <DialogPrimitive.Root
      open={open}
      onOpenChange={(next) => {
        if (!next) onOpenChange(false);
      }}
    >
      <DialogPrimitive.Portal>
        <DialogPrimitive.Overlay
          className={cn("fixed inset-0 bg-black/80", overlayClassName)}
          style={{ zIndex }}
        />
        <DialogPrimitive.Content
          className={cn(
            "fixed left-1/2 top-1/2 w-full -translate-x-1/2 -translate-y-1/2 outline-none",
            className,
          )}
          style={{ zIndex }}
          onEscapeKeyDown={(event) => {
            if (!dismissable) event.preventDefault();
          }}
          onPointerDownOutside={(event) => {
            if (!dismissable) event.preventDefault();
          }}
          onInteractOutside={(event) => {
            if (!dismissable) event.preventDefault();
          }}
        >
          {children}
        </DialogPrimitive.Content>
      </DialogPrimitive.Portal>
    </DialogPrimitive.Root>
  );
}
