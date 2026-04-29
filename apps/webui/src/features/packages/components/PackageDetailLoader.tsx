import PackageDetail from "./PackageDetail";
import PageRoot from "../../../components/common/PageRoot";
import ServerHardwareProvider from "../../../components/common/ServerHardwareProvider";

function PackageDetailLoaderInner() {
  const packageName = typeof window !== "undefined"
    ? new URL(window.location.href).searchParams.get("name") || ""
    : "";

  if (!packageName) {
    return (
      <div className="border-2 border-[var(--theme-error-red)] bg-black px-4 py-3 font-mono text-sm text-[var(--theme-error-red)]">
        Invalid package name
      </div>
    );
  }

  return <PackageDetail packageName={packageName} />;
}

export default function PackageDetailLoader() {
  return (
    <PageRoot>
      <ServerHardwareProvider>
        <PackageDetailLoaderInner />
      </ServerHardwareProvider>
    </PageRoot>
  );
}
