class RemoteScreenManager {
  constructor() {
    this.connections = new Map();
  }

  initRemoteScreen(clientId) {
    if (this.connections.has(clientId)) {
      console.log(`[RemoteScreen] Connection for ${clientId} already exists`);
      return;
    }

    const canvas = document.getElementById(`screen-canvas-${clientId}`);
    const fullscreenCanvas = document.getElementById(
      `fullscreen-screen-canvas-${clientId}`,
    );

    if (!canvas || !fullscreenCanvas) {
      console.error(`[RemoteScreen] Canvas elements not found for ${clientId}`);
      return;
    }

    const ctx = canvas.getContext("2d");
    const fullscreenCtx = fullscreenCanvas.getContext("2d");

    const wsProtocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const wsUrl = `${wsProtocol}//${window.location.host}/ws/screenfeed/${clientId}`;
    console.log(`[RemoteScreen] Connecting to ${wsUrl}`);

    // Define the message handler function once
    const messageHandler = async (event) => {
      if (!(event.data instanceof ArrayBuffer)) return;

      // The image data starts from the second byte (index 1), after the prefix.
      const imageBuffer = event.data.slice(1);
      const dataBlob = new Blob([imageBuffer], { type: "image/jpeg" });

      try {
        const image = await createImageBitmap(dataBlob);

        canvas.width = image.width;
        canvas.height = image.height;
        ctx.drawImage(image, 0, 0);

        fullscreenCanvas.width = image.width;
        fullscreenCanvas.height = image.height;
        fullscreenCtx.drawImage(image, 0, 0);

        image.close();
      } catch (e) {
        console.error(
          `[RemoteScreen] ❌ Failed to process image for ${clientId}:`,
          e,
        );
      }
    };

    let ws = new WebSocket(wsUrl);
    ws.binaryType = "arraybuffer";

    const connectionData = {
      ws: ws,
      reconnectInterval: null,
    };

    ws.onopen = () =>
      console.log(`[RemoteScreen] ✅ WebSocket connected for ${clientId}`);
    ws.onerror = (err) =>
      console.error(`[RemoteScreen] ❌ WebSocket error for ${clientId}:`, err);
    ws.onclose = () =>
      console.warn(`[RemoteScreen] ⭕ WebSocket closed for ${clientId}`);
    ws.onmessage = messageHandler;

    // Setup clean reconnection logic
    connectionData.reconnectInterval = setInterval(() => {
      if (ws.readyState === WebSocket.CLOSED) {
        console.log(`[RemoteScreen] Reconnecting to ${clientId}...`);
        ws = new WebSocket(wsUrl);
        ws.binaryType = "arraybuffer";

        // Re-assign all handlers to the new socket instance
        ws.onopen = () =>
          console.log(
            `[RemoteScreen] ✅ WebSocket reconnected for ${clientId}`,
          );
        ws.onerror = (err) =>
          console.error(
            `[RemoteScreen] ❌ Reconnect error for ${clientId}:`,
            err,
          );
        ws.onclose = () =>
          console.warn(
            `[RemoteScreen] ⭕ Reconnect attempt closed for ${clientId}`,
          );
        ws.onmessage = messageHandler;

        connectionData.ws = ws;
      }
    }, 5000);

    this.connections.set(clientId, connectionData);
  }

  cleanup(clientId) {
    const connection = this.connections.get(clientId);
    if (connection) {
      console.log(`[RemoteScreen] Cleaning up connection for ${clientId}`);
      clearInterval(connection.reconnectInterval);
      if (connection.ws && connection.ws.readyState === WebSocket.OPEN) {
        connection.ws.close();
      }
      this.connections.delete(clientId);
    }
  }

  openFullscreenScreen(clientId) {
    const modal = document.getElementById(`fullscreen-screen-${clientId}`);
    if (modal) modal.classList.remove("hidden");
  }

  closeFullscreenScreen(clientId) {
    const modal = document.getElementById(`fullscreen-screen-${clientId}`);
    if (modal) modal.classList.add("hidden");
  }
}

// Global instance and functions
window.remoteScreenManager = new RemoteScreenManager();
window.openFullscreenScreen = (id) =>
  window.remoteScreenManager.openFullscreenScreen(id);
window.closeFullscreenScreen = (id) =>
  window.remoteScreenManager.closeFullscreenScreen(id);
window.initRemoteScreen = (id) =>
  window.remoteScreenManager.initRemoteScreen(id);
window.cleanupRemoteScreen = (id) => window.remoteScreenManager.cleanup(id);

document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    document.querySelectorAll('[id^="fullscreen-screen-"]').forEach((modal) => {
      if (!modal.classList.contains("hidden")) {
        const clientId = modal.id.replace("fullscreen-screen-", "");
        window.remoteScreenManager.closeFullscreenScreen(clientId);
      }
    });
  }
});
