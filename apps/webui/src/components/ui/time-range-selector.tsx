import type { TimeRange } from "../../lib/types";
import SegmentedControl from "./segmented-control";

interface Props {
  value: TimeRange;
  onChange: (next: TimeRange) => void;
  className?: string;
}

const ITEMS = [
  { value: "24h" as const, label: "24h" },
  { value: "7d" as const, label: "7d" },
  { value: "30d" as const, label: "30d" },
];

export default function TimeRangeSelector({ value, onChange, className }: Props) {
  return (
    <SegmentedControl
      value={value}
      onChange={onChange}
      items={ITEMS}
      ariaLabel="Time range"
      size="md"
      className={className}
    />
  );
}
