export function createSetupSigningController({
    toggleButton,
    stateNote,
    generateButton,
    importButton,
    importFileInput,
    keyNote,
    clearSetupError,
    setSetupError,
}) {
    let enabled = true;
    let mode = "generate";
    let importedPrivateKey = "";
    let importedFilename = "";

    function updateUi() {
        const disabledButtonClass =
            "border-zinc-700 bg-black text-zinc-200 hover:border-white hover:bg-zinc-950";
        const activeButtonClass =
            "border-[var(--theme-accent-lime)] bg-[var(--theme-accent-lime)] text-black hover:bg-[#d8ff72]";
        const buttonBaseClass =
            "border-2 px-4 py-3 font-mono text-xs font-bold uppercase tracking-[0.16em] transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px]";

        if (toggleButton instanceof HTMLButtonElement) {
            if (enabled) {
                toggleButton.textContent = "Disable Signing";
                toggleButton.className = `${buttonBaseClass} ${activeButtonClass}`;
            } else {
                toggleButton.textContent = "Enable Signing";
                toggleButton.className = `${buttonBaseClass} ${disabledButtonClass}`;
            }
        }

        if (stateNote) {
            stateNote.textContent = enabled
                ? "Signing will be enabled after initialization."
                : "Signing will stay disabled after initialization.";
        }

        if (generateButton instanceof HTMLButtonElement) {
            generateButton.disabled = !enabled;
            generateButton.className = `${buttonBaseClass} ${
                enabled && mode === "generate"
                    ? activeButtonClass
                    : disabledButtonClass
            }`;
        }

        if (importButton instanceof HTMLButtonElement) {
            importButton.disabled = !enabled;
            importButton.className = `${buttonBaseClass} ${
                enabled && mode === "import"
                    ? activeButtonClass
                    : disabledButtonClass
            }`;
        }

        if (keyNote) {
            if (!enabled) {
                keyNote.textContent =
                    "Key actions are disabled while signing is disabled.";
            } else if (mode === "import") {
                keyNote.textContent = importedFilename
                    ? `Import selected: ${importedFilename}`
                    : "Import mode selected. Choose a private key file.";
            } else {
                keyNote.textContent = "Managed key generation is selected.";
            }
        }
    }

    function validate() {
        if (!enabled) {
            return null;
        }
        if (mode === "import" && importedPrivateKey.trim().length === 0) {
            return "Import mode selected but no key file has been loaded.";
        }
        return null;
    }

    function reset() {
        enabled = true;
        mode = "generate";
        importedPrivateKey = "";
        importedFilename = "";
        if (importFileInput instanceof HTMLInputElement) {
            importFileInput.value = "";
        }
        updateUi();
    }

    toggleButton?.addEventListener("click", () => {
        clearSetupError();
        enabled = !enabled;
        updateUi();
    });

    generateButton?.addEventListener("click", () => {
        clearSetupError();
        if (!enabled) {
            return;
        }
        mode = "generate";
        importedPrivateKey = "";
        importedFilename = "";
        if (importFileInput instanceof HTMLInputElement) {
            importFileInput.value = "";
        }
        updateUi();
    });

    importButton?.addEventListener("click", () => {
        clearSetupError();
        if (!enabled) {
            return;
        }
        mode = "import";
        updateUi();
        if (importFileInput instanceof HTMLInputElement) {
            importFileInput.click();
        }
    });

    importFileInput?.addEventListener("change", async (event) => {
        clearSetupError();
        const target = event.target;
        if (!(target instanceof HTMLInputElement)) {
            return;
        }
        const file = target.files?.[0];
        if (!file) {
            return;
        }
        try {
            mode = "import";
            importedPrivateKey = await file.text();
            importedFilename = file.name;
            updateUi();
        } catch {
            importedPrivateKey = "";
            importedFilename = "";
            setSetupError("Failed to read signing key file.");
            updateUi();
        }
    });

    return {
        reset,
        validate,
        payload() {
            return {
                enableSigning: enabled,
                signingArmoredPrivateKey:
                    enabled && mode === "import"
                        ? importedPrivateKey.trim()
                        : null,
            };
        },
    };
}
