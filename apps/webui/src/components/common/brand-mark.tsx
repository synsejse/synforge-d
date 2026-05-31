type BrandMarkProps = {
  className?: string;
  title?: string;
};

// The Synforge anvil mark, glyph-only (transparent) so it sits inside a
// framed container. The hardy hole is cut with even-odd fill so it reads as
// negative space against any background.
export default function BrandMark({ className, title }: BrandMarkProps) {
  return (
    <svg
      viewBox="9 3 52 47"
      className={className}
      role={title ? "img" : undefined}
      aria-label={title}
      aria-hidden={title ? undefined : true}
      focusable="false"
    >
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M12 19 H46 L58 24 L46 27 H39 V39 L51 41 V47 H13 V41 L25 39 V27 H12 Z M34 21 h4 v3.4 h-4 Z"
        fill="#BFFF00"
      />
      <path
        d="M45 6 L47.3 10 L51 11.5 L47.3 13 L45 17 L42.7 13 L39 11.5 L42.7 10 Z"
        fill="#FF6B00"
      />
    </svg>
  );
}
