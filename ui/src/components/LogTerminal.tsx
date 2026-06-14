import "@xterm/xterm/css/xterm.css";
import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import { useEffect, useRef } from "react";

interface LogTerminalProps {
  /** Raw log text to display */
  content: string;
  /** Additional class name */
  className?: string;
}

export function LogTerminal({ content, className }: LogTerminalProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<Terminal | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    const terminal = new Terminal({
      theme: {
        background: "transparent",
        foreground: "#cccccc",
        cursor: "#ffffff",
        selectionBackground: "#264f78",
      },
      allowProposedApi: true,
      fontFamily:
        "'JetBrains Mono', 'Fira Code', 'Cascadia Code', Menlo, monospace",
      fontSize: 12,
      lineHeight: 1.4,
      scrollback: 10000,
      disableStdin: true,
      cursorBlink: false,
      cursorStyle: "bar",
      allowTransparency: true,
    });

    const fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);

    terminal.open(containerRef.current);
    fitAddon.fit();

    terminalRef.current = terminal;

    const observer = new ResizeObserver(() => {
      fitAddon.fit();
    });
    observer.observe(containerRef.current);

    return () => {
      observer.disconnect();
      terminal.dispose();
      terminalRef.current = null;
    };
  }, []);

  useEffect(() => {
    const terminal = terminalRef.current;
    if (!terminal) return;

    terminal.clear();
    if (content) {
      // Convert \n to \r\n for xterm.js
      terminal.write(content.replaceAll("\n", "\r\n"));
    }
  }, [content]);

  return (
    <div
      ref={containerRef}
      className={className}
      style={{ width: "100%", height: "100%" }}
    />
  );
}
