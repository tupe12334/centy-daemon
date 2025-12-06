import { spawn, ChildProcess } from 'node:child_process';
import { mkdir, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { randomUUID } from 'node:crypto';

export interface DaemonInstance {
  process: ChildProcess;
  address: string;
  port: number;
  tempDir: string;
  stop: () => Promise<void>;
}

export interface DaemonManagerOptions {
  /** Path to the daemon binary */
  daemonBinaryPath?: string;
  /** Base port to start from (will find available port) */
  basePort?: number;
  /** Timeout for daemon startup in ms */
  startupTimeout?: number;
}

const DEFAULT_OPTIONS: Required<DaemonManagerOptions> = {
  daemonBinaryPath: join(process.cwd(), 'target/release/centy-daemon'),
  basePort: 50100,
  startupTimeout: 10000,
};

let portCounter = 0;

/**
 * Manages daemon instances for E2E testing.
 * Each test can get an isolated daemon with its own temp directory.
 */
export class DaemonManager {
  private options: Required<DaemonManagerOptions>;
  private instances: DaemonInstance[] = [];

  constructor(options: DaemonManagerOptions = {}) {
    this.options = { ...DEFAULT_OPTIONS, ...options };
  }

  /**
   * Start a new daemon instance with an isolated temp directory.
   */
  async start(): Promise<DaemonInstance> {
    const port = this.options.basePort + portCounter++;
    const address = `127.0.0.1:${port}`;

    // Create isolated temp directory for this daemon instance
    const tempDir = join(tmpdir(), `centy-e2e-${randomUUID()}`);
    await mkdir(tempDir, { recursive: true });

    const daemonProcess = spawn(this.options.daemonBinaryPath, [], {
      env: {
        ...process.env,
        CENTY_DAEMON_ADDR: address,
      },
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    // Wait for daemon to be ready
    await this.waitForDaemon(address, daemonProcess);

    const instance: DaemonInstance = {
      process: daemonProcess,
      address,
      port,
      tempDir,
      stop: async () => {
        await this.stopInstance(instance);
      },
    };

    this.instances.push(instance);
    return instance;
  }

  /**
   * Wait for daemon to be ready by checking if it accepts connections.
   */
  private async waitForDaemon(
    address: string,
    process: ChildProcess
  ): Promise<void> {
    const startTime = Date.now();

    return new Promise((resolve, reject) => {
      const checkInterval = setInterval(async () => {
        if (Date.now() - startTime > this.options.startupTimeout) {
          clearInterval(checkInterval);
          reject(new Error(`Daemon startup timeout at ${address}`));
          return;
        }

        try {
          // Try to connect to the daemon
          const { createGrpcClient } = await import('./grpc-client.js');
          const client = createGrpcClient(address);

          // Call GetDaemonInfo to check if ready
          await new Promise<void>((res, rej) => {
            client.getDaemonInfo({}, (err: Error | null) => {
              if (err) rej(err);
              else res();
            });
          });

          clearInterval(checkInterval);
          resolve();
        } catch {
          // Not ready yet, keep trying
        }
      }, 100);

      // Also listen for process errors
      process.on('error', (err) => {
        clearInterval(checkInterval);
        reject(new Error(`Daemon failed to start: ${err.message}`));
      });

      process.on('exit', (code) => {
        if (code !== 0) {
          clearInterval(checkInterval);
          reject(new Error(`Daemon exited with code ${code}`));
        }
      });
    });
  }

  /**
   * Stop a specific daemon instance and clean up.
   */
  private async stopInstance(instance: DaemonInstance): Promise<void> {
    // Try graceful shutdown via gRPC
    try {
      const { createGrpcClient } = await import('./grpc-client.js');
      const client = createGrpcClient(instance.address);

      await new Promise<void>((resolve) => {
        client.shutdown({ delaySeconds: 0 }, () => {
          resolve();
        });
      });

      // Wait a bit for graceful shutdown
      await new Promise((r) => setTimeout(r, 500));
    } catch {
      // Graceful shutdown failed, force kill
    }

    // Force kill if still running
    if (!instance.process.killed) {
      instance.process.kill('SIGTERM');
      await new Promise((r) => setTimeout(r, 100));

      if (!instance.process.killed) {
        instance.process.kill('SIGKILL');
      }
    }

    // Clean up temp directory
    try {
      await rm(instance.tempDir, { recursive: true, force: true });
    } catch {
      // Ignore cleanup errors
    }

    // Remove from instances list
    const index = this.instances.indexOf(instance);
    if (index > -1) {
      this.instances.splice(index, 1);
    }
  }

  /**
   * Stop all daemon instances.
   */
  async stopAll(): Promise<void> {
    await Promise.all(this.instances.map((i) => this.stopInstance(i)));
  }
}

// Singleton for shared daemon across tests
let sharedDaemon: DaemonInstance | null = null;
let sharedManager: DaemonManager | null = null;

/**
 * Get or start a shared daemon instance for tests that can share state.
 */
export async function getSharedDaemon(): Promise<DaemonInstance> {
  if (!sharedDaemon) {
    sharedManager = new DaemonManager();
    sharedDaemon = await sharedManager.start();
  }
  return sharedDaemon;
}

/**
 * Stop the shared daemon (call in globalTeardown).
 */
export async function stopSharedDaemon(): Promise<void> {
  if (sharedManager) {
    await sharedManager.stopAll();
    sharedDaemon = null;
    sharedManager = null;
  }
}

/**
 * Use existing daemon (e.g., manually started daemon).
 */
export function useExistingDaemon(
  address: string = '127.0.0.1:50051'
): DaemonInstance {
  return {
    process: null as unknown as ChildProcess,
    address,
    port: parseInt(address.split(':')[1]),
    tempDir: '',
    stop: async () => {
      /* no-op for external daemon */
    },
  };
}
