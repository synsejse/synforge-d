import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import TimeSeriesChart from "./time-series-chart";

describe("TimeSeriesChart", () => {
  it("renders visible totals and an accessible data table", () => {
    const html = renderToStaticMarkup(
      <TimeSeriesChart
        ariaLabel="Build outcomes"
        data={{
          range: "24h",
          bucket_seconds: 3600,
          started_at: "2026-01-01T00:00:00Z",
          points: [
            {
              timestamp: "2026-01-01T00:00:00Z",
              succeeded: 3,
              failed: 1,
            },
            {
              timestamp: "2026-01-01T01:00:00Z",
              succeeded: 2,
              failed: 0,
            },
          ],
        }}
      />,
    );

    expect(html).toContain("5</strong>");
    expect(html).toContain("1</strong>");
    expect(html).toContain("<caption>Build outcomes data</caption>");
    expect(html).toContain("<th scope=\"col\">Succeeded</th>");
  });
});
