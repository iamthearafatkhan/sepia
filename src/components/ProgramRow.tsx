import type { InstalledProgram } from "../App";
import { convertFileSrc } from "@tauri-apps/api/core";

type Props = {
    program: InstalledProgram;
    onUninstall: (program: InstalledProgram) => void;
    busy: boolean;
};

export function ProgramRow({ program, onUninstall, busy }: Props) {
    const initial = program.name.charAt(0).toUpperCase();

    let iconUrl = "";
    if (program.icon_path && !program.icon_path.endsWith(".exe")) {
        iconUrl = convertFileSrc(program.icon_path);
    }

    // Custom Badge Logic
    let badgeColor = "var(--elevated)";
    let badgeText = initial;
    if (program.source === "pip") { badgeColor = "#3776AB"; badgeText = "Py"; }
    if (program.source === "npm") { badgeColor = "#CB3837"; badgeText = "N"; }
    if (program.source === "choco") { badgeColor = "#8B4513"; badgeText = "C"; } // Chocolate brown
    if (program.source === "scoop") { badgeColor = "#2E8B57"; badgeText = "S"; } // Sea green

    const displaySize = program.size_mb > 0
        ? `${program.size_mb.toFixed(1)} MB`
        : "—";

    return (
        <div
            style={{
                display: "grid",
                gridTemplateColumns: "36px 2fr 80px 1.5fr 80px auto",
                gap: "1rem",
                alignItems: "center",
                background: "#4A3B38",
                padding: "0.75rem 1.1rem",
                borderRadius: "var(--radius)",
                border: "1px solid rgba(193, 160, 148, 0.1)",
                transition: "background 0.2s",
            }}
            onMouseEnter={(e) => e.currentTarget.style.background = "#5A4743"}
            onMouseLeave={(e) => e.currentTarget.style.background = "#4A3B38"}
        >
            {/* Icon Container */}
            <div
                style={{
                    width: 36, height: 36, borderRadius: 8,
                    background: badgeColor,
                    display: "flex", alignItems: "center", justifyContent: "center",
                    overflow: "hidden", fontWeight: 700, color: "#fff", fontSize: "0.9rem"
                }}
            >
                {iconUrl ? (
                    <img src={iconUrl} alt="" style={{ width: "100%", height: "100%", objectFit: "contain", background: "#211B1A" }}
                        onError={(e) => { e.currentTarget.style.display = 'none'; }} />
                ) : (
                    <span>{badgeText}</span>
                )}
            </div>

            {/* Name & Publisher */}
            <div style={{ overflow: "hidden" }}>
                <div style={{ fontWeight: 600, color: "var(--text)", whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
                    {program.name}
                </div>
                <div style={{ fontSize: "0.75rem", color: "var(--muted)" }}>
                    {program.registry_view} · {program.publisher || "Unknown"}
                </div>
            </div>

            {/* Version */}
            <div style={{ fontSize: "0.8rem", color: "var(--accent)" }}>
                {program.version || "—"}
            </div>

            {/* Path */}
            <div style={{ fontSize: "0.75rem", color: "var(--muted)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }} title={program.install_location}>
                {program.install_location || "—"}
            </div>

            {/* Size */}
            <div style={{ fontSize: "0.8rem", color: "var(--text)", fontWeight: 500 }}>
                {displaySize}
            </div>

            {/* Uninstall Button */}
            <button
                disabled={busy}
                onClick={() => onUninstall(program)}
                style={{
                    background: "var(--surface)",
                    color: "var(--text)",
                    padding: "0.4rem 0.8rem",
                    fontSize: "0.8rem",
                    border: "1px solid rgba(193, 160, 148, 0.2)",
                    borderRadius: 6,
                    cursor: busy ? "not-allowed" : "pointer",
                    justifySelf: "end"
                }}
            >
                Uninstall
            </button>
        </div>
    );
}