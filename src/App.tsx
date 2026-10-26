import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ProgramRow } from "./components/ProgramRow";

export type InstalledProgram = {
    name: string;
    version: string;
    publisher: string;
    install_location: string;
    uninstall_string: string;
    registry_view: string;
    icon_path: string;
    source: string;
    size_mb: number;
};

export default function App() {
    const [programs, setPrograms] = useState<InstalledProgram[]>([]);
    const [loading, setLoading] = useState(true);
    const [busyName, setBusyName] = useState<string | null>(null);
    const [message, setMessage] = useState<string | null>(null);
    const [searchQuery, setSearchQuery] = useState("");

    async function refresh() {
        setLoading(true);
        try {
            const list = await invoke<InstalledProgram[]>("list_programs");
            setPrograms(list);
        } catch (err) {
            setMessage(`Scan failed: ${err}`);
        } finally {
            setLoading(false);
        }
    }

    useEffect(() => {
        refresh();
    }, []);

    async function handleUninstall(program: InstalledProgram) {
        if (!window.confirm(`Uninstall "${program.name}"?`)) return;
        setBusyName(program.name);
        try {
            const result = await invoke<string>("uninstall_program", { program });
            setMessage(result);
            await refresh();
        } catch (err) {
            setMessage(`Uninstall failed: ${err}`);
        } finally {
            setBusyName(null);
        }
    }

    const filteredPrograms = programs.filter((p) =>
        p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        p.publisher.toLowerCase().includes(searchQuery.toLowerCase())
    );

    return (
        <div style={{ maxWidth: 1200, margin: "0 auto", padding: "2.5rem 1.5rem" }}>

            {/* Header */}
            <header style={{ marginBottom: "1.5rem", display: "flex", justifyContent: "space-between", alignItems: "flex-end" }}>
                <div>
                    <h1 style={{ fontSize: "1.6rem", fontWeight: 700, margin: 0, color: "var(--text)" }}>Sepia</h1>
                    <p style={{ margin: "0.3rem 0 0", color: "var(--muted)", fontSize: "0.9rem" }}>Your software, curated.</p>
                </div>
                <button onClick={refresh} disabled={loading} style={{ background: "var(--surface)", color: "var(--accent)", padding: "0.6rem 1rem", borderRadius: 8, border: "1px solid rgba(193, 160, 148, 0.25)", cursor: loading ? "not-allowed" : "pointer" }}>
                    {loading ? "Scanning..." : "Rescan"}
                </button>
            </header>

            {/* Search Bar */}
            <div style={{ marginBottom: "1.5rem" }}>
                <input
                    type="text"
                    placeholder="Search software, packages, or publishers..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    style={{ width: "100%", background: "var(--surface)", color: "var(--text)", border: "1px solid rgba(145, 107, 96, 0.3)", padding: "0.8rem 1rem", borderRadius: "10px", fontSize: "0.95rem", outline: "none" }}
                />
            </div>

            {/* Status Message */}
            {message && (
                <div style={{ background: "var(--surface)", borderLeft: "3px solid var(--accent)", padding: "0.75rem 1rem", borderRadius: 8, marginBottom: "1.25rem", fontSize: "0.85rem" }}>
                    {message}
                </div>
            )}

            {/* List Area */}
            {loading ? (
                <div style={{ color: "var(--muted)", textAlign: "center", padding: "3rem 0" }}>
                    Calculating sizes and scanning packages...
                </div>
            ) : (
                <>
                    {/* Column Headers */}
                    <div style={{ display: "grid", gridTemplateColumns: "36px 2fr 80px 1.5fr 80px auto", gap: "1rem", padding: "0 1.1rem", marginBottom: "0.5rem", color: "var(--muted)", fontSize: "0.75rem", fontWeight: 600, textTransform: "uppercase", letterSpacing: "0.05em" }}>
                        <div></div>
                        <div>Name</div>
                        <div>Version</div>
                        <div>Path</div>
                        <div>Size</div>
                        <div style={{ textAlign: "right", paddingRight: "0.5rem" }}>Action</div>
                    </div>

                    {/* List */}
                    <div style={{ display: "flex", flexDirection: "column", gap: "0.5rem" }}>
                        {filteredPrograms.map((program) => (
                            <ProgramRow
                                key={`${program.source}-${program.name}-${program.version}`}
                                program={program}
                                onUninstall={handleUninstall}
                                busy={busyName === program.name}
                            />
                        ))}
                        {filteredPrograms.length === 0 && (
                            <div style={{ color: "var(--muted)", textAlign: "center", padding: "2rem" }}>
                                No software found matching "{searchQuery}"
                            </div>
                        )}
                    </div>
                </>
            )}

            {/* Footer */}
            <footer style={{
                marginTop: "3rem",
                paddingTop: "1.25rem",
                borderTop: "1px solid rgba(145, 107, 96, 0.2)",
                color: "var(--muted)",
                fontSize: "0.75rem",
                fontWeight: 500,
                textAlign: "center"
            }}>
                Built by{" "}
                <span
                    onClick={() => invoke("open_url", { url: "https://github.com/iamthearafatkhan" })}
                    style={{
                        color: "var(--accent)",
                        textDecoration: "none",
                        fontWeight: 600,
                        cursor: "pointer", // Makes it look like a link
                        transition: "color 0.2s ease"
                    }}
                    onMouseEnter={(e) => e.currentTarget.style.color = "var(--text)"}
                    onMouseLeave={(e) => e.currentTarget.style.color = "var(--accent)"}
                >
                    Arafat Khan
                </span>
            </footer>
        </div>
    );
}