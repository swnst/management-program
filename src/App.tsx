import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { 
  HardDrive, 
  Trash2, 
  Cpu, 
  ShieldCheck, 
  Search, 
  RotateCcw, 
  Activity, 
  Settings,
  Sparkles,
  AlertTriangle,
  FolderTree,
  ChevronRight
} from "lucide-react";

interface JunkItem {
  id: string;
  category: string;
  title: string;
  description: string;
  risk: string;
  matched_bytes: number;
  matched_files_count: number;
}

interface CleanPlan {
  plan_id: string;
  items: JunkItem[];
  total_bytes: number;
  total_files: number;
  safe_bytes: number;
}

interface MemoryInsight {
  total_ram_bytes: number;
  used_ram_bytes: number;
  free_ram_bytes: number;
  top_processes: {
    pid: number;
    name: string;
    private_working_set_bytes: number;
    cpu_percent: number;
  }[];
}

interface StartupApp {
  id: string;
  name: string;
  command: string;
  location: string;
  is_enabled: boolean;
}

interface DiskSummary {
  volume_letter: string;
  volume_name: string;
  total_bytes: number;
  free_bytes: number;
  is_ntfs: boolean;
  is_system: boolean;
}

export default function App() {
  const [activeTab, setActiveTab] = useState<"home" | "explorer" | "cleaner" | "insight">("home");
  const [disks, setDisks] = useState<DiskSummary[]>([]);
  const [cleanPlan, setCleanPlan] = useState<CleanPlan | null>(null);
  const [memory, setMemory] = useState<MemoryInsight | null>(null);
  const [startupApps, setStartupApps] = useState<StartupApp[]>([]);
  const [statusMessage, setStatusMessage] = useState<string>("Ready");
  const [isScanning, setIsScanning] = useState(false);

  useEffect(() => {
    loadDisks();
    loadMemory();
    loadStartup();
  }, []);

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  };

  const loadDisks = async () => {
    try {
      const res = await invoke<DiskSummary[]>("get_disk_summary");
      setDisks(res);
    } catch (e) {
      console.error("Failed to load disks:", e);
    }
  };

  const loadMemory = async () => {
    try {
      const res = await invoke<MemoryInsight>("get_system_memory");
      setMemory(res);
    } catch (e) {
      console.error(e);
    }
  };

  const loadStartup = async () => {
    try {
      const res = await invoke<StartupApp[]>("get_startup_apps");
      setStartupApps(res);
    } catch (e) {
      console.error(e);
    }
  };

  const runSmartScan = async () => {
    setIsScanning(true);
    setStatusMessage("Analyzing drive C: and temporary files...");
    try {
      const plan = await invoke<CleanPlan>("preview_junk_clean");
      setCleanPlan(plan);
      setStatusMessage(`Scan Complete: Found ${formatBytes(plan.total_bytes)} across ${plan.total_files} files.`);
    } catch (e) {
      setStatusMessage("Scan failed: " + String(e));
    } finally {
      setIsScanning(false);
    }
  };

  const executeClean = async () => {
    if (!cleanPlan) return;
    setStatusMessage("Cleaning safe caches...");
    try {
      const res = await invoke<{ freed_immediate_bytes: number; deleted_files_count: number }>("execute_junk_clean", { onlySafe: true });
      setStatusMessage(`Clean Complete! Freed ${formatBytes(res.freed_immediate_bytes)} (${res.deleted_files_count} files removed).`);
      runSmartScan();
    } catch (e) {
      setStatusMessage("Cleanup failed: " + String(e));
    }
  };

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-slate-950 text-slate-100">
      {/* Sidebar Navigation */}
      <aside className="w-64 border-r border-slate-800 bg-slate-900/60 backdrop-blur-md p-4 flex flex-col justify-between">
        <div className="space-y-6">
          <div className="flex items-center space-x-3 px-2">
            <div className="h-9 w-9 rounded-xl bg-gradient-to-tr from-sky-500 to-indigo-500 flex items-center justify-center font-bold text-white shadow-lg shadow-sky-500/20">
              L
            </div>
            <div>
              <h1 className="font-semibold text-base leading-tight tracking-tight">Lumen</h1>
              <p className="text-xs text-slate-400">System Optimizer</p>
            </div>
          </div>

          <nav className="space-y-1">
            <button
              onClick={() => setActiveTab("home")}
              className={`w-full flex items-center space-x-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors ${
                activeTab === "home" ? "bg-sky-500/10 text-sky-400 border border-sky-500/20" : "text-slate-400 hover:bg-slate-800/60 hover:text-slate-200"
              }`}
            >
              <Sparkles className="w-4 h-4" />
              <span>Smart Dashboard</span>
            </button>

            <button
              onClick={() => setActiveTab("cleaner")}
              className={`w-full flex items-center space-x-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors ${
                activeTab === "cleaner" ? "bg-sky-500/10 text-sky-400 border border-sky-500/20" : "text-slate-400 hover:bg-slate-800/60 hover:text-slate-200"
              }`}
            >
              <Trash2 className="w-4 h-4" />
              <span>Junk Cleaner</span>
            </button>

            <button
              onClick={() => setActiveTab("explorer")}
              className={`w-full flex items-center space-x-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors ${
                activeTab === "explorer" ? "bg-sky-500/10 text-sky-400 border border-sky-500/20" : "text-slate-400 hover:bg-slate-800/60 hover:text-slate-200"
              }`}
            >
              <FolderTree className="w-4 h-4" />
              <span>Space Explorer</span>
            </button>

            <button
              onClick={() => setActiveTab("insight")}
              className={`w-full flex items-center space-x-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors ${
                activeTab === "insight" ? "bg-sky-500/10 text-sky-400 border border-sky-500/20" : "text-slate-400 hover:bg-slate-800/60 hover:text-slate-200"
              }`}
            >
              <Cpu className="w-4 h-4" />
              <span>Memory & Startup</span>
            </button>
          </nav>
        </div>

        <div className="pt-4 border-t border-slate-800/80 text-xs text-slate-500 flex items-center justify-between">
          <span>v0.1.0 (Production)</span>
          <span className="flex items-center space-x-1">
            <span className="h-2 w-2 rounded-full bg-emerald-500 animate-pulse"></span>
            <span className="text-emerald-500 font-medium">Safe Mode</span>
          </span>
        </div>
      </aside>

      {/* Main Content Area */}
      <main className="flex-1 flex flex-col overflow-hidden bg-slate-950">
        {/* Top Header */}
        <header className="h-14 border-b border-slate-800/80 px-6 flex items-center justify-between bg-slate-900/30">
          <div className="text-xs text-slate-400 flex items-center space-x-2">
            <span className="font-semibold text-slate-300">Status:</span>
            <span>{statusMessage}</span>
          </div>
          <div className="flex items-center space-x-3">
            <button 
              onClick={runSmartScan}
              disabled={isScanning}
              className="px-3.5 py-1.5 rounded-lg bg-sky-500 hover:bg-sky-400 text-slate-950 font-medium text-xs shadow-md shadow-sky-500/20 transition-all flex items-center space-x-2 disabled:opacity-50"
            >
              <Search className="w-3.5 h-3.5" />
              <span>{isScanning ? "Scanning..." : "Quick Scan"}</span>
            </button>
          </div>
        </header>

        {/* Tab Panes */}
        <div className="flex-1 overflow-y-auto p-6 space-y-6">
          {activeTab === "home" && (
            <div className="space-y-6 max-w-4xl">
              {/* Hero Banner */}
              <div className="relative overflow-hidden rounded-2xl border border-slate-800 bg-gradient-to-br from-slate-900 via-slate-900 to-indigo-950/40 p-8 shadow-2xl">
                <div className="relative z-10 max-w-xl space-y-3">
                  <span className="inline-flex items-center space-x-1.5 rounded-full bg-sky-500/10 px-3 py-1 text-xs font-medium text-sky-400 border border-sky-500/20">
                    <Sparkles className="w-3.5 h-3.5" />
                    <span>Native High-Speed Scanning</span>
                  </span>
                  <h2 className="text-2xl font-bold tracking-tight text-white">Keep Your PC Peak Performing</h2>
                  <p className="text-sm text-slate-400 leading-relaxed">
                    Instantly analyze system junk, deep driver shader caches, and memory hogs without breaking critical Windows files.
                  </p>
                  <div className="pt-2 flex items-center space-x-3">
                    <button
                      onClick={runSmartScan}
                      disabled={isScanning}
                      className="px-5 py-2.5 rounded-xl bg-sky-500 hover:bg-sky-400 text-slate-950 font-semibold text-sm shadow-lg shadow-sky-500/30 transition-all flex items-center space-x-2"
                    >
                      <Sparkles className="w-4 h-4" />
                      <span>{isScanning ? "Scanning Entire System..." : "One-Click Smart Scan"}</span>
                    </button>
                    {cleanPlan && cleanPlan.safe_bytes > 0 && (
                      <button
                        onClick={executeClean}
                        className="px-5 py-2.5 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-slate-950 font-semibold text-sm shadow-lg shadow-emerald-500/30 transition-all flex items-center space-x-2"
                      >
                        <Trash2 className="w-4 h-4" />
                        <span>Clean {formatBytes(cleanPlan.safe_bytes)}</span>
                      </button>
                    )}
                  </div>
                </div>
              </div>

              {/* Metric Cards Grid */}
              <div className="grid grid-cols-3 gap-4">
                {disks.length > 0 ? (
                  disks.map((d) => {
                    const usedBytes = d.total_bytes - d.free_bytes;
                    const usedPercent = d.total_bytes > 0 ? Math.round((usedBytes / d.total_bytes) * 100) : 0;
                    return (
                      <div key={d.volume_letter} className="p-4 rounded-xl border border-slate-800 bg-slate-900/50 backdrop-blur space-y-2">
                        <div className="flex items-center justify-between text-xs text-slate-400">
                          <span>{d.volume_name} ({d.volume_letter}:)</span>
                          <HardDrive className="w-4 h-4 text-sky-400" />
                        </div>
                        <div className="text-xl font-bold text-white">{formatBytes(d.free_bytes)} Free</div>
                        <div className="flex items-center justify-between text-[11px] text-slate-500">
                          <span>Used: {formatBytes(usedBytes)}</span>
                          <span>Total: {formatBytes(d.total_bytes)}</span>
                        </div>
                        <div className="w-full bg-slate-800 rounded-full h-1.5">
                          <div className="bg-sky-500 h-1.5 rounded-full" style={{ width: `${usedPercent}%` }}></div>
                        </div>
                      </div>
                    );
                  })
                ) : (
                  <div className="p-4 rounded-xl border border-slate-800 bg-slate-900/50 backdrop-blur space-y-2">
                    <div className="flex items-center justify-between text-xs text-slate-400">
                      <span>Storage</span>
                      <HardDrive className="w-4 h-4 text-sky-400" />
                    </div>
                    <div className="text-xl font-bold text-white">Detecting...</div>
                  </div>
                )}

                <div className="p-4 rounded-xl border border-slate-800 bg-slate-900/50 backdrop-blur space-y-2">
                  <div className="flex items-center justify-between text-xs text-slate-400">
                    <span>RAM Active Use</span>
                    <Cpu className="w-4 h-4 text-indigo-400" />
                  </div>
                  <div className="text-xl font-bold text-white">
                    {memory ? `${formatBytes(memory.used_ram_bytes)} / ${formatBytes(memory.total_ram_bytes)}` : "--"}
                  </div>
                  <div className="w-full bg-slate-800 rounded-full h-1.5">
                    <div
                      className="bg-indigo-500 h-1.5 rounded-full"
                      style={{
                        width: memory ? `${Math.round((memory.used_ram_bytes / memory.total_ram_bytes) * 100)}%` : "0%",
                      }}
                    ></div>
                  </div>
                </div>

                <div className="p-4 rounded-xl border border-slate-800 bg-slate-900/50 backdrop-blur space-y-2">
                  <div className="flex items-center justify-between text-xs text-slate-400">
                    <span>Recycle Bin Protection</span>
                    <ShieldCheck className="w-4 h-4 text-emerald-400" />
                  </div>
                  <div className="text-xl font-bold text-white">Active</div>
                  <div className="text-xs text-slate-500">Windows Shell Undo Available</div>
                </div>
              </div>

              {/* Clean Preview List */}
              {cleanPlan && (
                <div className="space-y-3">
                  <h3 className="text-sm font-semibold text-slate-300">Clean Recommendations</h3>
                  <div className="border border-slate-800 rounded-xl overflow-hidden divide-y divide-slate-800/80 bg-slate-900/40">
                    {cleanPlan.items.map((item) => (
                      <div key={item.id} className="p-4 flex items-center justify-between">
                        <div className="space-y-1">
                          <div className="flex items-center space-x-2">
                            <span className="font-medium text-sm text-slate-200">{item.title}</span>
                            <span
                              className={`text-[10px] font-semibold px-2 py-0.5 rounded-full ${
                                item.risk === "Safe"
                                  ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20"
                                  : "bg-amber-500/10 text-amber-400 border border-amber-500/20"
                              }`}
                            >
                              {item.risk}
                            </span>
                          </div>
                          <p className="text-xs text-slate-400">{item.description}</p>
                        </div>
                        <div className="text-right">
                          <div className="font-semibold text-sm text-slate-200">{formatBytes(item.matched_bytes)}</div>
                          <div className="text-xs text-slate-500">{item.matched_files_count} files</div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}

          {activeTab === "insight" && (
            <div className="space-y-6 max-w-4xl">
              <div className="flex items-center justify-between">
                <div>
                  <h2 className="text-lg font-bold text-white">Memory Insight & Startup Manager</h2>
                  <p className="text-xs text-slate-400">Inspect real-time memory usage and background startup applications.</p>
                </div>
                <button
                  onClick={async () => {
                    setStatusMessage("Refreshing process memory stats...");
                    await loadMemory();
                    setStatusMessage("Memory metrics updated.");
                  }}
                  className="px-4 py-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 text-xs font-semibold flex items-center space-x-2 transition-all"
                >
                  <RotateCcw className="w-3.5 h-3.5 text-slate-400" />
                  <span>Refresh Telemetry</span>
                </button>
              </div>

              {/* Top Processes */}
              <div className="space-y-3">
                <h3 className="text-sm font-semibold text-slate-300">Top Memory Consumers</h3>
                <div className="border border-slate-800 rounded-xl overflow-hidden bg-slate-900/40 divide-y divide-slate-800/80">
                  {memory?.top_processes.map((proc) => (
                    <div key={proc.pid} className="px-4 py-3 flex items-center justify-between text-sm">
                      <div className="flex items-center space-x-3">
                        <span className="font-mono text-xs text-slate-500">#{proc.pid}</span>
                        <span className="font-medium text-slate-200">{proc.name}</span>
                      </div>
                      <div className="flex items-center space-x-6">
                        <span className="text-xs font-mono text-slate-400">{proc.cpu_percent.toFixed(1)}% CPU</span>
                        <span className="font-semibold font-mono text-sky-400">{formatBytes(proc.private_working_set_bytes)}</span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Startup Programs */}
              <div className="space-y-3">
                <h3 className="text-sm font-semibold text-slate-300">Windows Startup Apps</h3>
                <div className="border border-slate-800 rounded-xl overflow-hidden bg-slate-900/40 divide-y divide-slate-800/80">
                  {startupApps.map((app) => (
                    <div key={app.id} className="p-4 flex items-center justify-between">
                      <div className="space-y-1 max-w-md">
                        <div className="font-medium text-sm text-slate-200">{app.name}</div>
                        <div className="text-xs text-slate-500 font-mono truncate">{app.command}</div>
                      </div>
                      <div className="text-xs text-slate-400 font-medium px-2.5 py-1 rounded-md bg-slate-800">
                        {app.location}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {activeTab === "cleaner" && (
            <div className="space-y-6 max-w-4xl">
              <div>
                <h2 className="text-lg font-bold text-white">Smart Junk Cleaner</h2>
                <p className="text-xs text-slate-400">Rule-based cleaning with risk levels to protect system stability.</p>
              </div>
              <button
                onClick={runSmartScan}
                className="px-4 py-2 rounded-xl bg-sky-500 text-slate-950 font-semibold text-xs flex items-center space-x-2"
              >
                <Search className="w-3.5 h-3.5" />
                <span>Analyze System Clean Targets</span>
              </button>
            </div>
          )}

          {activeTab === "explorer" && (
            <div className="space-y-6 max-w-4xl">
              <div>
                <h2 className="text-lg font-bold text-white">Space Explorer</h2>
                <p className="text-xs text-slate-400">Fast visual disk analysis and age classification.</p>
              </div>
              <div className="p-8 border border-dashed border-slate-800 rounded-xl text-center text-slate-500 text-sm">
                Select a directory or drive above to run deep multi-threaded indexing.
              </div>
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
