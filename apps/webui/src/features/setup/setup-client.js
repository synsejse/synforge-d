import {
    collectSetupSettings,
    loadConfigSchema,
    renderSetupConfigFields,
    validateSetupConfigFields,
} from "./setup-config";
import { createSetupSigningController } from "./setup-signing";

export function createSetupController({ apiPath, showAuthScreen }) {
    const setupScreen = document.getElementById("setup-screen");
    const setupForm = document.getElementById("setup-form");
    const setupMessage = document.getElementById("setup-message");
    const setupStepLabel = document.getElementById("setup-step-label");
    const setupError = document.getElementById("setup-error");
    const setupConfigFields = document.getElementById("setup-config-fields");
    const setupStepConfig = document.getElementById("setup-step-config");
    const setupStepAdmin = document.getElementById("setup-step-admin");
    const setupStepSigning = document.getElementById("setup-step-signing");
    const setupBackButton = document.getElementById("setup-back-button");
    const setupNextButton = document.getElementById("setup-next-button");
    const setupSubmitButton = document.getElementById("setup-submit-button");

    let configSchema = [];
    let setupStep = "config";

    function clearSetupError() {
        if (setupError) {
            setupError.classList.add("hidden");
            setupError.textContent = "";
        }
    }

    function setSetupError(message) {
        if (setupError) {
            setupError.textContent = message;
            setupError.classList.remove("hidden");
        }
    }

    function setSetupStep(step) {
        if (step !== "config" && step !== "signing" && step !== "admin") {
            setupStep = "config";
        } else {
            setupStep = step;
        }

        const inConfigStep = setupStep === "config";
        const inSigningStep = setupStep === "signing";
        const inAdminStep = setupStep === "admin";

        setupStepConfig?.classList.toggle("hidden", !inConfigStep);
        setupStepSigning?.classList.toggle("hidden", !inSigningStep);
        setupStepAdmin?.classList.toggle("hidden", !inAdminStep);
        setupBackButton?.classList.toggle("hidden", inConfigStep);
        setupNextButton?.classList.toggle("hidden", inAdminStep);
        setupSubmitButton?.classList.toggle("hidden", !inAdminStep);

        if (setupStepLabel) {
            if (inConfigStep) {
                setupStepLabel.textContent = "Step 1 of 3 · Configuration";
            } else if (inSigningStep) {
                setupStepLabel.textContent = "Step 2 of 3 · Signing";
            } else {
                setupStepLabel.textContent = "Step 3 of 3 · First account";
            }
        }

        if (setupMessage) {
            if (inConfigStep) {
                setupMessage.textContent =
                    "Configure daemon settings for first run.";
            } else if (inSigningStep) {
                setupMessage.textContent =
                    "Choose whether to enable managed repository signing.";
            } else {
                setupMessage.textContent = "Create the first admin account.";
            }
        }
    }

    const signing = createSetupSigningController({
        toggleButton: document.getElementById("setup-signing-toggle-button"),
        stateNote: document.getElementById("setup-signing-state-note"),
        generateButton: document.getElementById("setup-signing-generate-button"),
        importButton: document.getElementById("setup-signing-import-button"),
        importFileInput: document.getElementById("setup-signing-import-file"),
        keyNote: document.getElementById("setup-signing-key-note"),
        clearSetupError,
        setSetupError,
    });

    function showSetupScreen(message) {
        clearSetupError();
        setupScreen?.classList.remove("hidden");
        setupScreen?.classList.add("flex");
        setSetupStep("config");
        if (setupMessage && message) {
            setupMessage.textContent = message;
        }
        const adminPassword = document.getElementById("setup-admin-password");
        if (adminPassword instanceof HTMLInputElement) {
            adminPassword.value = "";
        }
        const adminPasswordConfirm = document.getElementById(
            "setup-admin-password-confirm",
        );
        if (adminPasswordConfirm instanceof HTMLInputElement) {
            adminPasswordConfirm.value = "";
        }
        signing.reset();
    }

    async function loadSetupStatus() {
        const response = await fetch(apiPath("/api/v1/setup/status"), {
            method: "GET",
        });
        if (!response.ok) {
            return { initialized: true };
        }
        return response.json();
    }

    function focusAdminHandle() {
        const handleInput = document.getElementById("setup-admin-handle");
        if (handleInput instanceof HTMLInputElement) {
            handleInput.focus();
        }
    }

    function readSetupPayload() {
        const input = (id) => document.getElementById(id);

        return {
            adminHandle:
                input("setup-admin-handle") instanceof HTMLInputElement
                    ? input("setup-admin-handle").value.trim()
                    : "",
            adminDisplayName:
                input("setup-admin-display-name") instanceof HTMLInputElement
                    ? input("setup-admin-display-name").value.trim()
                    : "",
            adminPassword:
                input("setup-admin-password") instanceof HTMLInputElement
                    ? input("setup-admin-password").value
                    : "",
            adminPasswordConfirm:
                input("setup-admin-password-confirm") instanceof HTMLInputElement
                    ? input("setup-admin-password-confirm").value
                    : "",
        };
    }

    setupNextButton?.addEventListener("click", () => {
        clearSetupError();
        if (setupStep === "config") {
            const validationError = validateSetupConfigFields(
                setupConfigFields,
                configSchema,
            );
            if (validationError) {
                setSetupError(validationError);
                return;
            }
            setSetupStep("signing");
            return;
        }
        if (setupStep !== "signing") {
            setSetupStep("config");
            return;
        }
        const signingValidationError = signing.validate();
        if (signingValidationError) {
            setSetupError(signingValidationError);
            return;
        }
        setSetupStep("admin");
        focusAdminHandle();
    });

    setupBackButton?.addEventListener("click", () => {
        clearSetupError();
        if (setupStep === "admin") {
            setSetupStep("signing");
            return;
        }
        setSetupStep("config");
    });

    setupForm?.addEventListener("submit", async (event) => {
        event.preventDefault();
        clearSetupError();

        if (setupStep !== "admin") {
            if (setupStep === "config") {
                const validationError = validateSetupConfigFields(
                    setupConfigFields,
                    configSchema,
                );
                if (validationError) {
                    setSetupError(validationError);
                    return;
                }
                setSetupStep("signing");
                return;
            }
            if (setupStep !== "signing") {
                setSetupStep("config");
                return;
            }
            const signingValidationError = signing.validate();
            if (signingValidationError) {
                setSetupError(signingValidationError);
                return;
            }
            setSetupStep("admin");
            focusAdminHandle();
            return;
        }

        const {
            adminHandle,
            adminDisplayName,
            adminPassword,
            adminPasswordConfirm,
        } = readSetupPayload();

        if (!adminHandle || !adminDisplayName || !adminPassword) {
            setSetupError(
                "Admin handle, display name, and password are required.",
            );
            return;
        }
        if (adminPassword !== adminPasswordConfirm) {
            setSetupError("Password confirmation does not match.");
            return;
        }

        const { enableSigning, signingArmoredPrivateKey } = signing.payload();
        const payload = {
            settings: collectSetupSettings(setupConfigFields, configSchema),
            enable_signing: enableSigning,
            signing_armored_private_key: signingArmoredPrivateKey,
            admin: {
                handle: adminHandle,
                display_name: adminDisplayName,
                password: adminPassword,
            },
        };

        try {
            const response = await fetch(apiPath("/api/v1/setup/initialize"), {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify(payload),
            });
            if (!response.ok) {
                const error = await response.json().catch(() => ({
                    message: "Setup failed.",
                }));
                setSetupError(error?.message || "Setup failed.");
                return;
            }
            showAuthScreen(
                "Setup complete. Sign in with the admin account you just created.",
            );
        } catch {
            setSetupError("Setup failed.");
        }
    });

    return {
        async loadInitialState() {
            const [status, schema] = await Promise.all([
                loadSetupStatus(),
                loadConfigSchema(apiPath),
            ]);
            configSchema = schema.fields || [];
            renderSetupConfigFields(setupConfigFields, configSchema);
            return status;
        },
        showSetupScreen,
    };
}
