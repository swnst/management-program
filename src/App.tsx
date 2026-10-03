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
  Dismiss20Regular,
  CheckmarkCircle20Regular,
  Filter20Regular,
  ShieldCheckmark24Regular,
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

// Mocked Explorer item structure for the virtualized/table grid
interface FileRow {
  id: string;
  name: string;
  path: string;
  size_bytes: number;
  created_year: number;
  age_label: string;
  is_dir: boolean;
  label: "Safe" | "Review" | "Keep";
  reason: string;
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

  // Sample items for table display
  const [fileList, setFileList] = useState<FileRow[]>([]);

  useEffect(() => {
    // Check onboarding
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

      // Auto-scan first volume
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
      setStatusText(
        `สแกนเสร็จสิ้น (${res.is_cached ? "จาก Cache" : "สแกนสด"}): ตรวจพบ ${res.total_files.toLocaleString()} ไฟล์ (${formatBytes(
          res.total_bytes
        )})`
      );

      // Generate visual sample breakdown
      generateSampleFiles(driveLetter);
    } catch (e) {
      setStatusText(`การสแกนขัดข้อง: ${String(e)}`);
    } finally {
      setIsScanningDisk(false);
    }
  };

  const generateSampleFiles = (driveLetter: string) => {
    const samples: FileRow[] = [
      {
        id: "1",
        name: "Windows.iso",
        path: `${driveLetter}:\\Downloads\\Windows11_Install.iso`,
        size_bytes: 5800000000,
        created_year: 2024,
        age_label: "2 ปี 1 เดือน",
        is_dir: false,
        label: "Review",
        reason: "ไฟล์ Installer เก่าใน Downloads ที่ไม่ได้เปิดนาน",
      },
      {
        id: "2",
        name: "PackageCache",
        path: `${driveLetter}:\\ProgramData\\Package Cache`,
        size_bytes: 3200000000,
        created_year: 2023,
        age_label: "3 ปี",
        is_dir: true,
        label: "Safe",
        reason: "Installer cache ของโปรแกรมที่ติดตั้งสำเร็จแล้ว",
      },
      {
        id: "3",
        name: "NodeModulesArchive.zip",
        path: `${driveLetter}:\\Projects\\OldApp\\node_modules.zip`,
        size_bytes: 1800000000,
        created_year: 2022,
        age_label: "4 ปี",
        is_dir: false,
        label: "Review",
        reason: "ไฟล์บีบอัดขนาดใหญ่ในโฟลเดอร์โปรเจกต์เก่า",
      },
      {
        id: "4",
        name: "NVIDIA_DXCache",
        path: `${driveLetter}:\\Users\\AppData\\Local\\NVIDIA\\DXCache`,
        size_bytes: 950000000,
        created_year: 2025,
        age_label: "8 เดือน",
        is_dir: true,
        label: "Safe",
        reason: "Shader cache ของการ์ดจอ สามารถลบได้ระบบจะสร้างใหม่เมื่อเล่นเกม",
      },
      {
        id: "5",
        name: "ScreenRecording_4K.mp4",
        path: `${driveLetter}:\\Videos\\Captures\\gameplay.mp4`,
        size_bytes: 4200000000,
        created_year: 2024,
        age_label: "1 ปี 6 เดือน",
        is_dir: false,
        label: "Keep",
        reason: "ไฟล์วิดีโอส่วนบุคคล",
      },
    ];

    setFileList(samples);
  };

  const handleSelectMode = (mode: "simple" | "advanced") => {
    setUserMode(mode);
    localStorage.setItem("lumen_user_mode", mode);
    setShowOnboarding(false);
  };

  const handleRecycleFile = async (path: string) => {
    try {
      await invoke("recycle_file_or_folder", { path });
      setStatusText(`ย้าย "${path}" ไปยัง Windows Recycle Bin สำเร็จ (กู้คืนได้เสมอ)`);
      setFileList((prev) => prev.filter((f) => f.path !== path));
    } catch (e) {
      setStatusText(`ล้มเหลว: ${String(e)}`);
    }
  };

  return (
    <FluentProvider theme={webDarkTheme} className="h-screen w-screen flex flex-col bg-[#141414] text-[#f5f5f5] select-none font-sans overflow-hidden">
      {/* Top Header / App Titlebar */}
      <header className="h-12 border-b border-[#292929] px-5 flex items-center justify-between bg-[#1f1f1f]/80 backdrop-blur-md">
        <div className="flex items-center space-x-3">
          <div className="h-6 w-6 rounded-md bg-gradient-to-tr from-sky-500 to-indigo-600 flex items-center justify-center font-bold text-xs text-white shadow-sm">
            L
          </div>
          <span className="font-semibold text-sm tracking-tight text-white">Lumen</span>
          <Badge appearance="tint" color={userMode === "advanced" ? "informative" : "brand"}>
            {userMode === "advanced" ? "Advanced Mode" : "Simple Mode"}
          </Badge>
        </div>

        <div className="flex items-center space-x-4 text-xs text-neutral-400">
          <div className="flex items-center space-x-2">
            <span className="h-2 w-2 rounded-full bg-emerald-500"></span>
            <span>{statusText}</span>
          </div>
          <Button
            size="small"
            appearance="subtle"
            icon={<Settings24Regular />}
            onClick={() => setShowOnboarding(true)}
          >
            ตั้งค่าโหมด
          </Button>
        </div>
      </header>

      {/* Main Container */}
      <div className="flex-1 flex overflow-hidden">
        {/* Navigation Sidebar */}
        <aside className="w-56 border-r border-[#242424] bg-[#1a1a1a]/70 p-3 flex flex-col justify-between">
          <TabList
            selectedValue={selectedTab}
            onTabSelect={(_, data) => setSelectedTab(data.value as string)}
            vertical
            className="space-y-1"
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
            <Tab value="memory" icon={<Apps24Regular />}>
              วิเคราะห์แรม & สตาร์ตอัป
            </Tab>
          </TabList>

          <div className="p-3 border-t border-[#292929] text-[11px] text-neutral-500 space-y-1">
            <div className="flex items-center space-x-1 text-emerald-400 font-medium">
              <ShieldCheckmark24Regular className="w-4 h-4" />
              <span>Recycle Bin Protection</span>
            </div>
            <div>ข้อมูลปลอดภัย ไม่ลบทิ้งสุ่มสี่สุ่มห้า</div>
          </div>
        </aside>

        {/* Content View */}
        <main className="flex-1 overflow-y-auto p-6 bg-[#121212]/90">
          {selectedTab === "overview" && (
            <div className="max-w-4xl space-y-6">
              <div>
                <h1 className="text-xl font-bold text-white tracking-tight">ภาพรวมระบบและพื้นที่จัดเก็บ</h1>
                <p className="text-xs text-neutral-400 mt-1">
                  ตรวจสอบความจุของดิสก์จริงจากระบบปฏิบัติการ Windows และสถานะการจัดเก็บ
                </p>
              </div>

              {/* Real Drives Grid */}
              <div className="grid grid-cols-2 gap-4">
                {disks.map((d) => {
                  const used = d.total_bytes - d.free_bytes;
                  const pct = d.total_bytes > 0 ? (used / d.total_bytes) * 100 : 0;
                  return (
                    <Card key={d.volume_letter} className="bg-[#1e1e1e] border border-[#2b2b2b] p-4 space-y-3">
                      <CardHeader
                        header={<span className="font-semibold text-white">{d.volume_name} ({d.volume_letter}:)</span>}
                        description={<span>ระบบไฟล์ {d.is_ntfs ? "NTFS" : "FAT32"} {d.is_system ? "• ไดรฟ์หลักของระบบ" : ""}</span>}
                        action={
                          <Button
                            size="small"
                            appearance="primary"
                            icon={<ArrowClockwise20Regular />}
                            disabled={isScanningDisk}
                            onClick={() => runVolumeScan(d.volume_letter)}
                          >
                            สแกนไดรฟ์
                          </Button>
                        }
                      />
                      <div className="space-y-1">
                        <div className="flex justify-between text-xs text-neutral-300">
                          <span>ว่าง {formatBytes(d.free_bytes)}</span>
                          <span>ใช้ไป {formatBytes(used)} / {formatBytes(d.total_bytes)}</span>
                        </div>
                        <ProgressBar value={pct / 100} color={pct > 90 ? "error" : "brand"} />
                      </div>
                    </Card>
                  );
                })}
              </div>

              {/* Scan Status Summary Card */}
              {scanResult && (
                <div className="p-4 rounded-xl border border-sky-900/40 bg-sky-950/20 flex items-center justify-between">
                  <div>
                    <h3 className="font-semibold text-sm text-sky-300">ผลการวิเคราะห์โครงสร้างไฟล์ไดรฟ์ {scanResult.volume_letter}:</h3>
                    <p className="text-xs text-neutral-400 mt-0.5">
                      สำรวจพบทั้งหมด {scanResult.total_files.toLocaleString()} ไฟล์ ใน {scanResult.total_dirs.toLocaleString()} โฟลเดอร์ ({formatBytes(scanResult.total_bytes)})
                    </p>
                  </div>
                  <Button
                    appearance="outline"
                    onClick={() => setSelectedTab("explorer")}
                  >
                    เปิดตารางแยกตามอายุ/ขนาด
                  </Button>
                </div>
              )}
            </div>
          )}

          {selectedTab === "explorer" && (
            <div className="max-w-5xl space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <h1 className="text-xl font-bold text-white tracking-tight">สำรวจพื้นที่และอายุของไฟล์ในเครื่อง</h1>
                  <p className="text-xs text-neutral-400 mt-0.5">
                    จัดลำดับไฟล์ตามขนาดใหญ่ที่สุด หรือเรียงตามปีที่ไฟล์เข้ามาอยู่ในเครื่องเพื่อให้คุณตัดสินใจเองได้
                  </p>
                </div>

                <div className="flex items-center space-x-2">
                  <Button
                    size="small"
                    appearance={explorerSort === "size" ? "primary" : "subtle"}
                    onClick={() => setExplorerSort("size")}
                  >
                    เรียงตามขนาด (ใหญ่ไปเล็ก)
                  </Button>
                  <Button
                    size="small"
                    appearance={explorerSort === "year" ? "primary" : "subtle"}
                    onClick={() => setExplorerSort("year")}
                  >
                    เรียงตามปี (เก่าที่สุดก่อน)
                  </Button>
                </div>
              </div>

              {/* Data Table */}
              <div className="rounded-xl border border-[#2b2b2b] bg-[#1a1a1a] overflow-hidden">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHeaderCell>ชื่อไฟล์ / โฟลเดอร์</TableHeaderCell>
                      <TableHeaderCell>ขนาด</TableHeaderCell>
                      <TableHeaderCell>อยู่ในเครื่องมานาน</TableHeaderCell>
                      <TableHeaderCell>การประเมินความปลอดภัย</TableHeaderCell>
                      <TableHeaderCell>การจัดการ</TableHeaderCell>
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
                        <TableRow key={row.id}>
                          <TableCell>
                            <TableCellLayout>
                              <div className="flex flex-col">
                                <span className="font-medium text-white">{row.name}</span>
                                <span className="text-neutral-500 text-[11px] truncate max-w-xs">{row.path}</span>
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
                              <span className="text-neutral-200">{row.age_label}</span>
                              <span className="text-neutral-500 block text-[10px]">ตั้งแต่ปี {row.created_year}</span>
                            </div>
                          </TableCell>
                          <TableCell>
                            <div className="space-y-0.5">
                              <Badge
                                appearance="tint"
                                color={
                                  row.label === "Safe"
                                    ? "success"
                                    : row.label === "Review"
                                    ? "warning"
                                    : "informative"
                                }
                              >
                                {row.label === "Safe" ? "ปลอดภัยที่จะลบ" : row.label === "Review" ? "ควรตรวจสอบก่อน" : "ควรเก็บไว้"}
                              </Badge>
                              <span className="text-[10px] text-neutral-400 block max-w-xs truncate">
                                {row.reason}
                              </span>
                            </div>
                          </TableCell>
                          <TableCell>
                            <Button
                              size="small"
                              appearance="subtle"
                              icon={<Delete20Regular />}
                              onClick={() => handleRecycleFile(row.path)}
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

          {selectedTab === "cleaner" && (
            <div className="max-w-4xl space-y-6">
              <div>
                <h1 className="text-xl font-bold text-white tracking-tight">ทำความสะอาดไฟล์ขยะระบบ</h1>
                <p className="text-xs text-neutral-400 mt-1">
                  ล้างไฟล์ Temp แคชไดรเวอร์การ์ดจอ และไฟล์รีพอร์ตที่ไม่จำเป็นอย่างปลอดภัย
                </p>
              </div>

              <div className="p-6 rounded-xl border border-[#2b2b2b] bg-[#1a1a1a] flex flex-col items-center justify-center text-center space-y-4">
                <Flash24Regular className="w-10 h-10 text-amber-400" />
                <div className="max-w-md">
                  <h3 className="text-base font-semibold text-white">วิเคราะห์ขยะระบบและ Shader Cache</h3>
                  <p className="text-xs text-neutral-400 mt-1">
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
                    >
                      ยืนยันการล้าง ({formatBytes(cleanPlan.safe_bytes)})
                    </Button>
                  </div>

                  <div className="border border-[#2b2b2b] rounded-xl overflow-hidden divide-y divide-[#242424] bg-[#1a1a1a]">
                    {cleanPlan.items.map((item) => (
                      <div key={item.id} className="p-4 flex items-center justify-between">
                        <div>
                          <span className="font-semibold text-sm text-neutral-200">{item.title}</span>
                          <p className="text-xs text-neutral-400 mt-0.5">{item.description}</p>
                        </div>
                        <div className="text-right">
                          <span className="font-mono text-sm text-sky-400 font-bold">{formatBytes(item.matched_bytes)}</span>
                          <span className="block text-[11px] text-neutral-500">{item.matched_files_count} ไฟล์</span>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}

          {selectedTab === "memory" && (
            <div className="max-w-4xl space-y-6">
              <div className="flex items-center justify-between">
                <div>
                  <h1 className="text-xl font-bold text-white tracking-tight">การทำงานของหน่วยความจำ (RAM) และ Startup Apps</h1>
                  <p className="text-xs text-neutral-400 mt-1">
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
                >
                  รีเฟรชสถิติ
                </Button>
              </div>

              {memory && (
                <Card className="bg-[#1a1a1a] border border-[#2b2b2b] p-5 space-y-4">
                  <div className="flex justify-between items-center text-xs text-neutral-300">
                    <span className="font-semibold text-white">การใช้งาน RAM จริง</span>
                    <span className="font-mono">{formatBytes(memory.used_ram_bytes)} / {formatBytes(memory.total_ram_bytes)} ({Math.round((memory.used_ram_bytes / memory.total_ram_bytes) * 100)}%)</span>
                  </div>
                  <ProgressBar value={memory.used_ram_bytes / memory.total_ram_bytes} />

                  <div className="pt-2 border-t border-[#2b2b2b] flex items-center justify-between">
                    <div>
                      <span className="text-xs font-semibold text-white block">Standby Memory Cache Purge</span>
                      <span className="text-[11px] text-neutral-400">
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
                    >
                      เคลียร์ Standby Cache
                    </Button>
                  </div>
                </Card>
              )}

              {/* Process List */}
              <div className="space-y-3">
                <h3 className="text-sm font-semibold text-white">โปรเซสที่ใช้หน่วยความจำสูงสุด</h3>
                <div className="rounded-xl border border-[#2b2b2b] bg-[#1a1a1a] divide-y divide-[#242424] overflow-hidden">
                  {memory?.top_processes.slice(0, 8).map((p) => (
                    <div key={p.pid} className="px-4 py-3 flex items-center justify-between text-sm">
                      <div className="flex items-center space-x-3">
                        <span className="font-mono text-xs text-neutral-500">#{p.pid}</span>
                        <span className="font-medium text-white">{p.name}</span>
                      </div>
                      <div className="flex items-center space-x-4">
                        <span className="text-xs text-neutral-400 font-mono">{p.cpu_percent.toFixed(1)}% CPU</span>
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
        <DialogSurface className="bg-[#1c1c1c] text-white border border-[#333] max-w-lg">
          <DialogBody>
            <DialogTitle>ยินดีต้อนรับสู่ Lumen</DialogTitle>
            <DialogContent className="space-y-4 text-xs text-neutral-300 mt-2">
              <p>เลือกรูปแบบการใช้งานที่เหมาะสมกับคุณที่สุด (คุณสามารถเปลี่ยนได้ตลอดเวลา):</p>

              <div className="grid grid-cols-2 gap-3 pt-2">
                <button
                  onClick={() => handleSelectMode("simple")}
                  className={`p-4 rounded-xl border text-left space-y-2 transition-all ${
                    userMode === "simple"
                      ? "border-sky-500 bg-sky-950/30"
                      : "border-[#333] bg-[#222] hover:border-neutral-500"
                  }`}
                >
                  <div className="font-bold text-sm text-white flex items-center space-x-1">
                    <CheckmarkCircle20Regular className="text-sky-400" />
                    <span>Simple Mode</span>
                  </div>
                  <p className="text-[11px] text-neutral-400 leading-relaxed">
                    เหมาะสำหรับผู้ใช้ทั่วไป ซ่อนไฟล์ระบบและไฟล์ซ่อนที่ซับซ้อน ลบเฉพาะสิ่งที่ปลอดภัย พร้อมป้ายกำกับที่เข้าใจง่าย
                  </p>
                </button>

                <button
                  onClick={() => handleSelectMode("advanced")}
                  className={`p-4 rounded-xl border text-left space-y-2 transition-all ${
                    userMode === "advanced"
                      ? "border-indigo-500 bg-indigo-950/30"
                      : "border-[#333] bg-[#222] hover:border-neutral-500"
                  }`}
                >
                  <div className="font-bold text-sm text-white flex items-center space-x-1">
                    <CheckmarkCircle20Regular className="text-indigo-400" />
                    <span>Advanced Mode</span>
                  </div>
                  <p className="text-[11px] text-neutral-400 leading-relaxed">
                    สำหรับ Power User แสดงข้อมูลเชิงลึก ละเอียดทุก Attribute, เข้าถึงการสแกนลึก MFT, ควบคุม Startup และคัดกรองขั้นสูง
                  </p>
                </button>
              </div>
            </DialogContent>
            <DialogActions className="mt-4">
              <Button appearance="primary" onClick={() => setShowOnboarding(false)}>
                เริ่มต้นใช้งาน
              </Button>
            </DialogActions>
          </DialogBody>
        </DialogSurface>
      </Dialog>
    </FluentProvider>
  );
}
