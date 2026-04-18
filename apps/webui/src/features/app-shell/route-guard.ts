const publicApiUrl = document.body.dataset.publicApiUrl || "";
const authMode = document.body.dataset.authMode || "protected";
const appShell = document.getElementById("app-shell");

function apiPath(path: string): string {
    return `${publicApiUrl}${path}`;
}

function currentReturnPath(): string {
    return `${window.location.pathname}${window.location.search}${window.location.hash}`;
}

function redirect(path: string): void {
    if (window.location.pathname === path) {
        return;
    }
    window.location.href = path;
}

function redirectToLogin(message?: string): void {
    const params = new URLSearchParams();
    const next = currentReturnPath();
    if (next && next !== "/login/") {
        params.set("next", next);
    }
    if (message) {
        params.set("message", message);
    }
    redirect(`/login/${params.toString() ? `?${params.toString()}` : ""}`);
}

async function requireInitialized(): Promise<boolean> {
    const response = await fetch(apiPath("/api/v1/setup/status"), {
        credentials: "include",
    });
    if (!response.ok) {
        redirectToLogin("Failed to load daemon configuration.");
        return false;
    }
    const status = (await response.json()) as { initialized: boolean };
    if (!status.initialized) {
        redirect("/setup/");
        return false;
    }
    return true;
}

async function requireSession(): Promise<void> {
    const response = await fetch(apiPath("/api/v1/session"), {
        credentials: "include",
    });
    if (!response.ok) {
        redirectToLogin("Use a Synforge account to access the console.");
        return;
    }
    appShell?.classList.remove("hidden");
}

window.addEventListener("synforge:auth-required", () => {
    redirectToLogin("The daemon rejected the current session.");
});

if (authMode === "protected") {
    void requireInitialized().then((initialized) => {
        if (initialized) {
            void requireSession();
        }
    });
}
