import { describe, it, expect, afterAll } from 'vitest';
import { createGrpcClient, promisifyClient } from './fixtures/grpc-client.js';

/**
 * gRPC Plain Text Tests for Daemon Control
 *
 * These tests verify direct gRPC communication with the daemon
 * without going through the CLI or web app layers.
 */
describe('gRPC: Daemon Control', () => {
  const client = promisifyClient(createGrpcClient());

  afterAll(() => {
    client.close();
  });

  describe('GetDaemonInfo', () => {
    it('should return daemon version information', async () => {
      const info = await client.getDaemonInfo({});

      expect(info).toBeDefined();
      expect(info.version).toBeDefined();
      expect(typeof info.version).toBe('string');
      expect(info.version.length).toBeGreaterThan(0);
    });

    it('should return available versions for migration', async () => {
      const info = await client.getDaemonInfo({});

      expect(info.availableVersions).toBeDefined();
      expect(Array.isArray(info.availableVersions)).toBe(true);
    });
  });

  describe('Connection', () => {
    it('should handle multiple concurrent requests', async () => {
      const requests = Array.from({ length: 10 }, () =>
        client.getDaemonInfo({})
      );

      const results = await Promise.all(requests);

      results.forEach((info) => {
        expect(info.version).toBeDefined();
      });
    });

    it('should handle rapid sequential requests', async () => {
      for (let i = 0; i < 20; i++) {
        const info = await client.getDaemonInfo({});
        expect(info.version).toBeDefined();
      }
    });
  });
});
