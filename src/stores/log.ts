import { defineStore } from 'pinia';

interface Log {
  type: string;
  message: string;
  timestamp: string;
}

export const useLogStore = defineStore('logs', {
  state: () => ({
    logs: [] as Log[],
    maxLogs: 1000, // Limite pour éviter une surcharge mémoire
  }),

  actions: {
    addLog(log: Log) {
      this.logs.push(log);
      if (this.logs.length > this.maxLogs) {
        this.logs.shift();
      }
    },

    clearLogs() {
      this.logs = [];
    }
  }
});