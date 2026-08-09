import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import AnsiText from "./ansi-text";

describe("AnsiText", () => {
  it("renders ANSI colors without interpreting log text as HTML", () => {
    const html = renderToStaticMarkup(
      <AnsiText>{"<script>alert(1)</script> \u001b[31mfailed\u001b[0m"}</AnsiText>,
    );

    expect(html).toContain("&lt;script&gt;alert(1)&lt;/script&gt;");
    expect(html).toContain("color:rgb(187, 0, 0)");
    expect(html).not.toContain("<script>");
  });
});
