/**
 * Widget SDK
 * 这是一个轻量级的垫片脚本，供第三方 Widget 引入。
 * 它在 iframe 内部代理了 window.__widgetBridge 对象，通过 postMessage 与宿主通信。
 */
(function() {
  const callbacks = new Map();
  const eventListeners = new Map();
  
  // 监听来自宿主 (Host) 的回调与事件广播
  window.addEventListener('message', (e) => {
    const data = e.data;
    if (!data) return;

    if (data.type === 'RESPONSE') {
      const cb = callbacks.get(data.reqId);
      if (cb) {
        if (data.error) cb.reject(data.error);
        else cb.resolve(data.payload);
        callbacks.delete(data.reqId);
      }
    } else if (data.type === 'EVENT') {
      const listeners = eventListeners.get(data.event) || [];
      listeners.forEach(fn => fn(data.payload));
    }
  });

  // 转发键盘事件给宿主，以防止焦点在 iframe 内时顶层无法响应快捷键
  window.addEventListener('keydown', (e) => {
    window.parent.postMessage({
      type: 'KEY_DOWN',
      key: e.key,
      altKey: e.altKey,
      ctrlKey: e.ctrlKey,
      shiftKey: e.shiftKey,
      metaKey: e.metaKey
    }, '*');
  });

  // 注入全局 API 供 Widget 业务代码调用
  window.__widgetBridge = {
    // 调用 Rust 后端能力
    invoke: (command, args) => {
      return new Promise((resolve, reject) => {
        // 生成唯一请求 ID
        const reqId = Math.random().toString(36).substring(2, 9);
        callbacks.set(reqId, { resolve, reject });
        
        // 发送给宿主
        window.parent.postMessage({ 
          type: 'INVOKE', 
          command, 
          args, 
          reqId 
        }, '*');
      });
    },
    
    // 监听全局事件（如配置更新、系统状态更新）
    on: (event, callback) => {
      if (!eventListeners.has(event)) {
        eventListeners.set(event, []);
      }
      eventListeners.get(event).push(callback);
      
      // 可以选择通知宿主，当前 Widget 订阅了某个高频事件，避免无效推送
      window.parent.postMessage({ 
        type: 'SUBSCRIBE', 
        event, 
        reqId: Math.random().toString(36).substring(2, 9) 
      }, '*');
    }
  };
})();
