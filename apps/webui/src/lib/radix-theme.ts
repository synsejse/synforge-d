/**
 * Radix Themes Configuration - Technical Brutalism
 * Sharp-edge design tokens for SynForge build orchestration
 */

export const radixThemeConfig = {
  accentColor: "gray",
  grayColor: "gray",
  radius: "none", // Zero border radius - brutalist aesthetic
  scaling: "100%",
} as const;

/**
 * Custom CSS overrides for Radix Themes
 * Brutalist design system with sharp edges and high contrast
 */
export const radixCustomStyles = `
  .radix-themes {
    --default-font-family: "Inter Tight", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    --heading-font-family: "JetBrains Mono", monospace;
    --code-font-family: "Fira Code", "JetBrains Mono", monospace;
    
    /* Override Radix's border radius to enforce sharp edges */
    --radius-1: 0px;
    --radius-2: 0px;
    --radius-3: 0px;
    --radius-4: 0px;
    --radius-5: 0px;
    --radius-6: 0px;
    --radius-full: 0px;
    
    /* Brutalist accent color - electric lime */
    --accent-9: var(--theme-accent-lime);
    --accent-10: var(--theme-accent-lime);
    --accent-11: var(--theme-accent-lime);
    
    /* Sharp focus outlines */
    --focus-8: var(--theme-accent-lime);
  }
  
  /* Force sharp corners on all Radix components */
  .rt-reset,
  .rt-BaseButton,
  .rt-Button,
  .rt-Card,
  .rt-Dialog,
  .rt-DropdownMenu,
  .rt-Select,
  .rt-TextField,
  .rt-TextArea,
  .rt-Checkbox,
  .rt-Switch,
  .rt-Badge,
  .rt-Code {
    border-radius: 0 !important;
  }
  
  /* Brutalist shadows - hard edges, no blur */
  .rt-Card,
  .rt-Dialog {
    box-shadow: var(--shadow-brutal-md);
  }
  
  .rt-DropdownMenuContent,
  .rt-PopoverContent {
    box-shadow: var(--shadow-brutal-lg);
  }
  
  /* Sharp hover states - linear transitions */
  .rt-Button,
  .rt-BaseButton {
    transition: all 0.1s linear;
  }
  
  /* Monospace for display elements */
  .rt-Heading,
  .rt-DialogTitle {
    font-family: var(--heading-font-family);
    letter-spacing: -0.02em;
  }
`;
