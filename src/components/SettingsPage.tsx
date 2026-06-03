import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { WidgetLayout } from './GridEngine';

interface AppConfig {
  wallpaperType: 'static' | 'video';
  glassIntensity: number;
  activeWallpaper?: string;
  dailyAutoFetch?: boolean;
  lastFetchDate?: string;
}

interface WallpaperInfo {
  filename: string;
  path: string;
  wtype: string;
  size: number;
}

interface SettingsPageProps {
  isOpen: boolean;
  onClose: () => void;
  widgets: WidgetLayout[];
  onWidgetChange: (widgets: WidgetLayout[]) => void;
  config: AppConfig;
  onConfigChange: (config: AppConfig) => void;
  onRefreshWallpaper: (type?: 'static' | 'video') => Promise<void>;
  onSwitchWallpaper: (filename: string, path: string, wtype: string) => void;
}

type TabKey = 'appearance' | 'layout' | 'about';

const TABS: { key: TabKey; icon: string; label: string }[] = [
  { key: 'appearance', icon: '🎨', label: '外观' },
  { key: 'layout', icon: '📐', label: '布局' },
  { key: 'about', icon: 'ℹ️', label: '关于' },
];

export default function SettingsPage({ isOpen, onClose, widgets, onWidgetChange, config, onConfigChange, onRefreshWallpaper, onSwitchWallpaper }: SettingsPageProps) {
  const [activeTab, setActiveTab] = useState<TabKey>('appearance');
  const [refreshing, setRefreshing] = useState(false);
  const [wallpapers, setWallpapers] = useState<WallpaperInfo[]>([]);
  const [thumbUrls, setThumbUrls] = useState<Record<string, string>>({});

  const loadWallpapers = useCallback(async () => {
    try {
      const list = await invoke<WallpaperInfo[]>('list_wallpapers');
      setWallpapers(list);
      // Load preview images for all wallpapers
      const urls: Record<string, string> = {};
      for (const wp of list) {
        try {
          if (wp.wtype === 'video') {
            const thumbPath = await invoke<string | null>('generate_video_thumbnail', { filename: wp.filename });
            if (thumbPath) {
              const dataUrl = await invoke<string | null>('get_image_data_url', { path: thumbPath });
              if (dataUrl) urls[wp.filename] = dataUrl;
            }
          } else {
            const dataUrl = await invoke<string | null>('get_image_data_url', { path: wp.path });
            if (dataUrl) urls[wp.filename] = dataUrl;
          }
        } catch {}
      }
      setThumbUrls(urls);
    } catch {}
  }, []);

  useEffect(() => {
    if (isOpen) loadWallpapers();
  }, [isOpen, loadWallpapers]);

  if (!isOpen) return null;

  const handleRefresh = async (type?: 'static' | 'video') => {
    setRefreshing(true);
    try {
      await onRefreshWallpaper(type);
      setTimeout(() => loadWallpapers(), 1500); // 等下载完成后刷新列表
    } finally {
      setTimeout(() => setRefreshing(false), 800);
    }
  };

  const handleDelete = async (filename: string) => {
    if (!confirm(`确定要删除壁纸 ${filename} 吗？`)) return;
    try {
      await invoke('delete_wallpaper', { filename });
      loadWallpapers();
      // 如果删除的是当前壁纸，清除配置
      if (config.activeWallpaper === filename) {
        onConfigChange({ ...config, activeWallpaper: undefined });
      }
    } catch (e) {
      console.error('删除壁纸失败:', e);
    }
  };

  const staticWallpapers = wallpapers.filter(w => w.wtype === 'static');
  const videoWallpapers = wallpapers.filter(w => w.wtype === 'video');

  return (
    <>
      <div className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm" onClick={onClose} />

      <div className="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-50
                      w-[600px] h-[480px] bg-[#1a1a2e]/95 backdrop-blur-xl rounded-2xl
                      border border-white/10 shadow-2xl flex overflow-hidden">

        {/* 左侧导航 */}
        <div className="w-36 bg-black/20 border-r border-white/8 p-3 flex flex-col flex-shrink-0">
          <div className="text-white/80 text-xs font-bold mb-3 px-2">设置</div>
          <div className="space-y-0.5">
            {TABS.map(tab => (
              <button
                key={tab.key}
                onClick={() => setActiveTab(tab.key)}
                className={`w-full flex items-center gap-2 px-2.5 py-2 rounded-lg text-xs transition-all cursor-pointer ${
                  activeTab === tab.key
                    ? 'bg-white/12 text-white font-medium'
                    : 'text-white/45 hover:text-white/70 hover:bg-white/5'
                }`}
              >
                <span className="text-sm">{tab.icon}</span>
                <span>{tab.label}</span>
              </button>
            ))}
          </div>
          <div className="flex-1" />
          <button
            onClick={onClose}
            className="w-full flex items-center gap-2 px-2.5 py-2 rounded-lg text-xs
                       text-white/35 hover:text-white/60 hover:bg-white/5 transition-all cursor-pointer"
          >
            <span className="text-sm">✕</span>
            <span>关闭</span>
          </button>
        </div>

        {/* 右侧内容 */}
        <div className="flex-1 overflow-y-auto p-5">
          {activeTab === 'appearance' && (
            <div className="space-y-5">
              {/* 壁纸类型选择 + 获取 */}
              <section>
                <div className="text-white/50 text-[10px] font-semibold uppercase tracking-wider mb-2">壁纸来源</div>
                <div className="grid grid-cols-2 gap-2 mb-3">
                  {([
                    { key: 'static' as const, icon: '🖼️', title: '静态壁纸', desc: 'Bing 每日美图' },
                    { key: 'video' as const, icon: '📺', title: '动态壁纸', desc: 'Apple TV 航拍' },
                  ]).map(opt => (
                    <div
                      key={opt.key}
                      className={`p-3 rounded-xl border transition-all cursor-pointer ${
                        config.wallpaperType === opt.key
                          ? 'border-blue-500/60 bg-blue-500/10'
                          : 'border-white/8 bg-white/4 hover:bg-white/7'
                      }`}
                      onClick={() => onConfigChange({ ...config, wallpaperType: opt.key })}
                    >
                      <div className="flex items-center justify-between mb-1.5">
                        <div className="text-lg">{opt.icon}</div>
                        <button
                          onClick={(e) => { e.stopPropagation(); handleRefresh(opt.key); }}
                          disabled={refreshing}
                          className="text-white/40 hover:text-white/80 disabled:opacity-30 transition-all cursor-pointer"
                          title="获取新壁纸"
                        >
                          <svg className={`w-3.5 h-3.5 ${refreshing ? 'animate-spin' : ''}`} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                            <path d="M21 12a9 9 0 1 1-6.219-8.56" />
                            <polyline points="21 3 21 9 15 9" />
                          </svg>
                        </button>
                      </div>
                      <div className="text-white/80 text-xs font-medium">{opt.title}</div>
                      <div className="text-white/35 text-[10px] mt-0.5">{opt.desc}</div>
                    </div>
                  ))}
                </div>

                {/* 每日自动获取 */}
                {config.wallpaperType === 'static' && (
                  <div className="flex items-center justify-between px-3 py-2 rounded-lg bg-white/4 border border-white/8">
                    <span className="text-white/60 text-xs">每日自动更新壁纸</span>
                    <div
                      className={`w-8 h-4.5 rounded-full cursor-pointer transition-all relative ${config.dailyAutoFetch ? 'bg-blue-500' : 'bg-white/15'}`}
                      onClick={() => onConfigChange({ ...config, dailyAutoFetch: !config.dailyAutoFetch })}
                    >
                      <div className={`absolute top-0.5 w-3.5 h-3.5 bg-white rounded-full shadow transition-transform ${config.dailyAutoFetch ? 'translate-x-4' : 'translate-x-0.5'}`} />
                    </div>
                  </div>
                )}
              </section>

              {/* 壁纸画廊 */}
              <section>
                <div className="text-white/50 text-[10px] font-semibold uppercase tracking-wider mb-2">
                  已保存的壁纸 ({wallpapers.length})
                </div>

                {wallpapers.length === 0 ? (
                  <div className="text-white/25 text-xs text-center py-4">暂无已保存的壁纸</div>
                ) : (
                  <div className="max-h-52 overflow-y-auto pr-1 space-y-3">
                    {/* 静态壁纸 */}
                    {staticWallpapers.length > 0 && (
                      <div>
                        <div className="text-white/30 text-[9px] uppercase tracking-wider mb-1.5">静态壁纸 ({staticWallpapers.length})</div>
                        <div className="grid grid-cols-4 gap-1.5">
                          {staticWallpapers.map(wp => (
                            <div
                              key={wp.filename}
                              className={`group relative rounded-lg overflow-hidden cursor-pointer border transition-all ${
                                config.activeWallpaper === wp.filename
                                  ? 'border-blue-500 ring-1 ring-blue-500/50'
                                  : 'border-white/8 hover:border-white/20'
                              }`}
                              onClick={() => onSwitchWallpaper(wp.filename, wp.path, wp.wtype)}
                            >
                              {thumbUrls[wp.filename] ? (
                                <img
                                  src={thumbUrls[wp.filename]}
                                  alt={wp.filename}
                                  className="w-full h-16 object-cover"
                                />
                              ) : (
                                <div className="w-full h-16 bg-white/5 flex items-center justify-center text-lg">🖼️</div>
                              )}
                              <button
                                onClick={(e) => { e.stopPropagation(); handleDelete(wp.filename); }}
                                className="absolute top-1 right-1 opacity-0 group-hover:opacity-100 bg-red-500/80 text-white w-4 h-4 rounded-full text-[10px] flex items-center justify-center transition-opacity z-10"
                                title="删除"
                              >
                                ×
                              </button>
                              <div className="absolute bottom-0 left-0 right-0 bg-black/50 px-1 py-0.5 text-[8px] text-white/60 truncate">
                                {wp.filename.replace('.jpg', '')}
                              </div>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}

                    {/* 动态壁纸 */}
                    {videoWallpapers.length > 0 && (
                      <div>
                        <div className="text-white/30 text-[9px] uppercase tracking-wider mb-1.5">动态壁纸 ({videoWallpapers.length})</div>
                        <div className="grid grid-cols-4 gap-1.5">
                          {videoWallpapers.map(wp => (
                            <div
                              key={wp.filename}
                              className={`group relative rounded-lg overflow-hidden cursor-pointer border transition-all ${
                                config.activeWallpaper === wp.filename
                                  ? 'border-blue-500 ring-1 ring-blue-500/50'
                                  : 'border-white/8 hover:border-white/20'
                              }`}
                              onClick={() => onSwitchWallpaper(wp.filename, wp.path, wp.wtype)}
                            >
                              {thumbUrls[wp.filename] ? (
                                <img
                                  src={thumbUrls[wp.filename]}
                                  alt={wp.filename}
                                  className="w-full h-14 object-cover"
                                />
                              ) : (
                                <div className="w-full h-14 bg-white/5 flex items-center justify-center text-lg">🎬</div>
                              )}
                              <button
                                onClick={(e) => { e.stopPropagation(); handleDelete(wp.filename); }}
                                className="absolute top-1 right-1 opacity-0 group-hover:opacity-100 bg-red-500/80 text-white w-4 h-4 rounded-full text-[10px] flex items-center justify-center transition-opacity z-10"
                                title="删除"
                              >
                                ×
                              </button>
                              <div className="absolute bottom-0 left-0 right-0 bg-black/50 px-1 py-0.5 text-[8px] text-white/60 truncate">
                                {wp.filename.replace(/\.[^.]+$/, '')}
                              </div>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}
                  </div>
                )}
              </section>

              {/* 毛玻璃 */}
              <section>
                <div className="flex items-center justify-between mb-2">
                  <div className="text-white/50 text-[10px] font-semibold uppercase tracking-wider">毛玻璃强度</div>
                  <div className="text-white/40 text-[10px] tabular-nums">{config.glassIntensity}px</div>
                </div>
                <input
                  type="range"
                  min="0"
                  max="60"
                  step="2"
                  value={config.glassIntensity}
                  onChange={(e) => onConfigChange({ ...config, glassIntensity: Number(e.target.value) })}
                  className="w-full h-1.5 bg-white/10 rounded-full appearance-none cursor-pointer
                             [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3.5
                             [&::-webkit-slider-thumb]:h-3.5 [&::-webkit-slider-thumb]:bg-blue-500
                             [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:shadow-md
                             [&::-webkit-slider-thumb]:cursor-pointer"
                />
                <div className="flex justify-between text-[9px] text-white/20 mt-1">
                  <span>清晰</span>
                  <span>模糊</span>
                </div>
              </section>
            </div>
          )}

          {activeTab === 'layout' && (
            <div className="space-y-3">
              <div className="text-white/50 text-[10px] font-semibold uppercase tracking-wider mb-2">布局管理</div>
              <button
                onClick={() => {
                  const data = JSON.stringify(widgets, null, 2);
                  const blob = new Blob([data], { type: 'application/json' });
                  const url = URL.createObjectURL(blob);
                  const a = document.createElement('a');
                  a.href = url; a.download = 'omnidesk-layout.json'; a.click();
                  URL.revokeObjectURL(url);
                }}
                className="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl bg-white/4 border border-white/8
                           text-white/70 text-xs hover:bg-white/8 transition-all cursor-pointer"
              >
                <span className="text-sm">📤</span>
                <span>导出布局配置</span>
              </button>
              <label className="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl bg-white/4 border border-white/8
                                text-white/70 text-xs hover:bg-white/8 transition-all cursor-pointer block">
                <span className="text-sm">📥</span>
                <span>导入布局配置</span>
                <input type="file" accept=".json" className="hidden" onChange={(e) => {
                  const file = e.target.files?.[0];
                  if (!file) return;
                  const reader = new FileReader();
                  reader.onload = () => {
                    try {
                      const imported = JSON.parse(reader.result as string);
                      if (Array.isArray(imported)) onWidgetChange(imported);
                    } catch { alert('无效的布局文件'); }
                  };
                  reader.readAsText(file);
                }} />
              </label>
              <button
                onClick={() => { if (confirm('确定要恢复默认布局吗？')) window.location.reload(); }}
                className="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl bg-red-500/6 border border-red-500/12
                           text-red-400/70 text-xs hover:bg-red-500/10 transition-all cursor-pointer"
              >
                <span className="text-sm">🔄</span>
                <span>恢复默认布局</span>
              </button>
            </div>
          )}

          {activeTab === 'about' && (
            <div className="space-y-4">
              <div className="text-white/50 text-[10px] font-semibold uppercase tracking-wider">关于</div>
              <div className="text-center py-6">
                <div className="text-3xl mb-2">🌌</div>
                <div className="text-white/80 text-sm font-bold">OmniDesk</div>
                <div className="text-white/30 text-xs mt-1">v0.1.0</div>
                <div className="text-white/20 text-[10px] mt-3 leading-relaxed">
                  高度可定制的桌面微件引擎<br/>
                  Tauri + Rust + React
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </>
  );
}
