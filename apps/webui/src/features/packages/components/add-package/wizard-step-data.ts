export type AddPackageStep = "source" | "targets" | "build" | "review";

export const ADD_PACKAGE_STEPS: Array<{
  value: AddPackageStep;
  label: string;
}> = [
  { value: "source", label: "Source" },
  { value: "targets", label: "Targets" },
  { value: "build", label: "Build" },
  { value: "review", label: "Review" },
];
