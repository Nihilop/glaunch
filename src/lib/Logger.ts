// logger.ts
import {useLogStore} from "@/stores/log.ts";

const originalConsole = {
  log: console.log,
  error: console.error,
  warn: console.warn,
  info: console.info
};

export function setupLogger() {
  console.log = (...args) => {
    // Garde une copie du log original
    originalConsole.log(...args);
    // Ajouter votre logique de capture ici
    handleLog('log', ...args);
  };

  console.error = (...args) => {
    originalConsole.error(...args);
    handleLog('error', ...args);
  };

  console.warn = (...args) => {
    originalConsole.warn(...args);
    handleLog('warn', ...args);
  };

  console.info = (...args) => {
    originalConsole.info(...args);
    handleLog('info', ...args);
  };
}

function handleLog(type: string, ...args: any[]) {
  const timestamp = new Date().toISOString();
  const message = args.map(arg =>
    typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
  ).join(' ');

  // Vous pouvez stocker les logs dans un state global (Pinia par exemple)
  useLogStore().addLog({
    type,
    message,
    timestamp
  });
}