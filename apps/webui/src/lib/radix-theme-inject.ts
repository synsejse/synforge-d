import { radixCustomStyles } from "./radix-theme";

const STYLE_ID = "synforge-radix-theme";

if (typeof document !== "undefined" && !document.getElementById(STYLE_ID)) {
  const style = document.createElement("style");
  style.id = STYLE_ID;
  style.textContent = radixCustomStyles;
  document.head.appendChild(style);
}
