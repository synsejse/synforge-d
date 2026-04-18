import {
    collectSetupSettings,
    loadConfigSchema,
    renderSetupConfigFields,
    validateSetupConfigFields,
} from "./setup-config";
import { createSetupSigningController } from "./setup-signing";
import type {
    ConfigFieldDescriptor,
    SetupInitializeRequest,
    SetupStatusResponse,
} from "../../lib/types";

type SetupControllerOptions = {
    apiPath: (path: string) => string;
    showAuthScreen: (message?: string) => void;
};

type SetupStep = "config" | "signing" | "admin";

type SetupPayloadFields = {
    adminHandle: string;
    adminDisplayName: string;
    adminPassword: string;
    adminPasswordConfirm: string;
};

type ApiErrorBody = {
    message?: string;
};

export type SetupController = {
    loadInitialState: () => Promise<SetupStatusResponse>;
    showSetupScreen: (message?: string) => void;
};

export function createSetupController({
    apiPath,
    showAuthScreen,
}: SetupControllerOptions): SetupController {
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

    let configSchema: ConfigFieldDescriptor[] = [];
    let setupStep: SetupStep = "config";

    function clearSetupError(): void {
        if (setupError) {
            setupError.classList.add("hidden");
            setupError.textContent = "";
        }
    }

    function setSetupError(message: string): void {
        if (setupError) {
            setupError.textContent = message;
            setupError.classList.remove("hidden");
        }
    }

    function setSetupStep(step: SetupStep): void {
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

    function showSetupScreen(message?: string): void {
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

    async function loadSetupStatus(): Promise<SetupStatusResponse> {
        const response = await fetch(apiPath("/api/v1/setup/status"), {
            method: "GET",
        });
        if (!response.ok) {
            return { initialized: true };
        }
        return response.json();
    }

    function focusAdminHandle(): void {
        const handleInput = document.getElementById("setup-admin-handle");
        if (handleInput instanceof HTMLInputElement) {
            handleInput.focus();
        }
    }

    function readSetupPayload(): SetupPayloadFields {
        const inputValue = (id: string): string => {
            const element = document.getElementById(id);
            return element instanceof HTMLInputElement ? element.value : "";
        };

        return {
            adminHandle: inputValue("setup-admin-handle").trim(),
            adminDisplayName: inputValue("setup-admin-display-name").trim(),
            adminPassword: inputValue("setup-admin-password"),
            adminPasswordConfirm: inputValue("setup-admin-password-confirm"),
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
        const payload: SetupInitializeRequest = {
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
                const error = (await response.json().catch(() => ({
                    message: "Setup failed.",
                }))) as ApiErrorBody;
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
