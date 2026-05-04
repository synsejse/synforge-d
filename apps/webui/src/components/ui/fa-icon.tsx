import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";

interface Props {
  icon: IconDefinition;
  className?: string;
}

export default function FaIcon({ icon, className }: Props) {
  const classes = [
    "inline-block h-[1em] w-[1.25em] shrink-0 align-[-0.125em]",
    className,
  ]
    .filter(Boolean)
    .join(" ");

  return <FontAwesomeIcon icon={icon} className={classes} />;
}
