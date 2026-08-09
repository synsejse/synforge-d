import Anser from "anser";
import { escapeCarriageReturn } from "escape-carriage";
import { memo, useMemo, type CSSProperties } from "react";

function applyBackspaces(value: string): string {
  const characters: string[] = [];
  for (const character of value) {
    if (
      character === "\b" &&
      characters.length > 0 &&
      characters.at(-1) !== "\n"
    ) {
      characters.pop();
    } else {
      characters.push(character);
    }
  }
  return characters.join("");
}

function bundleStyle(bundle: Anser.AnserJsonEntry): CSSProperties {
  const style: CSSProperties = {};
  if (bundle.bg) style.backgroundColor = `rgb(${bundle.bg})`;
  if (bundle.fg) style.color = `rgb(${bundle.fg})`;

  const decorations = new Set(bundle.decorations);
  if (decorations.has("bold")) style.fontWeight = "bold";
  if (decorations.has("dim")) style.opacity = 0.5;
  if (decorations.has("italic")) style.fontStyle = "italic";
  if (decorations.has("hidden")) style.visibility = "hidden";

  const textDecorations: string[] = [];
  if (decorations.has("underline")) textDecorations.push("underline");
  if (decorations.has("strikethrough")) textDecorations.push("line-through");
  if (textDecorations.length > 0) {
    style.textDecoration = textDecorations.join(" ");
  }
  return style;
}

const AnsiText = memo(function AnsiText({ children }: { children: string }) {
  const bundles = useMemo(
    () =>
      Anser.ansiToJson(
        escapeCarriageReturn(applyBackspaces(children)),
        { json: true, remove_empty: true },
      ),
    [children],
  );

  return (
    <code>
      {bundles.map((bundle, index) => (
        <span key={index} style={bundleStyle(bundle)}>
          {bundle.content}
        </span>
      ))}
    </code>
  );
});

export default AnsiText;
