import { useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

interface IframeHostProps {
  id: string;          // Widget 类型 ID，例如 'clock'
  instanceId: string;  // Widget 实例 ID，例如 'clock_1'
  src: string;         // Widget 的入口 URL
}

export default function IframeHost({ instanceId, src }: IframeHostProps) {
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const unlistenFns = useRef<Record<string, UnlistenFn>>({});

  useEffect(() => {
    // 监听来自 Iframe 的 postMessage 请求
    const handleMessage = async (event: MessageEvent) => {
      // 安全校验：确保消息来源于我们包裹的 iframe
      if (event.source !== iframeRef.current?.contentWindow) return;

      const data = event.data;
      if (!data || typeof data !== 'object') return;

      // 处理 invoke 调用请求
      if (data.type === 'INVOKE') {
        try {
          const result = await invoke(data.command, data.args);
          iframeRef.current?.contentWindow?.postMessage({
            type: 'RESPONSE',
            reqId: data.reqId,
            payload: result
          }, '*');
        } catch (error) {
          iframeRef.current?.contentWindow?.postMessage({
            type: 'RESPONSE',
            reqId: data.reqId,
            error: String(error)
          }, '*');
        }
      } 
      // 处理事件订阅请求
      else if (data.type === 'SUBSCRIBE') {
        const eventName = data.event;
        // 如果之前已经订阅过，先取消旧的订阅，防止闭包或引用失效
        if (unlistenFns.current[eventName]) {
          unlistenFns.current[eventName]();
        }
        
        const unlisten = await listen(eventName, (tauriEvent) => {
          iframeRef.current?.contentWindow?.postMessage({
            type: 'EVENT',
            event: eventName,
            payload: tauriEvent.payload
          }, '*');
        });
        unlistenFns.current[eventName] = unlisten;
      }
      // 处理来自 Iframe 的键盘事件穿透
      else if (data.type === 'KEY_DOWN') {
        const keyboardEvent = new KeyboardEvent('keydown', {
          key: data.key,
          altKey: data.altKey,
          ctrlKey: data.ctrlKey,
          shiftKey: data.shiftKey,
          metaKey: data.metaKey,
          bubbles: true,
          cancelable: true
        });
        window.dispatchEvent(keyboardEvent);
      }
    };

    window.addEventListener('message', handleMessage);
    
    // 清理函数
    return () => {
      window.removeEventListener('message', handleMessage);
      Object.values(unlistenFns.current).forEach(unlisten => unlisten());
    };
  }, []);

  return (
    <div className="w-full h-full relative group">
      <iframe
        ref={iframeRef}
        src={src}
        title={`widget-${instanceId}`}
        // 赋予基础的脚本执行权限，但禁止它跳出沙盒或访问顶层
        sandbox="allow-scripts allow-same-origin"
        className="w-full h-full border-none rounded-2xl pointer-events-auto"
      />
      
      {/* 遮罩层：当处于“编辑模式”时，可以激活这层遮罩拦截点击事件，方便拖拽 */}
      {/* <div className="absolute inset-0 z-10 bg-black/10 hidden group-hover:block" /> */}
    </div>
  );
}
