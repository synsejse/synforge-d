import PackageDetail from "./PackageDetail";

export default function PackageDetailLoader() {
  const packageName = typeof window !== "undefined"
    ? new URL(window.location.href).searchParams.get("name") || ""
    : "";

  if (!packageName) {
    return <div className="text-red-500">Invalid package name</div>;
  }

  return <PackageDetail packageName={packageName} />;
}
