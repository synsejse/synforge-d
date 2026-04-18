type AuthScreenOptions = {
    apiPath: (path: string) => string;
    onAuthenticated: () => void;
    showAuthScreen?: (message?: string) => void;
};

type ApiErrorBody = {
    message?: string;
};

export function initAuthScreen({
    apiPath,
    onAuthenticated,
    showAuthScreen,
}: AuthScreenOptions) {
    const authForm = document.getElementById("auth-form");
    const authHandle = document.getElementById("auth-handle");
    const authPassword = document.getElementById("auth-password");
    const authError = document.getElementById("auth-error");

    function setAuthError(message: string) {
        if (!authError) {
            return;
        }
        authError.textContent = message;
        authError.classList.remove("hidden");
    }

    authForm?.addEventListener("submit", async (event) => {
        event.preventDefault();
        const handle =
            authHandle instanceof HTMLInputElement
                ? authHandle.value.trim()
                : "";
        const password =
            authPassword instanceof HTMLInputElement ? authPassword.value : "";

        if (!handle || !password) {
            setAuthError("Handle and password are required.");
            return;
        }

        try {
            const response = await fetch(apiPath("/api/v1/session/login"), {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                credentials: "include",
                body: JSON.stringify({ handle, password }),
            });
            if (!response.ok) {
                const error = (await response.json().catch(() => ({
                    message: "Login failed.",
                }))) as ApiErrorBody;
                setAuthError(error?.message || "Login failed.");
                return;
            }
            onAuthenticated();
        } catch {
            setAuthError("Login failed.");
        }
    });

    window.addEventListener("synforge:auth-required", (event) => {
        const detail = event instanceof CustomEvent ? event.detail : null;
        const message = getAuthRequiredMessage(detail);
        if (showAuthScreen) {
            showAuthScreen(message);
            return;
        }
        const params = new URLSearchParams({ message });
        window.location.href = `/login/?${params.toString()}`;
    });
}

function getAuthRequiredMessage(detail: unknown): string {
    if (
        typeof detail === "object" &&
        detail !== null &&
        "error" in detail &&
        typeof detail.error === "object" &&
        detail.error !== null &&
        "message" in detail.error &&
        typeof detail.error.message === "string"
    ) {
        return detail.error.message;
    }
    return "The daemon rejected the current session.";
}
