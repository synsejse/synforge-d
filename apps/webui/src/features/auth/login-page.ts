import { initAuthScreen } from "./auth-client";

const publicApiUrl = document.body.dataset.publicApiUrl || "";
const authMessage = document.getElementById("auth-message");

function apiPath(path: string): string {
    return `${publicApiUrl}${path}`;
}

function nextPath(): string {
    const params = new URLSearchParams(window.location.search);
    const next = params.get("next");
    if (next?.startsWith("/") && !next.startsWith("//")) {
        return next;
    }
    return "/";
}

async function redirectIfSetupRequired(): Promise<boolean> {
    const response = await fetch(apiPath("/api/v1/setup/status"), {
        credentials: "include",
    });
    if (!response.ok) {
        return false;
    }
    const status = (await response.json()) as { initialized: boolean };
    if (!status.initialized) {
        window.location.href = "/setup/";
        return true;
    }
    return false;
}

async function redirectIfAlreadyAuthenticated(): Promise<void> {
    const response = await fetch(apiPath("/api/v1/session"), {
        credentials: "include",
    });
    if (response.ok) {
        window.location.href = nextPath();
    }
}

const message = new URLSearchParams(window.location.search).get("message");
if (authMessage && message) {
    authMessage.textContent = message;
}

initAuthScreen({
    apiPath,
    onAuthenticated: () => {
        window.location.href = nextPath();
    },
});

void redirectIfSetupRequired().then((redirected) => {
    if (!redirected) {
        void redirectIfAlreadyAuthenticated();
    }
});
