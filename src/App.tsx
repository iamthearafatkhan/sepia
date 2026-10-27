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

export type JunkCategory = {
    id: string;
    name: string;
    size_mb: number;
    path: string;
};

export default function App() {
    const [activeTab, setActiveTab] = useState<"software" | "cleaner">("software");

    // Software State
    const [programs, setPrograms] = useState<InstalledProgram[]>([]);
    const [loading, setLoading] = useState(true);
    const [busyName, setBusyName] = useState<string | null>(null);
    const [message, setMessage] = useState<string | null>(null);
    const [searchQuery, setSearchQuery] = useState("");
    const [confirmTarget, setConfirmTarget] = useState<InstalledProgram | null>(null);

    // Cleaner State
    const [junk, setJunk] = useState<JunkCategory[]>([]);
    const [scanningJunk, setScanningJunk] = useState(false);
    const [cleaningId, setCleaningId] = useState<string | null>(null);

    async function refreshPrograms() {
        setLoading(true);
        try {
            const list = await invoke<InstalledProgram[]>("list_programs");
            setPrograms(list);
        } catch (err) { setMessage(`Scan failed: ${err}`); }
        finally { setLoading(false); }
    }

    async function scanJunk() {
        setScanningJunk(true);
        try {
            const list = await invoke<JunkCategory[]>("scan_junk");
            setJunk(list);
        } catch (err) { setMessage(`Junk scan failed: ${err}`); }
        finally { setScanningJunk(false); }
    }

    useEffect(() => {
        if (activeTab === "software" && programs.length === 0) refreshPrograms();
        if (activeTab === "cleaner" && junk.length === 0) scanJunk();
    }, [activeTab]);

    async function executeUninstall() {
        if (!confirmTarget) return;
        const target = confirmTarget;
        setConfirmTarget(null);
        setBusyName(target.name);
        try {
            const result = await invoke<string>("uninstall_program", { program: target });
            setMessage(result);
            setPrograms((prev) => prev.filter((p) => !(p.name === target.name && p.source === target.source)));
        } catch (err) { setMessage(`Uninstall failed: ${err}`); }
        finally { setBusyName(null); }
    }

    async function handleCleanJunk(id: string) {
        setCleaningId(id);
        try {
            const result = await invoke<string>("clean_junk", { id });
            setMessage(result);
            await scanJunk(); // Rescan sizes after cleaning
        } catch (err) {
            setMessage(`Clean failed: ${err}`);
        } finally {
            setCleaningId(null);
        }
    }

    const filteredPrograms = programs.filter((p) =>
        p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        p.publisher.toLowerCase().includes(searchQuery.toLowerCase())
    );

    const totalJunk = junk.reduce((acc, curr) => acc + curr.size_mb, 0);

    return (
        <div style={{ maxWidth: 1200, margin: "0 auto", padding: "2.5rem 1.5rem" }}>

            {/* Header */}
            <header style={{ marginBottom: "1.5rem", display: "flex", justifyContent: "space-between", alignItems: "flex-end" }}>
                <div>
                    <h1 style={{ fontSize: "1.6rem", fontWeight: 700, margin: 0, color: "var(--text)" }}>Sepia</h1>
                    <p style={{ margin: "0.3rem 0 0", color: "var(--muted)", fontSize: "0.9rem" }}>Your software, curated.</p>
                </div>
                <button onClick={activeTab === "software" ? refreshPrograms : scanJunk} disabled={loading || scanningJunk} style={{ background: "var(--surface)", color: "var(--accent)", padding: "0.6rem 1rem", borderRadius: 8, border: "1px solid rgba(193, 160, 148, 0.25)", cursor: "pointer" }}>
                    {loading || scanningJunk ? "Scanning..." : "Rescan"}
                </button>
            </header>

            {/* Tabs */}
            <div style={{ display: "flex", gap: "1rem", marginBottom: "1.5rem", borderBottom: "1px solid rgba(145, 107, 96, 0.2)", paddingBottom: "0.5rem" }}>
                <button
                    onClick={() => setActiveTab("software")}
                    style={{ background: "none", border: "none", color: activeTab === "software" ? "var(--text)" : "var(--muted)", fontSize: "1rem", fontWeight: 600, padding: "0.5rem 1rem", cursor: "pointer", borderBottom: activeTab === "software" ? "2px solid var(--accent)" : "none" }}
                >
                    Installed Software
                </button>
                <button
                    onClick={() => setActiveTab("cleaner")}
                    style={{ background: "none", border: "none", color: activeTab === "cleaner" ? "var(--text)" : "var(--muted)", fontSize: "1rem", fontWeight: 600, padding: "0.5rem 1rem", cursor: "pointer", borderBottom: activeTab === "cleaner" ? "2px solid var(--accent)" : "none" }}
                >
                    Smart Cleaner
                </button>
            </div>

            {/* Status Message */}
            {message && (
                <div style={{ background: "var(--surface)", borderLeft: "3px solid var(--accent)", padding: "0.75rem 1rem", borderRadius: 8, marginBottom: "1.25rem", fontSize: "0.85rem" }}>
                    {message}
                </div>
            )}

            {/* --- SOFTWARE TAB --- */}
            {activeTab === "software" && (
                <>
                    <div style={{ marginBottom: "1.5rem" }}>
                        <input type="text" placeholder="Search software..." value={searchQuery} onChange={(e) => setSearchQuery(e.target.value)} style={{ width: "100%", background: "var(--surface)", color: "var(--text)", border: "1px solid rgba(145, 107, 96, 0.3)", padding: "0.8rem 1rem", borderRadius: "10px", fontSize: "0.95rem", outline: "none" }} />
                    </div>

                    {loading ? (
                        <div style={{ color: "var(--muted)", textAlign: "center", padding: "3rem 0" }}>Scanning packages...</div>
                    ) : (
                        <>
                            <div style={{ display: "grid", gridTemplateColumns: "36px 2fr 80px 1.5fr 80px auto", gap: "1rem", padding: "0 1.1rem", marginBottom: "0.5rem", color: "var(--muted)", fontSize: "0.75rem", fontWeight: 600, textTransform: "uppercase" }}>
                                <div></div><div>Name</div><div>Version</div><div>Path</div><div>Size</div><div style={{ textAlign: "right" }}>Action</div>
                            </div>
                            <div style={{ display: "flex", flexDirection: "column", gap: "0.5rem" }}>
                                {filteredPrograms.map((program) => (
                                    <ProgramRow key={`${program.source}-${program.name}-${program.version}`} program={program} onUninstall={(p) => setConfirmTarget(p)} busy={busyName === program.name} />
                                ))}
                            </div>
                        </>
                    )}
                </>
            )}

            {/* --- CLEANER TAB --- */}
            {activeTab === "cleaner" && (
                <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
                    <div style={{ background: "var(--surface)", padding: "2rem", borderRadius: "16px", textAlign: "center", border: "1px solid var(--elevated)" }}>
                        <h2 style={{ fontSize: "2.5rem", color: "var(--accent)", margin: "0 0 0.5rem 0" }}>{totalJunk.toFixed(1)} MB</h2>
                        <p style={{ color: "var(--muted)", margin: 0 }}>Temporary files and caches found</p>
                    </div>

                    {scanningJunk ? (
                        <div style={{ color: "var(--muted)", textAlign: "center", padding: "2rem" }}>Analyzing system caches...</div>
                    ) : (
                        junk.map((item) => (
                            <div key={item.id} style={{ display: "grid", gridTemplateColumns: "2fr 120px auto", gap: "1rem", alignItems: "center", background: "#4A3B38", padding: "1rem 1.5rem", borderRadius: "var(--radius)", border: "1px solid rgba(193, 160, 148, 0.1)" }}>
                                <div>
                                    <div style={{ fontWeight: 600, color: "var(--text)", fontSize: "1.05rem" }}>{item.name}</div>
                                    <div style={{ fontSize: "0.8rem", color: "var(--muted)" }}>Safe to remove</div>
                                </div>
                                <div style={{ fontSize: "1rem", color: "var(--accent)", fontWeight: 600 }}>
                                    {item.size_mb > 0 ? `${item.size_mb.toFixed(1)} MB` : "Empty"}
                                </div>
                                <button
                                    disabled={item.size_mb <= 0 || cleaningId === item.id}
                                    onClick={() => handleCleanJunk(item.id)}
                                    style={{ background: "var(--surface)", color: item.size_mb > 0 ? "var(--text)" : "var(--muted)", padding: "0.6rem 1.2rem", fontSize: "0.85rem", border: "1px solid rgba(193, 160, 148, 0.2)", borderRadius: 6, cursor: item.size_mb > 0 ? "pointer" : "not-allowed" }}
                                >
                                    {cleaningId === item.id ? "Cleaning..." : "Clean"}
                                </button>
                            </div>
                        ))
                    )}
                </div>
            )}

            {/* Footer */}
            <footer style={{ marginTop: "3rem", paddingTop: "1.25rem", borderTop: "1px solid rgba(145, 107, 96, 0.2)", color: "var(--muted)", fontSize: "0.75rem", fontWeight: 500, textAlign: "center" }}>
                Built by{" "}
                <span onClick={() => invoke("open_url", { url: "https://github.com/iamthearafatkhan" })} style={{ color: "var(--accent)", cursor: "pointer", fontWeight: 600 }}>
                    Arafat Khan
                </span>
            </footer>

            {/* Modal */}
            {confirmTarget && (
                <div style={{ position: "fixed", inset: 0, backgroundColor: "rgba(33, 27, 26, 0.85)", backdropFilter: "blur(4px)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1000 }}>
                    <div style={{ background: "var(--surface)", padding: "2rem", borderRadius: "16px", width: "100%", maxWidth: "420px", border: "1px solid var(--elevated)", textAlign: "center" }}>
                        <h2 style={{ margin: "0 0 0.5rem 0", color: "var(--text)", fontSize: "1.25rem" }}>Uninstall Software?</h2>
                        <p style={{ color: "var(--muted)", fontSize: "0.9rem", marginBottom: "1.5rem" }}>Are you sure you want to remove <strong style={{ color: "var(--text)" }}>{confirmTarget.name}</strong>?</p>
                        <div style={{ display: "flex", gap: "0.75rem" }}>
                            <button onClick={() => setConfirmTarget(null)} style={{ flex: 1, padding: "0.75rem", borderRadius: "8px", background: "transparent", color: "var(--text)", border: "1px solid rgba(193, 160, 148, 0.3)", cursor: "pointer" }}>Cancel</button>
                            <button onClick={executeUninstall} style={{ flex: 1, padding: "0.75rem", borderRadius: "8px", background: "var(--accent)", color: "var(--canvas)", border: "none", fontWeight: 600, cursor: "pointer" }}>Uninstall</button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}