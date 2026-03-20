// ContextPaste -- Region Selector Overlay (Lightshot-style)
// Fullscreen transparent overlay for drag-selecting a screen region.
// On release, captures the region, OCRs text, and emits results.

import { useCallback, useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import type { RankedItem } from "../../lib/types";

interface SelectionRect {
  startX: number;
  startY: number;
  endX: number;
  endY: number;
}

export function RegionSelectorOverlay() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [selection, setSelection] = useState<SelectionRect | null>(null);
  const [processing, setProcessing] = useState(false);

  // Draw the overlay with selection rectangle
  const draw = useCallback((sel: SelectionRect | null) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const w = canvas.width;
    const h = canvas.height;

    // Clear
    ctx.clearRect(0, 0, w, h);

    // Light semi-transparent overlay — just enough to see the screen through
    ctx.fillStyle = "rgba(0, 0, 0, 0.15)";
    ctx.fillRect(0, 0, w, h);

    if (sel) {
      const x = Math.min(sel.startX, sel.endX);
      const y = Math.min(sel.startY, sel.endY);
      const rw = Math.abs(sel.endX - sel.startX);
      const rh = Math.abs(sel.endY - sel.startY);

      // Clear the selected region (show through)
      ctx.clearRect(x, y, rw, rh);

      // Blue border around selection
      ctx.strokeStyle = "#3b82f6";
      ctx.lineWidth = 2;
      ctx.strokeRect(x, y, rw, rh);

      // Dimensions label
      if (rw > 30 && rh > 10) {
        const label = `${Math.round(rw)} x ${Math.round(rh)}`;
        ctx.font = "12px sans-serif";
        ctx.fillStyle = "#3b82f6";
        ctx.fillText(label, x + 4, y - 6 > 0 ? y - 6 : y + rh + 16);
      }
    }
  }, []);

  // Set canvas size to screen size
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const dpr = window.devicePixelRatio || 1;
    canvas.width = window.screen.width * dpr;
    canvas.height = window.screen.height * dpr;
    canvas.style.width = `${window.screen.width}px`;
    canvas.style.height = `${window.screen.height}px`;
    const ctx = canvas.getContext("2d");
    if (ctx) ctx.scale(dpr, dpr);
    draw(null);
  }, [draw]);

  // Add transparent background class on mount
  useEffect(() => {
    document.body.classList.add("region-selector-mode");
    return () => document.body.classList.remove("region-selector-mode");
  }, []);

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
    setIsDragging(true);
    setSelection({ startX: e.clientX, startY: e.clientY, endX: e.clientX, endY: e.clientY });
  }, []);

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (!isDragging || !selection) return;
    const newSel = { ...selection, endX: e.clientX, endY: e.clientY };
    setSelection(newSel);
    draw(newSel);
  }, [isDragging, selection, draw]);

  const handleMouseUp = useCallback(async () => {
    if (!selection || !isDragging) return;
    setIsDragging(false);

    const x = Math.min(selection.startX, selection.endX);
    const y = Math.min(selection.startY, selection.endY);
    const w = Math.abs(selection.endX - selection.startX);
    const h = Math.abs(selection.endY - selection.startY);

    // Minimum selection size
    if (w < 10 || h < 10) {
      setSelection(null);
      draw(null);
      return;
    }

    setProcessing(true);

    // Hide the overlay so it doesn't appear in the screenshot
    await getCurrentWindow().hide();

    // Small delay for the window to hide
    await new Promise((r) => setTimeout(r, 100));

    try {
      // Multiply by devicePixelRatio for actual screen coordinates
      const dpr = window.devicePixelRatio || 1;
      const result = await invoke<[string, RankedItem[]]>("capture_and_ocr_region", {
        x: Math.round(x * dpr),
        y: Math.round(y * dpr),
        width: Math.round(w * dpr),
        height: Math.round(h * dpr),
      });

      const [ocrText, predictions] = result;

      // Send results to main window
      await emit("region-ocr-results", { text: ocrText, predictions });
    } catch (e) {
      console.error("Region OCR failed:", e);
    }

    setProcessing(false);
    setSelection(null);
  }, [selection, isDragging, draw]);

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        cursor: "crosshair",
        zIndex: 99999,
        background: "transparent",
      }}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
    >
      <canvas
        ref={canvasRef}
        style={{ position: "absolute", inset: 0 }}
      />
      {processing && (
        <div style={{
          position: "absolute",
          top: "50%",
          left: "50%",
          transform: "translate(-50%, -50%)",
          background: "rgba(0,0,0,0.8)",
          color: "white",
          padding: "12px 24px",
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
