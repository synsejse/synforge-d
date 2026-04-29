import { apiPath } from "../../lib/api/script-helpers";

const logoutButton = document.getElementById("logout-button");
const mobileNavToggleButton = document.getElementById("mobile-nav-toggle");
const mobileNavPanel = document.getElementById("mobile-nav-panel");

let mobileNavOpen = false;

function isDesktopNavViewport(): boolean {
    return window.matchMedia("(min-width: 1280px)").matches;
}

function setMobileNavOpen(nextOpen: boolean): void {
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

logoutButton?.addEventListener("click", async () => {
    try {
        await fetch(apiPath("/api/v1/session/logout"), {
            method: "POST",
            credentials: "include",
        });
    } finally {
        const params = new URLSearchParams({
            message: "Enter account credentials to continue.",
        });
        window.location.href = `/login/?${params.toString()}`;
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

setMobileNavOpen(false);

export {};
