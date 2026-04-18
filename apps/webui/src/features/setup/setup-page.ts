import { createSetupController } from "./setup-client";

const publicApiUrl = document.body.dataset.publicApiUrl || "";

function apiPath(path: string): string {
    return `${publicApiUrl}${path}`;
}

function redirectToLogin(message?: string): void {
    const params = new URLSearchParams();
    if (message) {
        params.set("message", message);
    }
    window.location.href = `/login/${params.toString() ? `?${params.toString()}` : ""}`;
}

const setup = createSetupController({
    apiPath,
    showAuthScreen: redirectToLogin,
});

setup
    .loadInitialState()
    .then((status) => {
        if (status.initialized) {
            redirectToLogin("Synforge is already initialized. Sign in to continue.");
            return;
        }
        setup.showSetupScreen(
            "Configure daemon settings and create the initial admin account.",
        );
    })
    .catch(() => {
        redirectToLogin("Failed to load daemon configuration.");
    });
