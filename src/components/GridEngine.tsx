import { useState, useRef, useEffect } from 'react';
import IframeHost from './IframeHost';

export interface WidgetLayout {
  id: string;
  instanceId: string;
  x: number;
  y: number;
  w: number;
  h: number;
}

const COLUMNS = 24;
const GAP = 10;
const PADDING = 10; // 单边 padding，与 GAP 一致

// 默认 cellSize，在 ResizeObserver 触发前使用
const DEFAULT_CELL_SIZE = 60;

interface GridEngineProps {
  widgets: WidgetLayout[];
  isEditMode: boolean;
  onWidgetChange: (widgets: WidgetLayout[]) => void;
  onDeleteWidget?: (instanceId: string) => void;
}

export default function GridEngine({ widgets, isEditMode, onWidgetChange, onDeleteWidget }: GridEngineProps) {
  // 动态 cellSize，根据容器宽度自适应
  const [cellSize, setCellSize] = useState(DEFAULT_CELL_SIZE);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const computeCellSize = () => {
      const w = container.clientWidth;
      // cellSize = (容器宽度 - 左右padding - (列数-1) * gap) / 列数
      const size = (w - PADDING * 2 - (COLUMNS - 1) * GAP) / COLUMNS;
      setCellSize(Math.max(size, 40)); // 最小 40px，防止极端窄屏
    };

    computeCellSize();

    const observer = new ResizeObserver(() => computeCellSize());
    observer.observe(container);
    return () => observer.disconnect();
  }, []);

  // 拖拽状态
  const [dragState, setDragState] = useState<{
    id: string | null;
    deltaX: number;
    deltaY: number;
  }>({ id: null, deltaX: 0, deltaY: 0 });

  const dragRef = useRef({ startX: 0, startY: 0 });

  const onPointerDown = (e: React.PointerEvent, id: string) => {
    if (!isEditMode) return;
    // 捕获指针，这样拖动到屏幕外面也不会丢失事件
    e.currentTarget.setPointerCapture(e.pointerId);
    dragRef.current = { startX: e.clientX, startY: e.clientY };
    setDragState({ id, deltaX: 0, deltaY: 0 });
  };

  const onPointerMove = (e: React.PointerEvent) => {
    if (dragState.id) {
      setDragState(prev => ({
        ...prev,
        deltaX: e.clientX - dragRef.current.startX,
        deltaY: e.clientY - dragRef.current.startY,
      }));
    }
  };

  // 碰撞检测辅助函数
  const isOverlapping = (w1: WidgetLayout, w2: WidgetLayout) => {
    return w1.x < w2.x + w2.w &&
           w1.x + w1.w > w2.x &&
           w1.y < w2.y + w2.h &&
           w1.y + w1.h > w2.y;
  };

  // 向下挤压防重叠（自动补位）算法
  const resolveCollisions = (currentWidgets: WidgetLayout[], movedWidget: WidgetLayout): WidgetLayout[] => {
    let newWidgets = [...currentWidgets];
    let resolved = false;
    
    newWidgets.sort((a, b) => a.y === b.y ? a.x - b.x : a.y - b.y);

    while (!resolved) {
      resolved = true;
      for (let i = 0; i < newWidgets.length; i++) {
        for (let j = i + 1; j < newWidgets.length; j++) {
          const w1 = newWidgets[i];
          const w2 = newWidgets[j];
          if (isOverlapping(w1, w2)) {
            // 发生重叠，找出需要往下挤的那个组件
            let toMoveIndex = j;
            if (w2.instanceId === movedWidget.instanceId) {
              toMoveIndex = i;
            } else if (w1.instanceId !== movedWidget.instanceId) {
              // 如果两个都不是当前拖拽的，把原本就靠下（或者同高靠右）的那个继续往下挤
              toMoveIndex = (w1.y > w2.y || (w1.y === w2.y && w1.x > w2.x)) ? i : j;
            }
            newWidgets[toMoveIndex] = { ...newWidgets[toMoveIndex], y: newWidgets[toMoveIndex].y + 1 };
            resolved = false;
            break;
          }
        }
        if (!resolved) break;
      }
      if (!resolved) {
        newWidgets.sort((a, b) => a.y === b.y ? a.x - b.x : a.y - b.y);
      }
    }
    return newWidgets;
  };

  const onPointerUp = (e: React.PointerEvent, widget: WidgetLayout) => {
    if (dragState.id === widget.instanceId) {
      e.currentTarget.releasePointerCapture(e.pointerId);
      
      const cellTotalSize = cellSize + GAP;
      const deltaCellsX = Math.round(dragState.deltaX / cellTotalSize);
      const deltaCellsY = Math.round(dragState.deltaY / cellTotalSize);
      
      // 更新该 widget 的坐标
      if (deltaCellsX !== 0 || deltaCellsY !== 0) {
        let movedWidgetObj: WidgetLayout | null = null;
        let newWidgets = widgets.map(w => {
          if (w.instanceId === widget.instanceId) {
            movedWidgetObj = { 
              ...w, 
              // 防止拖出左上角边界
              x: Math.max(0, w.x + deltaCellsX), 
              y: Math.max(0, w.y + deltaCellsY) 
            };
            return movedWidgetObj;
          }
          return { ...w }; // 深拷贝避免原地修改
        });

        if (movedWidgetObj) {
          // 触发自动补位算法
          newWidgets = resolveCollisions(newWidgets, movedWidgetObj);
        }
        
        onWidgetChange(newWidgets);
      }
      
      setDragState({ id: null, deltaX: 0, deltaY: 0 });
    }
  };

  return (
    <div ref={containerRef} className="relative w-full h-full">
      {widgets.map((widget) => {
        // 计算标准位置（加上 padding 偏移，因为 absolute 定位相对容器外边缘）
        let left = PADDING + widget.x * (cellSize + GAP);
        let top = PADDING + widget.y * (cellSize + GAP);
        const width = widget.w * cellSize + (widget.w - 1) * GAP;
        const height = widget.h * cellSize + (widget.h - 1) * GAP;

        // 如果该组件正在被拖拽，加上实时的像素偏移量
        const isDragging = dragState.id === widget.instanceId;
        if (isDragging) {
          left += dragState.deltaX;
          top += dragState.deltaY;
        }

        return (
          <div
            key={widget.instanceId}
            // 只有非拖拽状态才加 transition，拖拽时必须跟手（无延迟）
            className={`absolute glass rounded-2xl overflow-hidden ${isDragging ? 'z-50 shadow-2xl scale-[1.02]' : 'transition-all duration-300 z-10'}`}
            style={{
              left: `${left}px`,
              top: `${top}px`,
              width: `${width}px`,
              height: `${height}px`,
            }}
          >
            <IframeHost 
              id={widget.id} 
              instanceId={widget.instanceId}
              src={`/widgets/${widget.id}/index.html?instanceId=${widget.instanceId}`}
            />

            {/* 编辑模式遮罩层：拦截所有的鼠标事件，防止穿透到 iframe 内部 */}
            {isEditMode && (
              <div
                className={`absolute inset-0 z-50 flex items-center justify-center backdrop-blur-sm bg-white/10
                           ${isDragging ? 'cursor-grabbing' : 'cursor-grab hover:bg-white/20'}`}
                onPointerDown={(e) => onPointerDown(e, widget.instanceId)}
                onPointerMove={onPointerMove}
                onPointerUp={(e) => onPointerUp(e, widget)}
                onPointerCancel={(e) => onPointerUp(e, widget)}
              >
                {/* 删除按钮 */}
                <button
                  className="absolute top-2 right-2 w-7 h-7 bg-red-500/80 hover:bg-red-500 text-white rounded-full flex items-center justify-center text-sm font-bold shadow-lg pointer-events-auto cursor-pointer transition-colors z-50"
                  onPointerDown={(e) => e.stopPropagation()}
                  onClick={(e) => {
                    e.stopPropagation();
                    onDeleteWidget?.(widget.instanceId);
                  }}
                >
                  ×
                </button>
                <div className="bg-black/50 text-white px-4 py-2 rounded-full font-bold shadow-lg pointer-events-none">
                  {isDragging ? '松开吸附' : '拖拽移动'}
                </div>
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
