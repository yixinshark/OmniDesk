import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { WidgetLayout } from './GridEngine';

interface WidgetManifest {
  id: string;
  name: string;
  description: string;
  width: number;
  height: number;
  icon?: string;
}

interface WidgetDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  onAddWidget: (widget: WidgetLayout) => void;
  existingWidgetIds: string[];
}

export default function WidgetDrawer({ isOpen, onClose, onAddWidget, existingWidgetIds }: WidgetDrawerProps) {
  const [availableWidgets, setAvailableWidgets] = useState<WidgetManifest[]>([]);

  useEffect(() => {
    if (isOpen) {
      invoke<WidgetManifest[]>('list_available_widgets')
        .then(setAvailableWidgets)
        .catch(err => {
          console.error("获取可用组件列表失败:", err);
          // Fallback: 从 manifest.json 文件直接读取
          setAvailableWidgets([]);
        });
    }
  }, [isOpen]);

  const handleAdd = (manifest: WidgetManifest) => {
    // 找一个空位放置新组件
    const instanceCount = existingWidgetIds.filter(id => id === manifest.id).length;
    const instanceId = `${manifest.id}_${instanceCount + 1}`;

    // 简单策略：放在左上角空位
    const newWidget: WidgetLayout = {
      id: manifest.id,
      instanceId,
      x: 0,
      y: 0,
      w: manifest.width || 2,
      h: manifest.height || 2,
    };

    onAddWidget(newWidget);
  };

  if (!isOpen) return null;

  return (
    <>
      {/* 背景遮罩 */}
      <div
        className="fixed inset-0 z-50 bg-black/40 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* 抽屉面板 */}
      <div className="fixed right-0 top-0 h-full w-80 bg-black/75 backdrop-blur-3xl border-l border-white/10 z-50 flex flex-col shadow-[0_0_40px_rgba(0,0,0,0.5)]">
        {/* 头部 */}
        <div className="flex items-center justify-between p-4 border-b border-white/10">
          <h2 className="text-white text-lg font-bold">🧩 添加组件</h2>
          <button
            onClick={onClose}
            className="text-white/60 hover:text-white text-xl cursor-pointer"
          >
            ✕
          </button>
        </div>

        {/* 组件列表 */}
        <div className="flex-1 overflow-y-auto p-4 space-y-3">
          {availableWidgets.length === 0 ? (
            <div className="text-white/40 text-center py-8">
              暂无可用组件
            </div>
          ) : (
            availableWidgets.map((widget) => {
              const isAdded = existingWidgetIds.includes(widget.id);
              return (
                <div
                  key={widget.id}
                  className={`p-3 rounded-2xl border transition-all duration-300 cursor-pointer ${
                    isAdded
                      ? 'border-white/10 bg-white/5 opacity-70'
                      : 'border-white/10 bg-gradient-to-br from-white/10 to-white/5 hover:from-white/20 hover:to-white/10 hover:border-white/30 hover:shadow-[0_8px_20px_rgba(0,0,0,0.2)] hover:-translate-y-1'
                  }`}
                  onClick={() => !isAdded && handleAdd(widget)}
                >
                  <div className="flex items-center gap-3">
                    <div className="text-2xl">{widget.icon || '📦'}</div>
                    <div className="flex-1 min-w-0">
                      <div className="text-white font-medium text-sm truncate">
                        {widget.name || widget.id}
                      </div>
                      <div className="text-white/40 text-xs truncate">
                        {widget.description || `${widget.width}×${widget.height}`}
                      </div>
                    </div>
                    {isAdded ? (
                      <span className="text-xs text-white/30 bg-white/10 px-2 py-1 rounded-full">已添加</span>
                    ) : (
                      <span className="text-xs text-green-400 bg-green-400/10 px-2 py-1 rounded-full">+ 添加</span>
                    )}
                  </div>
                </div>
              );
            })
          )}
        </div>
      </div>
    </>
  );
}
