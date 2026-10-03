import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  FluentProvider,
  webDarkTheme,
  Button,
  Card,
  CardHeader,
  ProgressBar,
  Badge,
  TabList,
  Tab,
  Dialog,
  DialogSurface,
  DialogTitle,
  DialogBody,
  DialogActions,
  DialogContent,
  Table,
  TableHeader,
  TableRow,
  TableHeaderCell,
  TableBody,
  TableCell,
  TableCellLayout,
  Spinner,
} from "@fluentui/react-components";
import {
  Storage24Regular,
  Flash24Regular,
  Folder24Regular,
  Apps24Regular,
  Settings24Regular,
  ArrowClockwise20Regular,
  Delete20Regular,
  CheckmarkCircle20Regular,
  ShieldCheckmark24Regular,
  DocumentCopy24Regular,
  Broom24Regular,
  Sparkle24Regular,
} from "@fluentui/react-icons";

// --- Types ---

interface DiskSummary {
  volume_letter: string;
  volume_name: string;
  total_bytes: number;
  free_bytes: number;
  is_ntfs: boolean;
  is_system: boolean;
}

interface VolumeScanResult {
  volume_letter: string;
  total_files: number;
  total_dirs: number;
  total_bytes: number;
  is_cached: boolean;
  top_items?: FileRow[];
}

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

interface FileRow {
  id: string;
  name: string;
  path: string;
  size_bytes: number;
  created_year: number;
  age_label: string;
  is_dir: boolean;
  label: "Safe" | "Review" | "Keep" | string;
  reason: string;
}

interface DuplicateGroup {
  hash: string;
  file_size: number;
  paths: string[];
}

interface LeftoverCandidate {
  program_hint: string;
  path: string;
  size_bytes: number;
  files_count: number;
  confidence: string;
}

export default function App() {
  const [selectedTab, setSelectedTab] = useState<string>("overview");
  const [userMode, setUserMode] = useState<"simple" | "advanced">("simple");
  const [showOnboarding, setShowOnboarding] = useState(false);

  const [disks, setDisks] = useState<DiskSummary[]>([]);
  const [memory, setMemory] = useState<MemoryInsight | null>(null);
  const [cleanPlan, setCleanPlan] = useState<CleanPlan | null>(null);
  const [startupApps, setStartupApps] = useState<StartupApp[]>([]);

  // Explorer state
  const [isScanningDisk, setIsScanningDisk] = useState(false);
  const [scanResult, setScanResult] = useState<VolumeScanResult | null>(null);
  const [explorerSort, setExplorerSort] = useState<"size" | "year">("size");
  const [statusText, setStatusText] = useState("พร้อมใช้งาน");

  // Real items from scan
  const [fileList, setFileList] = useState<FileRow[]>([]);

  // Duplicate files state
  const [duplicates, setDuplicates] = useState<DuplicateGroup[]>([]);
  const [isFindingDupes, setIsFindingDupes] = useState(false);

  // Leftovers state
  const [leftovers, setLeftovers] = useState<LeftoverCandidate[]>([]);
  const [isScanningLeftovers, setIsScanningLeftovers] = useState(false);

  useEffect(() => {
    const savedMode = localStorage.getItem("lumen_user_mode");
    if (!savedMode) {
      setShowOnboarding(true);
    } else {
      setUserMode(savedMode as "simple" | "advanced");
    }

    loadInitialData();
  }, []);

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  };

  const loadInitialData = async () => {
    try {
      const diskList = await invoke<DiskSummary[]>("get_disk_summary");
      setDisks(diskList);

      const mem = await invoke<MemoryInsight>("get_system_memory");
      setMemory(mem);

      const apps = await invoke<StartupApp[]>("get_startup_apps");
      setStartupApps(apps);

      if (diskList.length > 0) {
        runVolumeScan(diskList[0].volume_letter.charAt(0));
      }
    } catch (e) {
      console.error(e);
    }
  };

  const runVolumeScan = async (driveLetter: string) => {
    setIsScanningDisk(true);
    setStatusText(`กำลังสแกนไดรฟ์ ${driveLetter}: ด้วยเทคโนโลยี Win32 Multi-thread...`);
    try {
      const res = await invoke<VolumeScanResult>("scan_volume", { driveLetter: driveLetter.charAt(0) });
      setScanResult(res);

      if (res.top_items && res.top_items.length > 0) {
        setFileList(res.top_items);
      }

      setStatusText(
        `สแกนเสร็จสิ้น (${res.is_cached ? "จาก Cache" : "สแกนสด"}): ตรวจพบ ${res.total_files.toLocaleString()} ไฟล์ (${formatBytes(
          res.total_bytes
        )})`
      );
    } catch (e) {
      setStatusText(`การสแกนขัดข้อง: ${String(e)}`);
    } finally {
      setIsScanningDisk(false);
    }
  };

  const handleFindDuplicates = async () => {
    if (fileList.length === 0) {
      setStatusText("กรุณาสแกนไดรฟ์ก่อนเพื่อค้นหาไฟล์ซ้ำ");
      return;
    }
    setIsFindingDupes(true);
    setStatusText("กำลังตรวจสอบไฟล์ซ้ำด้วย 3-Stage BLAKE3 Hash...");
    try {
      const candidates = fileList.map((f) => [f.path, f.size_bytes] as [string, number]);
      const res = await invoke<DuplicateGroup[]>("find_duplicate_files", { paths: candidates });
      setDuplicates(res);
      setStatusText(`พบไฟล์ซ้ำ ${res.length} กลุ่ม`);
    } catch (e) {
      setStatusText(`ค้นหาไฟล์ซ้ำล้มเหลว: ${String(e)}`);
    } finally {
      setIsFindingDupes(false);
    }
  };

  const handleScanLeftovers = async () => {
    setIsScanningLeftovers(true);
    setStatusText("กำลังตรวจสอบซากโปรแกรมที่ถอนการติดตั้งแล้วจาก Registry...");
    try {
      const res = await invoke<LeftoverCandidate[]>("scan_app_leftovers");
      setLeftovers(res);
      const totalLeftoverBytes = res.reduce((acc, curr) => acc + curr.size_bytes, 0);
      setStatusText(`ตรวจพบซากโปรแกรมเก่า ${res.length} รายการ (${formatBytes(totalLeftoverBytes)})`);
    } catch (e) {
      setStatusText(`ตรวจสอบซากโปรแกรมล้มเหลว: ${String(e)}`);
    } finally {
      setIsScanningLeftovers(false);
    }
  };

  const handleSelectMode = (mode: "simple" | "advanced") => {
    setUserMode(mode);
    localStorage.setItem("lumen_user_mode", mode);
    setShowOnboarding(false);
  };

  const handleRecycleFile = async (path: string) => {
    try {
      await invoke("recycle_file_or_folder", { path });
      setStatusText(`ย้าย "${path}" ไปยัง Windows Recycle Bin สำเร็จ (ปลอดภัย กู้คืนได้เสมอ)`);
      setFileList((prev) => prev.filter((f) => f.path !== path));
      setLeftovers((prev) => prev.filter((l) => l.path !== path));
    } catch (e) {
      setStatusText(`ล้มเหลว: ${String(e)}`);
    }
  };

  return (
    <FluentProvider theme={webDarkTheme} className="h-screen w-screen flex flex-col bg-[#0b0f17] text-[#f1f5f9] select-none font-sans overflow-hidden">
      {/* Top Header / App Titlebar */}
      <header className="h-12 border-b border-white/[0.08] px-5 flex items-center justify-between bg-[#111827]/80 backdrop-blur-xl">
        <div className="flex items-center space-x-3">
          <div className="h-7 w-7 rounded-lg bg-gradient-to-tr from-sky-500 to-indigo-500 flex items-center justify-center font-bold text-xs text-white shadow-md shadow-sky-500/20">
            L
          </div>
          <span className="font-semibold text-sm tracking-tight text-white">Lumen</span>
          <Badge
            appearance="tint"
            color={userMode === "advanced" ? "informative" : "brand"}
            className="text-[11px] font-medium"
          >
            {userMode === "advanced" ? "Advanced Mode" : "Simple Mode"}
          </Badge>
        </div>

        <div className="flex items-center space-x-4 text-xs text-slate-400">
          <div className="flex items-center space-x-2 bg-slate-900/60 px-3 py-1 rounded-full border border-white/[0.06]">
            <span className="h-2 w-2 rounded-full bg-emerald-400 animate-pulse"></span>
            <span className="truncate max-w-sm">{statusText}</span>
          </div>
          <Button
            size="small"
            appearance="subtle"
            icon={<Settings24Regular />}
            onClick={() => setShowOnboarding(true)}
            className="text-slate-300 hover:text-white"
          >
            โหมดการใช้งาน
          </Button>
        </div>
      </header>

      {/* Main Container */}
      <div className="flex-1 flex overflow-hidden">
        {/* Navigation Sidebar */}
        <aside className="w-56 border-r border-white/[0.06] bg-[#0e1422]/90 p-3 flex flex-col justify-between">
          <TabList
            selectedValue={selectedTab}
            onTabSelect={(_, data) => setSelectedTab(data.value as string)}
            vertical
            className="space-y-1.5"
          >
            <Tab value="overview" icon={<Storage24Regular />}>
              หน้าภาพรวม
            </Tab>
            <Tab value="explorer" icon={<Folder24Regular />}>
              สำรวจพื้นที่ดิสก์
            </Tab>
            <Tab value="cleaner" icon={<Flash24Regular />}>
              ล้างไฟล์ขยะระบบ
            </Tab>
            <Tab value="dupes" icon={<DocumentCopy24Regular />}>
              ตรวจหาไฟล์ซ้ำ
            </Tab>
            <Tab value="leftovers" icon={<Broom24Regular />}>
              ซากโปรแกรมตกค้าง
            </Tab>
            <Tab value="memory" icon={<Apps24Regular />}>
              วิเคราะห์แรม & สตาร์ตอัป
            </Tab>
          </TabList>

          <div className="p-3 rounded-xl border border-white/[0.06] bg-slate-900/50 text-[11px] text-slate-400 space-y-1.5">
            <div className="flex items-center space-x-1.5 text-emerald-400 font-semibold">
              <ShieldCheckmark24Regular className="w-4 h-4 text-emerald-400" />
              <span>Recycle Bin Protection</span>
            </div>
            <div className="leading-relaxed">
              ทุกการลบไฟล์จะถูกย้ายลงถังขยะ Windows เสมอ สามารถกู้คืนได้ทันที
            </div>
          </div>
        </aside>

        {/* Content View */}
        <main className="flex-1 overflow-y-auto p-6 bg-[#090d16]">
          {/* OVERVIEW TAB */}
          {selectedTab === "overview" && (
            <div className="max-w-5xl space-y-6">
              <div>
                <h1 className="text-2xl font-bold text-white tracking-tight flex items-center space-x-2">
                  <span>ภาพรวมระบบและพื้นที่จัดเก็บ</span>
                  <Sparkle24Regular className="w-5 h-5 text-sky-400" />
                </h1>
                <p className="text-xs text-slate-400 mt-1">
                  ตรวจสอบความจุของดิสก์จริงจากระบบปฏิบัติการ Windows และสถานะการจัดเก็บ
                </p>
              </div>

              {/* Bento Grid: Storage Drives */}
              <div className="grid grid-cols-2 gap-4">
                {disks.map((d) => {
                  const used = d.total_bytes - d.free_bytes;
                  const pct = d.total_bytes > 0 ? (used / d.total_bytes) * 100 : 0;
                  return (
                    <Card key={d.volume_letter} className="acrylic-card rounded-2xl p-5 space-y-4">
                      <CardHeader
                        header={
                          <div className="flex items-center justify-between w-full">
                            <span className="font-semibold text-base text-white tracking-tight">
                              {d.volume_name} ({d.volume_letter}:)
                            </span>
                            <Badge appearance="tint" color="informative">
                              {d.is_ntfs ? "NTFS" : "FAT32"}
                            </Badge>
                          </div>
                        }
                        description={
                          <span className="text-xs text-slate-400">
                            {d.is_system ? "ไดรฟ์หลักของระบบ Windows" : "ไดรฟ์ข้อมูลสำรอง"}
                          </span>
                        }
                        action={
                          <Button
                            size="small"
                            appearance="primary"
                            icon={<ArrowClockwise20Regular />}
                            disabled={isScanningDisk}
                            onClick={() => runVolumeScan(d.volume_letter)}
                            className="tactile-btn"
                          >
                            สแกนไดรฟ์
                          </Button>
                        }
                      />
                      <div className="space-y-2">
                        <div className="flex justify-between text-xs text-slate-300">
                          <span className="font-medium text-emerald-400">ว่าง {formatBytes(d.free_bytes)}</span>
                          <span className="text-slate-400">
                            ใช้ไป {formatBytes(used)} / {formatBytes(d.total_bytes)} ({Math.round(pct)}%)
                          </span>
                        </div>
                        <ProgressBar
                          value={pct / 100}
                          color={pct > 90 ? "error" : pct > 75 ? "warning" : "brand"}
                        />
                      </div>
                    </Card>
                  );
                })}
              </div>

              {/* Scan Status Bento Banner */}
              {scanResult && (
                <div className="p-5 rounded-2xl border border-sky-500/20 bg-gradient-to-r from-sky-950/30 to-indigo-950/20 flex items-center justify-between">
                  <div className="space-y-1">
                    <h3 className="font-semibold text-sm text-sky-300">
                      ผลการวิเคราะห์โครงสร้างไฟล์ไดรฟ์ {scanResult.volume_letter}:
                    </h3>
                    <p className="text-xs text-slate-400">
                      สำรวจพบทั้งหมด <span className="font-mono text-white font-semibold">{scanResult.total_files.toLocaleString()}</span> ไฟล์ ใน{" "}
                      <span className="font-mono text-white font-semibold">{scanResult.total_dirs.toLocaleString()}</span> โฟลเดอร์ ({formatBytes(scanResult.total_bytes)})
                    </p>
                  </div>
                  <Button
                    appearance="outline"
                    onClick={() => setSelectedTab("explorer")}
                    className="tactile-btn text-white border-white/[0.12] hover:bg-white/[0.05]"
                  >
                    เปิดตารางแยกตามอายุ/ขนาด
                  </Button>
                </div>
              )}
            </div>
          )}

          {/* SPACE EXPLORER TAB */}
          {selectedTab === "explorer" && (
            <div className="max-w-5xl space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <h1 className="text-xl font-bold text-white tracking-tight">สำรวจพื้นที่และอายุของไฟล์ในเครื่อง</h1>
                  <p className="text-xs text-slate-400 mt-0.5">
                    จัดลำดับไฟล์ตามขนาดใหญ่ที่สุด หรือเรียงตามปีที่ไฟล์เข้ามาอยู่ในเครื่องเพื่อให้คุณตัดสินใจเองได้
                  </p>
                </div>

                <div className="flex items-center space-x-2">
                  <Button
                    size="small"
                    appearance={explorerSort === "size" ? "primary" : "subtle"}
                    onClick={() => setExplorerSort("size")}
                    className="tactile-btn"
                  >
                    เรียงตามขนาด (ใหญ่ไปเล็ก)
                  </Button>
                  <Button
                    size="small"
                    appearance={explorerSort === "year" ? "primary" : "subtle"}
                    onClick={() => setExplorerSort("year")}
                    className="tactile-btn"
                  >
                    เรียงตามปี (เก่าที่สุดก่อน)
                  </Button>
                </div>
              </div>

              {/* Data Table */}
              <div className="rounded-2xl border border-white/[0.08] bg-[#101726]/80 overflow-hidden shadow-xl">
                <Table>
                  <TableHeader>
                    <TableRow className="border-b border-white/[0.06] bg-slate-900/40">
                      <TableHeaderCell className="text-slate-300 font-semibold text-xs">ชื่อไฟล์ / โฟลเดอร์</TableHeaderCell>
                      <TableHeaderCell className="text-slate-300 font-semibold text-xs">ขนาด</TableHeaderCell>
                      <TableHeaderCell className="text-slate-300 font-semibold text-xs">อยู่ในเครื่องมานาน</TableHeaderCell>
                      <TableHeaderCell className="text-slate-300 font-semibold text-xs">การประเมินความปลอดภัย</TableHeaderCell>
                      <TableHeaderCell className="text-slate-300 font-semibold text-xs">การจัดการ</TableHeaderCell>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {fileList
                      .slice()
                      .sort((a, b) => {
                        if (explorerSort === "size") return b.size_bytes - a.size_bytes;
                        return a.created_year - b.created_year;
                      })
                      .map((row) => (
                        <TableRow key={row.id} className="border-b border-white/[0.04] hover:bg-white/[0.03] transition-colors duration-150">
                          <TableCell>
                            <TableCellLayout>
                              <div className="flex flex-col py-1">
                                <span className="font-medium text-white text-xs">{row.name}</span>
                                <span className="text-slate-500 text-[11px] truncate max-w-sm font-mono">{row.path}</span>
                              </div>
                            </TableCellLayout>
                          </TableCell>
                          <TableCell>
                            <span className="font-mono text-xs font-semibold text-sky-400">
                              {formatBytes(row.size_bytes)}
                            </span>
                          </TableCell>
                          <TableCell>
                            <div className="text-xs">
                              <span className="text-slate-200">{row.age_label}</span>
                              <span className="text-slate-500 block text-[10px]">ตั้งแต่ปี {row.created_year}</span>
                            </div>
                          </TableCell>
                          <TableCell>
                            <div className="space-y-0.5">
                              <span
                                className={`inline-block px-2 py-0.5 rounded text-[11px] font-medium border ${
                                  row.label === "Safe"
                                    ? "bg-emerald-950/40 text-emerald-300 border-emerald-500/20"
                                    : row.label === "Review"
                                    ? "bg-amber-950/40 text-amber-300 border-amber-500/20"
                                    : "bg-sky-950/40 text-sky-300 border-sky-500/20"
                                }`}
                              >
                                {row.label === "Safe" ? "ปลอดภัยที่จะลบ" : row.label === "Review" ? "ควรตรวจสอบก่อน" : "ควรเก็บไว้"}
                              </span>
                              <span className="text-[10px] text-slate-400 block max-w-xs truncate">
                                {row.reason}
                              </span>
                            </div>
                          </TableCell>
                          <TableCell>
                            <Button
                              size="small"
                              appearance="subtle"
                              icon={<Delete20Regular className="text-rose-400" />}
                              onClick={() => handleRecycleFile(row.path)}
                              className="text-slate-400 hover:text-rose-300 hover:bg-rose-950/20"
                            >
                              ย้ายลงถังขยะ
                            </Button>
                          </TableCell>
                        </TableRow>
                      ))}
                  </TableBody>
                </Table>
              </div>
            </div>
          )}

          {/* JUNK CLEANER TAB */}
          {selectedTab === "cleaner" && (
            <div className="max-w-4xl space-y-6">
              <div>
                <h1 className="text-xl font-bold text-white tracking-tight">ทำความสะอาดไฟล์ขยะระบบ</h1>
                <p className="text-xs text-slate-400 mt-1">
                  ล้างไฟล์ Temp แคชไดรเวอร์การ์ดจอ และไฟล์รีพอร์ตที่ไม่จำเป็นอย่างปลอดภัย
                </p>
              </div>

              <div className="acrylic-card p-6 rounded-2xl flex flex-col items-center justify-center text-center space-y-4">
                <Flash24Regular className="w-10 h-10 text-amber-400" />
                <div className="max-w-md">
                  <h3 className="text-base font-semibold text-white">วิเคราะห์ขยะระบบและ Shader Cache</h3>
                  <p className="text-xs text-slate-400 mt-1">
                    ระบบจะตรวจสอบเฉพาะแคชและไฟล์ชั่วคราวที่มีอายุเกิน 24 ชั่วโมง เพื่อไม่ให้กระทบต่อโปรแกรมที่กำลังทำงาน
                  </p>
                </div>
                <Button
                  appearance="primary"
                  size="large"
                  onClick={async () => {
                    setStatusText("กำลังตรวจสอบขยะระบบ...");
                    const plan = await invoke<CleanPlan>("preview_junk_clean");
                    setCleanPlan(plan);
                    setStatusText(`พบขยะที่ปลอดภัย ${formatBytes(plan.safe_bytes)}`);
                  }}
                  className="tactile-btn"
                >
                  เริ่มวิเคราะห์ขยะระบบ
                </Button>
              </div>

              {cleanPlan && (
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-semibold text-white">รายการที่ตรวจพบ ({formatBytes(cleanPlan.safe_bytes)})</span>
                    <Button
                      appearance="primary"
                      onClick={async () => {
                        setStatusText("กำลังล้างไฟล์ขยะปลอดภัย...");
                        await invoke("execute_junk_clean", { onlySafe: true });
                        setStatusText("ล้างไฟล์ขยะระบบเสร็จสิ้น");
                        setCleanPlan(null);
                      }}
                      className="tactile-btn bg-emerald-600 hover:bg-emerald-500"
                    >
                      ยืนยันการล้าง ({formatBytes(cleanPlan.safe_bytes)})
                    </Button>
                  </div>

                  <div className="rounded-2xl border border-white/[0.08] bg-[#101726]/80 overflow-hidden divide-y divide-white/[0.04]">
                    {cleanPlan.items.map((item) => (
                      <div key={item.id} className="p-4 flex items-center justify-between">
                        <div>
                          <span className="font-semibold text-sm text-slate-200">{item.title}</span>
                          <p className="text-xs text-slate-400 mt-0.5">{item.description}</p>
                        </div>
                        <div className="text-right">
                          <span className="font-mono text-sm text-sky-400 font-bold">{formatBytes(item.matched_bytes)}</span>
                          <span className="block text-[11px] text-slate-500">{item.matched_files_count} ไฟล์</span>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}

          {/* DUPLICATE FILES TAB */}
          {selectedTab === "dupes" && (
            <div className="max-w-5xl space-y-6">
              <div className="flex items-center justify-between">
                <div>
                  <h1 className="text-xl font-bold text-white tracking-tight">ตรวจหาไฟล์ซ้ำ (Duplicate Finder)</h1>
                  <p className="text-xs text-slate-400 mt-1">
                    ค้นหาไฟล์ที่เนื้อหาข้างในเหมือนกันเป๊ะด้วยขั้นตอน 3-Stage (Size Filter → 64KB Quick Hash → BLAKE3 Full Hash)
                  </p>
                </div>
                <Button
                  appearance="primary"
                  icon={<DocumentCopy24Regular />}
                  disabled={isFindingDupes}
                  onClick={handleFindDuplicates}
                  className="tactile-btn"
                >
                  {isFindingDupes ? "กำลังคำนวณ Hash..." : "เริ่มค้นหาไฟล์ซ้ำ"}
                </Button>
              </div>

              {duplicates.length === 0 ? (
                <div className="acrylic-card p-10 rounded-2xl text-center space-y-3">
                  <DocumentCopy24Regular className="w-12 h-12 text-slate-500 mx-auto" />
                  <div className="text-slate-300 font-medium text-sm">ยังไม่มีผลการตรวจสอบไฟล์ซ้ำ</div>
                  <p className="text-xs text-slate-500 max-w-sm mx-auto">
                    กดปุ่มเริ่มค้นหาเพื่อวิเคราะห์ไฟล์ที่เหมือนกันในไดรฟ์ เพื่อช่วยคืนพื้นที่จัดเก็บ
                  </p>
                </div>
              ) : (
                <div className="space-y-4">
                  {duplicates.map((group, idx) => (
                    <div key={idx} className="rounded-2xl border border-white/[0.08] bg-[#101726]/80 p-4 space-y-3">
                      <div className="flex items-center justify-between border-b border-white/[0.06] pb-2">
                        <span className="text-xs font-mono text-slate-400">
                          ขนาดไฟล์ละ: <strong className="text-sky-400">{formatBytes(group.file_size)}</strong> (ซ้ำ {group.paths.length} ไฟล์)
                        </span>
                        <span className="text-[10px] font-mono text-slate-500 truncate max-w-xs">
                          Hash: {group.hash.substring(0, 16)}...
                        </span>
                      </div>
                      <div className="space-y-2">
                        {group.paths.map((p, pIdx) => (
                          <div key={pIdx} className="flex items-center justify-between text-xs">
                            <span className="font-mono text-slate-300 truncate max-w-lg">{p}</span>
                            <Button
                              size="small"
                              appearance="subtle"
                              icon={<Delete20Regular className="text-rose-400" />}
                              onClick={() => handleRecycleFile(p)}
                              className="text-slate-400 hover:text-rose-300"
                            >
                              ย้ายลงถังขยะ
                            </Button>
                          </div>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* LEFTOVERS TAB */}
          {selectedTab === "leftovers" && (
            <div className="max-w-5xl space-y-6">
              <div className="flex items-center justify-between">
                <div>
                  <h1 className="text-xl font-bold text-white tracking-tight">ซากโปรแกรมตกค้าง (Leftovers Inspector)</h1>
                  <p className="text-xs text-slate-400 mt-1">
                    ตรวจสอบโฟลเดอร์ของโปรแกรมที่ถูก Uninstall ไปแล้ว แต่ยังทิ้งขยะสะสมไว้ใน AppData หรือ ProgramData
                  </p>
                </div>
                <Button
                  appearance="primary"
                  icon={<Broom24Regular />}
                  disabled={isScanningLeftovers}
                  onClick={handleScanLeftovers}
                  className="tactile-btn"
                >
                  {isScanningLeftovers ? "กำลังตรวจสอบ Registry..." : "ค้นหาซากโปรแกรม"}
                </Button>
              </div>

              {leftovers.length === 0 ? (
                <div className="acrylic-card p-10 rounded-2xl text-center space-y-3">
                  <Broom24Regular className="w-12 h-12 text-slate-500 mx-auto" />
                  <div className="text-slate-300 font-medium text-sm">ไม่พบซากโปรแกรมตกค้าง หรือยังไม่ได้เริ่มสแกน</div>
                  <p className="text-xs text-slate-500 max-w-sm mx-auto">
                    กดปุ่มค้นหาซากโปรแกรมเพื่อเปรียบเทียบ Registry ที่ติดตั้งกับโฟลเดอร์ในเครื่อง
                  </p>
                </div>
              ) : (
                <div className="rounded-2xl border border-white/[0.08] bg-[#101726]/80 overflow-hidden shadow-xl">
                  <Table>
                    <TableHeader>
                      <TableRow className="border-b border-white/[0.06] bg-slate-900/40">
                        <TableHeaderCell className="text-slate-300 font-semibold text-xs">ชื่อโปรแกรมที่เคยติดตั้ง</TableHeaderCell>
                        <TableHeaderCell className="text-slate-300 font-semibold text-xs">ที่ตั้งโฟลเดอร์</TableHeaderCell>
                        <TableHeaderCell className="text-slate-300 font-semibold text-xs">ขนาดพื้นที่</TableHeaderCell>
                        <TableHeaderCell className="text-slate-300 font-semibold text-xs">การจัดการ</TableHeaderCell>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {leftovers.map((l, idx) => (
                        <TableRow key={idx} className="border-b border-white/[0.04] hover:bg-white/[0.03]">
                          <TableCell>
                            <span className="font-semibold text-white text-xs">{l.program_hint}</span>
                          </TableCell>
                          <TableCell>
                            <span className="font-mono text-slate-400 text-xs truncate max-w-md block">{l.path}</span>
                          </TableCell>
                          <TableCell>
                            <span className="font-mono text-sky-400 font-semibold text-xs">{formatBytes(l.size_bytes)}</span>
                          </TableCell>
                          <TableCell>
                            <Button
                              size="small"
                              appearance="subtle"
                              icon={<Delete20Regular className="text-rose-400" />}
                              onClick={() => handleRecycleFile(l.path)}
                              className="text-slate-400 hover:text-rose-300"
                            >
                              ย้ายลงถังขยะ
                            </Button>
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>
              )}
            </div>
          )}

          {/* MEMORY TAB */}
          {selectedTab === "memory" && (
            <div className="max-w-4xl space-y-6">
              <div className="flex items-center justify-between">
                <div>
                  <h1 className="text-xl font-bold text-white tracking-tight">การทำงานของหน่วยความจำ (RAM) และ Startup Apps</h1>
                  <p className="text-xs text-slate-400 mt-1">
                    ดูโปรเซสที่ใช้แรมสูงสุด และโปรแกรมที่เปิดอัตโนมัติตอนเปิดเครื่อง
                  </p>
                </div>
                <Button
                  appearance="subtle"
                  icon={<ArrowClockwise20Regular />}
                  onClick={async () => {
                    const mem = await invoke<MemoryInsight>("get_system_memory");
                    setMemory(mem);
                    setStatusText("อัปเดตสถิติแรมเรียบร้อย");
                  }}
                  className="tactile-btn text-slate-300 hover:text-white"
                >
                  รีเฟรชสถิติ
                </Button>
              </div>

              {memory && (
                <Card className="acrylic-card rounded-2xl p-5 space-y-4">
                  <div className="flex justify-between items-center text-xs text-slate-300">
                    <span className="font-semibold text-white">การใช้งาน RAM จริง</span>
                    <span className="font-mono">{formatBytes(memory.used_ram_bytes)} / {formatBytes(memory.total_ram_bytes)} ({Math.round((memory.used_ram_bytes / memory.total_ram_bytes) * 100)}%)</span>
                  </div>
                  <ProgressBar value={memory.used_ram_bytes / memory.total_ram_bytes} />

                  <div className="pt-2 border-t border-white/[0.06] flex items-center justify-between">
                    <div>
                      <span className="text-xs font-semibold text-white block">Standby Memory Cache Purge</span>
                      <span className="text-[11px] text-slate-400">
                        เคลียร์ Standby List Cache ของระบบ Windows จริงด้วยสิทธิ์ SeProfileSingleProcessPrivilege (ไม่มี placebo)
                      </span>
                    </div>
                    <Button
                      appearance="primary"
                      size="medium"
                      onClick={async () => {
                        try {
                          setStatusText("กำลังเคลียร์ Standby List Cache ของ Windows...");
                          const res = await invoke<string>("purge_standby_memory");
                          setStatusText(res);
                          const mem = await invoke<MemoryInsight>("get_system_memory");
                          setMemory(mem);
                        } catch (err: any) {
                          setStatusText(`เคลียร์ไม่สำเร็จ: ${err}`);
                        }
                      }}
                      className="tactile-btn"
                    >
                      เคลียร์ Standby Cache
                    </Button>
                  </div>
                </Card>
              )}

              {/* Process List */}
              <div className="space-y-3">
                <h3 className="text-sm font-semibold text-white">โปรเซสที่ใช้หน่วยความจำสูงสุด</h3>
                <div className="rounded-2xl border border-white/[0.08] bg-[#101726]/80 divide-y divide-white/[0.04] overflow-hidden">
                  {memory?.top_processes.slice(0, 8).map((p) => (
                    <div key={p.pid} className="px-4 py-3 flex items-center justify-between text-sm hover:bg-white/[0.02]">
                      <div className="flex items-center space-x-3">
                        <span className="font-mono text-xs text-slate-500">#{p.pid}</span>
                        <span className="font-medium text-white">{p.name}</span>
                      </div>
                      <div className="flex items-center space-x-4">
                        <span className="text-xs text-slate-400 font-mono">{p.cpu_percent.toFixed(1)}% CPU</span>
                        <span className="text-xs font-mono font-bold text-sky-400">{formatBytes(p.private_working_set_bytes)}</span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}
        </main>
      </div>

      {/* Onboarding Dialog */}
      <Dialog open={showOnboarding} onOpenChange={(_, data) => setShowOnboarding(data.open)}>
        <DialogSurface className="bg-[#111827] text-white border border-white/[0.1] max-w-lg rounded-2xl shadow-2xl">
          <DialogBody>
            <DialogTitle className="text-lg font-bold">ยินดีต้อนรับสู่ Lumen</DialogTitle>
            <DialogContent className="space-y-4 text-xs text-slate-300 mt-2">
              <p>เลือกรูปแบบการใช้งานที่เหมาะสมกับคุณที่สุด (คุณสามารถเปลี่ยนได้ตลอดเวลา):</p>

              <div className="grid grid-cols-2 gap-3 pt-2">
                <button
                  onClick={() => handleSelectMode("simple")}
                  className={`p-4 rounded-xl border text-left space-y-2 transition-all tactile-btn ${
                    userMode === "simple"
                      ? "border-sky-500 bg-sky-950/40"
                      : "border-white/[0.08] bg-slate-900/60 hover:border-slate-500"
                  }`}
                >
                  <div className="font-bold text-sm text-white flex items-center space-x-1.5">
                    <CheckmarkCircle20Regular className="text-sky-400" />
                    <span>Simple Mode</span>
                  </div>
                  <p className="text-[11px] text-slate-400 leading-relaxed">
                    เหมาะสำหรับผู้ใช้ทั่วไป ซ่อนไฟล์ระบบที่ซับซ้อน ลบเฉพาะสิ่งที่ปลอดภัย พร้อมป้ายกำกับที่เข้าใจง่าย
                  </p>
                </button>

                <button
                  onClick={() => handleSelectMode("advanced")}
                  className={`p-4 rounded-xl border text-left space-y-2 transition-all tactile-btn ${
                    userMode === "advanced"
                      ? "border-indigo-500 bg-indigo-950/40"
                      : "border-white/[0.08] bg-slate-900/60 hover:border-slate-500"
                  }`}
                >
                  <div className="font-bold text-sm text-white flex items-center space-x-1.5">
                    <CheckmarkCircle20Regular className="text-indigo-400" />
                    <span>Advanced Mode</span>
                  </div>
                  <p className="text-[11px] text-slate-400 leading-relaxed">
                    สำหรับ Power User แสดงข้อมูลเชิงลึก ละเอียดทุก Attribute เข้าถึงการสแกน และคัดกรองขั้นสูง
                  </p>
                </button>
              </div>
            </DialogContent>
            <DialogActions className="mt-4">
              <Button appearance="primary" onClick={() => setShowOnboarding(false)} className="tactile-btn">
                เริ่มต้นใช้งาน
              </Button>
            </DialogActions>
          </DialogBody>
        </DialogSurface>
      </Dialog>
    </FluentProvider>
  );
}
