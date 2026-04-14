export function initAuthScreen({ apiPath, showAppShell, showAuthScreen }) {
    const authForm = document.getElementById("auth-form");
    const authHandle = document.getElementById("auth-handle");
    const authPassword = document.getElementById("auth-password");
    const authError = document.getElementById("auth-error");

    function setAuthError(message) {
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
                const error = await response.json().catch(() => ({
                    message: "Login failed.",
                }));
                setAuthError(error?.message || "Login failed.");
                return;
            }
            showAppShell();
            window.location.reload();
        } catch {
            setAuthError("Login failed.");
        }
    });

    window.addEventListener("synforge:auth-required", (event) => {
        const detail = event instanceof CustomEvent ? event.detail : null;
        const message =
            detail?.error?.message || "The daemon rejected the current session.";
        showAuthScreen(message);
    });
}
