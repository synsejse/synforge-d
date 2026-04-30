import { createFileRoute } from "@tanstack/react-router";
import PackageListPage from "../../../features/packages/PackageListPage";

export const Route = createFileRoute("/_authed/packages/")({
  component: PackageListPage,
});
