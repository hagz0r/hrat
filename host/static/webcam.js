class WebcamManager {
  constructor() {
    this.connections = new Map();
  }

  initWebcam(clientId) {
    if (this.connections.has(clientId)) {
      console.log(`[Webcam] Connection for ${clientId} already exists`);
      return;
    }

    const canvas = document.getElementById(`webcam-canvas-${clientId}`);
    const fullscreenCanvas = document.getElementById(
      `fullscreen-canvas-${clientId}`,
    );

    if (!canvas || !fullscreenCanvas) {
      console.error(`[Webcam] Canvas elements not found for ${clientId}`);
      return;
    }

    const ctx = canvas.getContext("2d");
    const fullscreenCtx = fullscreenCanvas.getContext("2d");

    const wsProtocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const wsUrl = `${wsProtocol}//${window.location.host}/ws/feed/${clientId}`;
    console.log(`[Webcam] Connecting to ${wsUrl}`);

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
          `[Webcam] ❌ Failed to process image for ${clientId}:`,
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
      console.log(`[Webcam] ✅ WebSocket connected for ${clientId}`);
    ws.onerror = (err) =>
      console.error(`[Webcam] ❌ WebSocket error for ${clientId}:`, err);
    ws.onclose = () =>
      console.warn(`[Webcam] ⭕ WebSocket closed for ${clientId}`);
    ws.onmessage = messageHandler;

    // Setup clean reconnection logic
    connectionData.reconnectInterval = setInterval(() => {
      if (ws.readyState === WebSocket.CLOSED) {
        console.log(`[Webcam] Reconnecting to ${clientId}...`);
        ws = new WebSocket(wsUrl);
        ws.binaryType = "arraybuffer";

        // Re-assign all handlers to the new socket instance
        ws.onopen = () =>
          console.log(`[Webcam] ✅ WebSocket reconnected for ${clientId}`);
        ws.onerror = (err) =>
          console.error(`[Webcam] ❌ Reconnect error for ${clientId}:`, err);
        ws.onclose = () =>
          console.warn(`[Webcam] ⭕ Reconnect attempt closed for ${clientId}`);
        ws.onmessage = messageHandler;

        connectionData.ws = ws;
      }
    }, 5000);

    this.connections.set(clientId, connectionData);
  }

  cleanup(clientId) {
    const connection = this.connections.get(clientId);
    if (connection) {
      console.log(`[Webcam] Cleaning up connection for ${clientId}`);
      clearInterval(connection.reconnectInterval);
      if (connection.ws && connection.ws.readyState === WebSocket.OPEN) {
        connection.ws.close();
      }
      this.connections.delete(clientId);
    }
  }

  openFullscreenWebcam(clientId) {
    const modal = document.getElementById(`fullscreen-webcam-${clientId}`);
    if (modal) modal.classList.remove("hidden");
  }

  closeFullscreenWebcam(clientId) {
    const modal = document.getElementById(`fullscreen-webcam-${clientId}`);
    if (modal) modal.classList.add("hidden");
  }

  hideWebcam(clientId) {
    const section = document.getElementById(`webcam-section-${clientId}`);
    const placeholder = document.getElementById(
      `webcam-placeholder-${clientId}`,
    );
    if (section && placeholder) {
      section.classList.add("hidden");
      placeholder.classList.remove("hidden");
    }
  }

  showWebcam(clientId) {
    const section = document.getElementById(`webcam-section-${clientId}`);
    const placeholder = document.getElementById(
      `webcam-placeholder-${clientId}`,
    );
    if (section && placeholder) {
      section.classList.remove("hidden");
      placeholder.classList.add("hidden");
    }
  }
}

// Global instance and functions
window.webcamManager = new WebcamManager();
window.openFullscreenWebcam = (id) =>
  window.webcamManager.openFullscreenWebcam(id);
window.closeFullscreenWebcam = (id) =>
  window.webcamManager.closeFullscreenWebcam(id);
window.hideWebcam = (id) => window.webcamManager.hideWebcam(id);
window.showWebcam = (id) => window.webcamManager.showWebcam(id);
window.initWebcam = (id) => window.webcamManager.initWebcam(id);
window.cleanupWebcam = (id) => window.webcamManager.cleanup(id);

document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    document.querySelectorAll('[id^="fullscreen-webcam-"]').forEach((modal) => {
      if (!modal.classList.contains("hidden")) {
        const clientId = modal.id.replace("fullscreen-webcam-", "");
        window.webcamManager.closeFullscreenWebcam(clientId);
      }
    });
  }
});
