export async function loadConfigSchema(apiPath) {
    const response = await fetch(apiPath("/api/v1/config/schema"), {
        method: "GET",
        credentials: "include",
    });
    if (!response.ok) {
        throw new Error("Failed to load config schema.");
    }
    return response.json();
}

export function validateSetupConfigFields(setupConfigFields, configSchema) {
    const fields =
        setupConfigFields?.querySelectorAll("[data-setup-config-key]") || [];

    for (const element of fields) {
        if (!(element instanceof HTMLInputElement)) {
            continue;
        }
        const key = element.dataset.setupConfigKey;
        const field = configSchema.find((entry) => entry.key === key);
        if (!field) {
            continue;
        }
        const value = element.value.trim();
        if (field.required && value.length === 0) {
            return `${field.label} is required.`;
        }
        if (
            field.type === "number" &&
            value.length > 0 &&
            Number.isNaN(Number(value))
        ) {
            return `${field.label} must be a valid number.`;
        }
    }

    return null;
}

export function renderSetupConfigFields(setupConfigFields, fields) {
    if (!setupConfigFields) {
        return;
    }
    setupConfigFields.innerHTML = "";
    const sections = groupConfigFields(
        fields.filter((field) => field.editable_in_setup),
    );

    sections.forEach((section) => {
        const container = document.createElement("section");
        container.className = "xl:col-span-2 border-2 border-zinc-700 bg-black p-5";

        const heading = document.createElement("h2");
        heading.className = "font-mono text-lg font-bold uppercase text-white";
        heading.textContent = section.label;

        const grid = document.createElement("div");
        grid.className = "mt-4 grid gap-4 xl:grid-cols-2";

        section.fields.forEach((field) => {
            const label = document.createElement("label");
            label.className = "block";

            const title = document.createElement("span");
            title.className =
                "mb-2 block font-mono text-xs font-bold uppercase tracking-[0.16em] text-zinc-300";
            title.textContent = field.label;

            const input = document.createElement("input");
            input.type = field.type === "number" ? "number" : "text";
            input.value =
                field.key === "public_base_url"
                    ? window.location.origin
                    : String(field.default_value ?? "");
            input.required = Boolean(field.required);
            input.className =
                "w-full border-2 border-zinc-700 bg-black px-4 py-3 font-mono text-sm text-zinc-100 outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)] focus:ring-2 focus:ring-[var(--theme-accent-lime)]";
            input.dataset.setupConfigKey = field.key;
            if (field.min_value !== undefined) {
                input.min = String(field.min_value);
            }

            const help = document.createElement("span");
            help.className = "mt-2 block text-xs text-zinc-500";
            help.textContent = field.description;

            label.appendChild(title);
            label.appendChild(input);
            label.appendChild(help);
            grid.appendChild(label);
        });

        container.appendChild(heading);
        container.appendChild(grid);
        setupConfigFields.appendChild(container);
    });
}

export function collectSetupSettings(setupConfigFields, configSchema) {
    const settings = {};
    setupConfigFields
        ?.querySelectorAll("[data-setup-config-key]")
        .forEach((element) => {
            if (!(element instanceof HTMLInputElement)) {
                return;
            }
            const key = element.dataset.setupConfigKey;
            const field = configSchema.find((entry) => entry.key === key);
            if (!key || !field) {
                return;
            }
            settings[key] =
                field.type === "number"
                    ? Number(element.value)
                    : element.value.trim();
        });
    return settings;
}

function groupConfigFields(fields) {
    const groups = new Map();
    fields.forEach((field) => {
        if (!groups.has(field.section_key)) {
            groups.set(field.section_key, {
                key: field.section_key,
                label: field.section_label,
                fields: [],
            });
        }
        groups.get(field.section_key).fields.push(field);
    });
    return Array.from(groups.values());
}
