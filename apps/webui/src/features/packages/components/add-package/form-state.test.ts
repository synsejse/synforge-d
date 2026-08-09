import { describe, expect, it } from "vitest";
import {
  buildCreatePackageRequest,
  INITIAL_ADD_PACKAGE_FORM,
} from "./form-state";

describe("buildCreatePackageRequest", () => {
  it("does not send a stale cache size when ccache is disabled", () => {
    const request = buildCreatePackageRequest(
      {
        ...INITIAL_ADD_PACKAGE_FORM,
        name: " mesa-git ",
        repoUrl: " https://example.com/mesa.git ",
        specPath: " rpm/mesa.spec ",
        ccacheEnabled: false,
        ccacheMaxSizeMb: "8192",
      },
      16,
    );

    expect(request.name).toBe("mesa-git");
    expect(request.source.repo_url).toBe("https://example.com/mesa.git");
    expect(request.source.spec_file).toBe("rpm/mesa.spec");
    expect(request.ccache_enabled).toBe(false);
    expect(request.ccache_max_size_mb).toBeUndefined();
  });

  it("includes explicitly enabled cache and resource limits", () => {
    const request = buildCreatePackageRequest(
      {
        ...INITIAL_ADD_PACKAGE_FORM,
        ccacheEnabled: true,
        ccacheMaxSizeMb: "4096",
        cpuLimitEnabled: true,
        cpuLimitCores: "6",
        memoryLimitEnabled: true,
        memoryLimitMb: "8192",
      },
      8,
    );

    expect(request.ccache_max_size_mb).toBe(4096);
    expect(request.cpu_limit_millicores).toBe(6000);
    expect(request.memory_limit_mb).toBe(8192);
  });
});
