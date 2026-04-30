import { useSearch } from "@tanstack/react-router";
import PackageDetail from "./components/PackageDetail";

export default function PackageDetailPage() {
  const search = useSearch({ from: "/_authed/packages/view" });
  const packageName = search.name;

  if (!packageName) {
    return (
      <div className="border-2 border-[var(--theme-error-red)] bg-black px-4 py-3 font-mono text-sm text-[var(--theme-error-red)]">
        Invalid package name
      </div>
    );
  }

  return <PackageDetail packageName={packageName} />;
}
