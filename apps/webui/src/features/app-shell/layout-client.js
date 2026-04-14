import { initAuthScreen } from "../auth/auth-client";
import { createSetupController } from "../setup/setup-client";

const publicApiUrl = document.body.dataset.publicApiUrl || "";

const authScreen = document.getElementById("auth-screen");
const setupScreen = document.getElementById("setup-screen");
const appShell = document.getElementById("app-shell");
const authHandle = document.getElementById("auth-handle");
const authPassword = document.getElementById("auth-password");
const authError = document.getElementById("auth-error");
const authMessage = document.getElementById("auth-message");
const logoutButton = document.getElementById("logout-button");
const mobileNavToggleButton = document.getElementById("mobile-nav-toggle");
const mobileNavPanel = document.getElementById("mobile-nav-panel");

let mobileNavOpen = false;

function apiPath(path) {
    return `${publicApiUrl}${path}`;
}

function isDesktopNavViewport() {
    return window.matchMedia("(min-width: 1280px)").matches;
}

function setMobileNavOpen(nextOpen) {
    mobileNavOpen = nextOpen;
    if (!(mobileNavPanel instanceof HTMLElement)) {
        return;
    }
    if (isDesktopNavViewport()) {
        mobileNavPanel.classList.remove("hidden");
        if (mobileNavToggleButton instanceof HTMLButtonElement) {
            mobileNavToggleButton.textContent = "Menu";
            mobileNavToggleButton.setAttribute("aria-expanded", "false");
        }
        return;
    }
    mobileNavPanel.classList.toggle("hidden", !mobileNavOpen);
    if (mobileNavToggleButton instanceof HTMLButtonElement) {
        mobileNavToggleButton.textContent = mobileNavOpen ? "Close" : "Menu";
        mobileNavToggleButton.setAttribute(
            "aria-expanded",
            mobileNavOpen ? "true" : "false",
        );
    }
}

function showAuthScreen(message) {
    if (authMessage && message) {
        authMessage.textContent = message;
    }
    if (authError) {
        authError.classList.add("hidden");
        authError.textContent = "";
    }
    if (authHandle instanceof HTMLInputElement) {
        authHandle.focus();
    }
    if (authPassword instanceof HTMLInputElement) {
        authPassword.value = "";
    }
    authScreen?.classList.remove("hidden");
    authScreen?.classList.add("flex");
    setupScreen?.classList.add("hidden");
    setupScreen?.classList.remove("flex");
    appShell?.classList.add("hidden");
}

function showSetupScreen(message) {
    authScreen?.classList.add("hidden");
    authScreen?.classList.remove("flex");
    setupScreen?.classList.remove("hidden");
    setupScreen?.classList.add("flex");
    appShell?.classList.add("hidden");
    setup.showSetupScreen(message);
}

function showAppShell() {
    authScreen?.classList.add("hidden");
    authScreen?.classList.remove("flex");
    setupScreen?.classList.add("hidden");
    setupScreen?.classList.remove("flex");
    appShell?.classList.remove("hidden");
    setMobileNavOpen(false);
}

async function restoreSession() {
    const response = await fetch(apiPath("/api/v1/session"), {
        method: "GET",
        credentials: "include",
    });
    if (response.ok) {
        showAppShell();
        return true;
    }
    return false;
}

const setup = createSetupController({ apiPath, showAuthScreen });

initAuthScreen({ apiPath, showAppShell, showAuthScreen });

logoutButton?.addEventListener("click", async () => {
    try {
        await fetch(apiPath("/api/v1/session/logout"), {
            method: "POST",
            credentials: "include",
        });
    } finally {
        showAuthScreen("Enter account credentials to continue.");
    }
});

mobileNavToggleButton?.addEventListener("click", () => {
    setMobileNavOpen(!mobileNavOpen);
});

mobileNavPanel?.addEventListener("click", (event) => {
    if (isDesktopNavViewport()) {
        return;
    }
    const target = event.target;
    if (!(target instanceof Element)) {
        return;
    }
    if (target.closest("a[href]")) {
        setMobileNavOpen(false);
    }
});

window.addEventListener("resize", () => {
    setMobileNavOpen(mobileNavOpen);
});

setup
    .loadInitialState()
    .then((status) => {
        if (!status.initialized) {
            showSetupScreen(
                "Configure daemon settings and create the initial admin account.",
            );
            return;
        }
        restoreSession().then((authenticated) => {
            if (!authenticated) {
                showAuthScreen("Use a Synforge account to access the console.");
            }
        });
    })
    .catch(() => {
        showAuthScreen("Failed to load daemon configuration.");
    });
