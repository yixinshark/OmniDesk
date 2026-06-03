import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./index.css";

// 使用自定义协议加载本地视频文件
function localVideoUrl(filename: string): string {
  return `local-video://localhost/${filename}`;
}
import GridEngine, { WidgetLayout } from "./components/GridEngine";
import WidgetDrawer from "./components/WidgetDrawer";
import SettingsPage from "./components/SettingsPage";

interface AppConfig {
  wallpaperType: 'static' | 'video';
  glassIntensity: number;
  activeWallpaper?: string;  // filename of current wallpaper
  dailyAutoFetch?: boolean;  // auto-fetch Bing daily wallpaper
  lastFetchDate?: string;    // last fetch date (YYYYMMDD)
}

const DEFAULT_CONFIG: AppConfig = {
  wallpaperType: 'static',
  glassIntensity: 30,
  dailyAutoFetch: true,
};

const DEFAULT_WIDGETS: WidgetLayout[] = [
  { id: "clock", instanceId: "clock_1", x: 2, y: 2, w: 4, h: 4 },
  { id: "system-monitor", instanceId: "sys_1", x: 6, y: 2, w: 4, h: 2 },
  { id: "todo", instanceId: "todo_1", x: 6, y: 4, w: 4, h: 4 },
  { id: "countdown", instanceId: "countdown_1", x: 10, y: 2, w: 2, h: 2 },
  { id: "shortcuts", instanceId: "shortcuts_1", x: 10, y: 4, w: 2, h: 2 },
  { id: "pomodoro", instanceId: "pomodoro_1", x: 2, y: 6, w: 2, h: 2 },
  { id: "prompt", instanceId: "prompt_1", x: 10, y: 6, w: 2, h: 2 },
  { id: "weather", instanceId: "weather_1", x: 12, y: 2, w: 2, h: 2 },
  { id: "github-monitor", instanceId: "github_1", x: 14, y: 2, w: 4, h: 4 },
  { id: "habit-tracker", instanceId: "habit_1", x: 4, y: 6, w: 2, h: 2 },
  { id: "ai-quota", instanceId: "quota_1", x: 12, y: 4, w: 2, h: 2 },
  { id: "linux-news", instanceId: "linux_1", x: 18, y: 6, w: 4, h: 4 },
  { id: "ai-news", instanceId: "news_1", x: 18, y: 2, w: 4, h: 4 },
  { id: "weibo-hot", instanceId: "weibo_1", x: 14, y: 6, w: 4, h: 4 }
];

function todayStr(): string {
  const d = new Date();
  return `${d.getFullYear()}${String(d.getMonth() + 1).padStart(2, '0')}${String(d.getDate()).padStart(2, '0')}`;
}

function App() {
  const [widgets, setWidgets] = useState<WidgetLayout[]>([]);
  const [config, setConfig] = useState<AppConfig>(DEFAULT_CONFIG);
  const [isDrawerOpen, setIsDrawerOpen] = useState(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [bgUrl, setBgUrl] = useState("");
  const [videoUrl, setVideoUrl] = useState('');
  const [isEditMode, setIsEditMode] = useState(false);
  const [contextMenu, setContextMenu] = useState<{x: number, y: number} | null>(null);
  const [isWidgetsHidden, setIsWidgetsHidden] = useState(false);

  // 启动时加载布局和配置
  useEffect(() => {
    invoke<WidgetLayout[] | null>('load_layout')
      .then(saved => {
        if (saved && saved.length > 0) setWidgets(saved);
        else setWidgets(DEFAULT_WIDGETS);
      })
      .catch(() => setWidgets(DEFAULT_WIDGETS));

    invoke<AppConfig>('read_config')
      .then(saved => {
        const merged = { ...DEFAULT_CONFIG, ...saved };
        setConfig(merged);
        // 加载已保存的壁纸
        loadSavedWallpaper(merged);
      })
      .catch(() => {});
  }, []);

  // 加载已保存的壁纸（从 activeWallpaper 配置）
  const loadSavedWallpaper = async (cfg: AppConfig) => {
    if (cfg.activeWallpaper) {
      try {
        const wallpapers = await invoke<Array<{filename: string, path: string, wtype: string}>>('list_wallpapers');
        const found = wallpapers.find(w => w.filename === cfg.activeWallpaper);
        if (found) {
          if (found.wtype === 'video') {
            setVideoUrl(localVideoUrl(found.filename));
          } else {
            const dataUrl = await invoke<string | null>('get_image_data_url', { path: found.path });
            if (dataUrl) setBgUrl(dataUrl);
          }
          return;
        }
      } catch {}
    }
    // 没有已保存的壁纸，首次自动获取
    fetchNewWallpaper(cfg.wallpaperType, cfg);
  };

  // 获取新壁纸并缓存
  const fetchNewWallpaper = useCallback(async (type?: 'static' | 'video', cfgOverride?: AppConfig) => {
    const wpType = type || config.wallpaperType;
    const cfg = cfgOverride || config;
    if (wpType === 'static') {
      try {
        const res = await invoke<string>('fetch_proxy', {
          url: 'https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=zh-CN',
          token: null
        });
        const data = JSON.parse(res);
        if (data.images && data.images[0]) {
          const realUrl = "https://www.bing.com" + data.images[0].url;
          const filename = data.images[0].enddate + '.jpg';
          const cachedPath = await invoke<string | null>('check_and_cache_image', { url: realUrl, filename });
          if (cachedPath) {
            const dataUrl = await invoke<string | null>('get_image_data_url', { path: cachedPath });
            if (dataUrl) setBgUrl(dataUrl);
            else setBgUrl(realUrl);
            // 更新配置：记录当前壁纸和获取日期
            const newCfg = { ...cfg, activeWallpaper: filename, lastFetchDate: todayStr() };
            setConfig(newCfg);
            invoke('write_config', { config: newCfg }).catch(() => {});
          } else {
            setBgUrl(realUrl);
          }
        }
      } catch (e) { console.error("获取静态壁纸失败:", e); }
    } else {
      try {
        const res = await invoke<string>('fetch_proxy', {
          url: 'http://a1.phobos.apple.com/us/r1000/000/Features/atv/AutumnResources/videos/entries.json',
          token: null
        });
        const data = JSON.parse(res);
        const allAssets = data.flatMap((d: any) => d.assets);
        const randomAsset = allAssets[Math.floor(Math.random() * allAssets.length)];
        const targetUrl = randomAsset.url;
        const filename = targetUrl.substring(targetUrl.lastIndexOf('/') + 1);
        const cachedPath = await invoke<string | null>('check_and_cache_video', { url: targetUrl, filename });
        if (cachedPath) {
          setVideoUrl(localVideoUrl(filename));
          const newCfg = { ...cfg, activeWallpaper: filename };
          setConfig(newCfg);
          invoke('write_config', { config: newCfg }).catch(() => {});
        } else {
          setVideoUrl(targetUrl);
        }
      } catch (e) { console.error("获取动态壁纸失败:", e); }
    }
  }, [config]);

  // 切换到已保存的壁纸
  const switchToWallpaper = useCallback(async (filename: string, path: string, wtype: string) => {
    if (wtype === 'video') {
      setVideoUrl(localVideoUrl(filename));
      setBgUrl('');
    } else {
      const dataUrl = await invoke<string | null>('get_image_data_url', { path });
      if (dataUrl) {
        setBgUrl(dataUrl);
      }
      setVideoUrl('');
    }
    const newCfg = { ...config, activeWallpaper: filename, wallpaperType: wtype as 'static' | 'video' };
    setConfig(newCfg);
    invoke('write_config', { config: newCfg }).catch(() => {});
  }, [config]);

  // 每日自动获取静态壁纸
  useEffect(() => {
    if (!config.dailyAutoFetch || config.wallpaperType !== 'static') return;
    const checkDaily = () => {
      const today = todayStr();
      if (config.lastFetchDate !== today) {
        fetchNewWallpaper('static');
      }
    };
    checkDaily();
    const timer = setInterval(checkDaily, 1000 * 60 * 60); // 每小时检查一次
    return () => clearInterval(timer);
  }, [config.dailyAutoFetch, config.wallpaperType, config.lastFetchDate, fetchNewWallpaper]);

  const handleConfigChange = useCallback((newConfig: AppConfig) => {
    setConfig(newConfig);
    invoke('write_config', { config: newConfig }).catch(err => console.error("保存配置失败:", err));
  }, []);

  const handleWidgetChange = (newWidgets: WidgetLayout[]) => {
    setWidgets(newWidgets);
    invoke('save_layout', { layout: newWidgets }).catch(err => console.error("保存布局失败:", err));
  };

  const handleDeleteWidget = (instanceId: string) => {
    handleWidgetChange(widgets.filter(w => w.instanceId !== instanceId));
  };

  const handleAddWidget = (newWidget: WidgetLayout) => {
    handleWidgetChange([...widgets, newWidget]);
    setIsDrawerOpen(false);
  };

  // 键盘快捷键
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.altKey && e.key.toLowerCase() === 'e') setIsEditMode(prev => !prev);
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  // 毛玻璃强度
  useEffect(() => {
    document.documentElement.style.setProperty('--glass-blur', `${config.glassIntensity}px`);
  }, [config.glassIntensity]);

  return (
    <main
      className="w-screen h-screen relative bg-cover bg-center overflow-hidden transition-all duration-1000 select-none"
      style={bgUrl ? { backgroundImage: `url('${bgUrl}')` } : { background: 'linear-gradient(135deg, #1e1e2f 0%, #2a2a4a 100%)' }}
      onDoubleClick={() => setIsWidgetsHidden(prev => !prev)}
      onContextMenu={(e) => { e.preventDefault(); setContextMenu({ x: e.clientX, y: e.clientY }); }}
    >
      {config.wallpaperType === 'video' && videoUrl && (
        <video
          key={videoUrl}
          src={videoUrl}
          autoPlay loop muted
          className="absolute inset-0 w-full h-full object-cover z-0 pointer-events-none"
        />
      )}

      <div className={`absolute inset-0 bg-black/30 pointer-events-none z-0 transition-opacity duration-500 ${isWidgetsHidden ? 'opacity-0' : 'opacity-100'}`} />

      {isEditMode && !isWidgetsHidden && (
        <div className="absolute top-0 left-0 w-full bg-blue-500/80 backdrop-blur text-white text-center py-2 z-50 font-bold shadow-lg transition-opacity flex items-center justify-center gap-4">
          <span>进入布局编辑模式：拖拽移动组件，点 × 删除。按 Alt+E 退出。</span>
          <button
            onClick={(e) => { e.stopPropagation(); setIsDrawerOpen(true); }}
            className="bg-white/20 hover:bg-white/30 px-3 py-1 rounded-full text-sm cursor-pointer transition-colors"
          >
            + 添加组件
          </button>
        </div>
      )}

      <div className={`relative z-10 w-full h-full transition-opacity duration-500 ${isWidgetsHidden ? 'opacity-0 pointer-events-none' : 'opacity-100'}`}>
        <GridEngine
          widgets={widgets}
          isEditMode={isEditMode}
          onWidgetChange={handleWidgetChange}
          onDeleteWidget={handleDeleteWidget}
        />
      </div>

      <div className={`absolute bottom-4 right-6 flex items-center gap-3 z-20 transition-opacity duration-500 ${isWidgetsHidden ? 'opacity-0 pointer-events-none' : 'opacity-100'}`}>
        <button onClick={() => setIsSettingsOpen(true)} className="text-white/40 hover:text-white/80 text-lg cursor-pointer transition-colors" title="设置">⚙️</button>
        <span className="text-white/30 text-sm pointer-events-none">OmniDesk v0.1.0</span>
      </div>

      {contextMenu && (
        <>
          <div className="fixed inset-0 z-40" onClick={() => setContextMenu(null)} onContextMenu={(e) => { e.preventDefault(); setContextMenu(null); }} />
          <div className="fixed z-50 bg-black/60 backdrop-blur-md border border-white/10 rounded-lg p-2 flex flex-col gap-1 text-sm shadow-2xl" style={{ left: contextMenu.x, top: contextMenu.y }}>
            <div className="px-4 py-2 hover:bg-white/10 rounded cursor-pointer text-white/90 transition-colors" onClick={() => { handleConfigChange({ ...config, wallpaperType: config.wallpaperType === 'static' ? 'video' : 'static' }); setContextMenu(null); }}>
              {config.wallpaperType === 'static' ? '📺 切换为动态壁纸' : '🖼️ 切换为静态壁纸'}
            </div>
            <div className="px-4 py-2 hover:bg-white/10 rounded cursor-pointer text-white/90 transition-colors" onClick={() => { setIsEditMode(!isEditMode); setContextMenu(null); }}>
              {isEditMode ? '✅ 退出编辑模式' : '📐 进入布局编辑模式'}
            </div>
            <div className="px-4 py-2 hover:bg-white/10 rounded cursor-pointer text-white/90 transition-colors" onClick={() => { setIsDrawerOpen(true); setContextMenu(null); }}>
              🧩 添加组件
            </div>
            <div className="px-4 py-2 hover:bg-white/10 rounded cursor-pointer text-white/90 transition-colors" onClick={() => { setIsSettingsOpen(true); setContextMenu(null); }}>
              ⚙️ 设置
            </div>
          </div>
        </>
      )}

      <WidgetDrawer isOpen={isDrawerOpen} onClose={() => setIsDrawerOpen(false)} onAddWidget={handleAddWidget} existingWidgetIds={widgets.map(w => w.id)} />
      <SettingsPage
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        widgets={widgets}
        onWidgetChange={handleWidgetChange}
        config={config}
        onConfigChange={handleConfigChange}
        onRefreshWallpaper={fetchNewWallpaper}
        onSwitchWallpaper={switchToWallpaper}
      />
    </main>
  );
}

export default App;
