// ContextPaste — Region Selector Overlay (Lightshot-style)
// Shows a captured screenshot of the screen as background, with drag-to-select.
// On release, OCRs the selected region and shows matched clipboard items.

import { useCallback, useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import type { RankedItem } from "../../lib/types";

interface SelectionRect {
  startX: number;
  startY: number;
  endX: number;
  endY: number;
}

export function RegionSelectorOverlay() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const bgImageRef = useRef<HTMLImageElement | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const selectionRef = useRef<SelectionRect | null>(null);
  const [processing, setProcessing] = useState(false);
  const [ready, setReady] = useState(false);

  // Draw the overlay: screenshot background + dim + selection cutout
  const draw = useCallback((sel: SelectionRect | null) => {
    const canvas = canvasRef.current;
    const bgImg = bgImageRef.current;
    if (!canvas || !bgImg) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const w = canvas.width;
    const h = canvas.height;

    // Draw the screenshot as background
    ctx.drawImage(bgImg, 0, 0, w, h);

    // Dim overlay on top
    ctx.fillStyle = "rgba(0, 0, 0, 0.25)";
    ctx.fillRect(0, 0, w, h);

    if (sel) {
      const x = Math.min(sel.startX, sel.endX);
      const y = Math.min(sel.startY, sel.endY);
      const rw = Math.abs(sel.endX - sel.startX);
      const rh = Math.abs(sel.endY - sel.startY);

      // Draw the clear (un-dimmed) region from the original screenshot
      ctx.drawImage(bgImg, x, y, rw, rh, x, y, rw, rh);

      // Blue border
      ctx.strokeStyle = "#3b82f6";
      ctx.lineWidth = 2;
      ctx.strokeRect(x, y, rw, rh);

      // Dimensions label
      if (rw > 30 && rh > 10) {
        const label = `${Math.round(rw)} × ${Math.round(rh)}`;
        ctx.font = "bold 13px sans-serif";
        ctx.fillStyle = "rgba(0,0,0,0.7)";
        const textY = y - 8 > 14 ? y - 8 : y + rh + 18;
        ctx.fillRect(x, textY - 14, ctx.measureText(label).width + 10, 20);
        ctx.fillStyle = "#3b82f6";
        ctx.fillText(label, x + 5, textY);
      }
    }

    // Help text at top center
    if (!sel) {
      const helpText = "Drag to select text on screen · ESC to cancel";
      ctx.font = "14px sans-serif";
      const tw = ctx.measureText(helpText).width;
      const cx = (w - tw) / 2;
      ctx.fillStyle = "rgba(0,0,0,0.7)";
      ctx.fillRect(cx - 10, 15, tw + 20, 30);
      ctx.fillStyle = "#ffffff";
      ctx.fillText(helpText, cx, 36);
    }
  }, []);

  // Listen for the screenshot from Rust
  useEffect(() => {
    const unlisten = listen<string>("screen-captured", (event) => {
      const b64 = event.payload;
      const img = new Image();
      img.onload = () => {
        bgImageRef.current = img;
        const canvas = canvasRef.current;
        if (canvas) {
          canvas.width = img.width;
          canvas.height = img.height;
          canvas.style.width = "100vw";
          canvas.style.height = "100vh";
        }
        setReady(true);
        draw(null);
      };
      img.src = `data:image/jpeg;base64,${b64}`;
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [draw]);

  // Also try to get screenshot on mount (in case event already fired)
  useEffect(() => {
    invoke<string>("capture_fullscreen").then((b64) => {
      const img = new Image();
      img.onload = () => {
        bgImageRef.current = img;
        const canvas = canvasRef.current;
        if (canvas) {
          canvas.width = img.width;
          canvas.height = img.height;
          canvas.style.width = "100vw";
          canvas.style.height = "100vh";
        }
        setReady(true);
        draw(null);
      };
      img.src = `data:image/jpeg;base64,${b64}`;
    }).catch(() => {});
  }, [draw]);

  // ESC to cancel
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        getCurrentWindow().hide();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (!ready) return;
    const sel = { startX: e.clientX, startY: e.clientY, endX: e.clientX, endY: e.clientY };
    selectionRef.current = sel;
    setIsDragging(true);
  }, [ready]);

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (!isDragging || !selectionRef.current) return;
    const newSel = { ...selectionRef.current, endX: e.clientX, endY: e.clientY };
    selectionRef.current = newSel;
    requestAnimationFrame(() => draw(newSel));
  }, [isDragging, draw]);

  const handleMouseUp = useCallback(async () => {
    const sel = selectionRef.current;
    if (!sel || !isDragging) return;
    setIsDragging(false);

    const x = Math.min(sel.startX, sel.endX);
    const y = Math.min(sel.startY, sel.endY);
    const w = Math.abs(sel.endX - sel.startX);
    const h = Math.abs(sel.endY - sel.startY);

    if (w < 10 || h < 10) {
      selectionRef.current = null;
      draw(null);
      return;
    }

    setProcessing(true);
    await getCurrentWindow().hide();
    await new Promise((r) => setTimeout(r, 50));

    try {
      const dpr = window.devicePixelRatio || 1;
      const result = await invoke<[string, RankedItem[]]>("capture_and_ocr_region", {
        x: Math.round(x * dpr),
        y: Math.round(y * dpr),
        width: Math.round(w * dpr),
        height: Math.round(h * dpr),
      });

      const [ocrText, predictions] = result;
      await emit("region-ocr-results", { text: ocrText, predictions });
    } catch (e) {
      console.error("Region OCR failed:", e);
    }

    setProcessing(false);
    selectionRef.current = null;
  }, [isDragging, draw]);

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        cursor: ready ? "crosshair" : "wait",
        zIndex: 99999,
        background: "#000",
        overflow: "hidden",
      }}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
    >
      <canvas
        ref={canvasRef}
        style={{ position: "absolute", inset: 0, width: "100vw", height: "100vh" }}
      />
      {!ready && (
        <div style={{
          position: "absolute",
          top: "50%",
          left: "50%",
          transform: "translate(-50%, -50%)",
          background: "rgba(0,0,0,0.8)",
          color: "white",
          padding: "16px 32px",
          borderRadius: "8px",
          fontSize: "14px",
          fontFamily: "sans-serif",
        }}>
          Capturing screen...
        </div>
      )}
      {processing && (
        <div style={{
          position: "absolute",
          top: "50%",
          left: "50%",
          transform: "translate(-50%, -50%)",
          background: "rgba(0,0,0,0.8)",
          color: "white",
          padding: "16px 32px",
          borderRadius: "8px",
          fontSize: "14px",
          fontFamily: "sans-serif",
        }}>
          Reading text...
        </div>
      )}
    </div>
  );
}
