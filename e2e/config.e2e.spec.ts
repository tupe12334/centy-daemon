import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { createTempProject, type TempProject } from './fixtures/temp-project.js';

/**
 * gRPC Plain Text Tests for Configuration Operations
 *
 * Tests direct gRPC calls for config and manifest management.
 */
describe('gRPC: Config Operations', () => {
  let project: TempProject;

  beforeEach(async () => {
    project = await createTempProject({ initialize: true });
  });

  afterEach(async () => {
    await project.cleanup();
  });

  describe('GetConfig', () => {
    it('should get project configuration', async () => {
      const config = await project.client.getConfig({
        projectPath: project.path,
      });

      expect(config).toBeDefined();
      expect(config.priorityLevels).toBeGreaterThan(0);
      expect(config.allowedStates).toBeDefined();
      expect(Array.isArray(config.allowedStates)).toBe(true);
    });

    it('should include default states', async () => {
      const config = await project.client.getConfig({
        projectPath: project.path,
      });

      expect(config.allowedStates).toContain('open');
      expect(config.allowedStates).toContain('closed');
    });

    it('should have default state', async () => {
      const config = await project.client.getConfig({
        projectPath: project.path,
      });

      expect(config.defaultState).toBeDefined();
      expect(config.allowedStates).toContain(config.defaultState);
    });
  });

  describe('UpdateConfig', () => {
    // Base config required by the daemon (allowedStates must not be empty)
    const baseConfig = {
      allowedStates: ['open', 'in-progress', 'closed'],
      defaultState: 'open',
      priorityLevels: 3,
    };

    it('should update priority levels', async () => {
      const result = await project.client.updateConfig({
        projectPath: project.path,
        config: {
          ...baseConfig,
          priorityLevels: 5,
        },
      });

      expect(result.success).toBe(true);
      expect(result.config?.priorityLevels).toBe(5);
    });

    it('should update allowed states', async () => {
      const newStates = ['todo', 'doing', 'done'];

      const result = await project.client.updateConfig({
        projectPath: project.path,
        config: {
          ...baseConfig,
          allowedStates: newStates,
          defaultState: 'todo',
        },
      });

      expect(result.success).toBe(true);
      expect(result.config?.allowedStates).toEqual(newStates);
    });

    it('should update state colors', async () => {
      const result = await project.client.updateConfig({
        projectPath: project.path,
        config: {
          ...baseConfig,
          stateColors: {
            open: '#00ff00',
            closed: '#ff0000',
          },
        },
      });

      expect(result.success).toBe(true);
      expect(result.config?.stateColors).toBeDefined();
    });

    it('should update priority colors', async () => {
      const result = await project.client.updateConfig({
        projectPath: project.path,
        config: {
          ...baseConfig,
          priorityColors: {
            '1': '#ff0000',
            '2': '#ffff00',
            '3': '#00ff00',
          },
        },
      });

      expect(result.success).toBe(true);
      expect(result.config?.priorityColors).toBeDefined();
    });

    it('should update LLM config', async () => {
      const result = await project.client.updateConfig({
        projectPath: project.path,
        config: {
          ...baseConfig,
          llm: {
            autoCloseOnComplete: true,
            updateStatusOnStart: true,
            allowDirectEdits: false,
          },
        },
      });

      expect(result.success).toBe(true);
      expect(result.config?.llm).toBeDefined();
      expect(result.config?.llm?.autoCloseOnComplete).toBe(true);
    });
  });

  describe('GetManifest', () => {
    it('should get project manifest', async () => {
      const manifest = await project.client.getManifest({
        projectPath: project.path,
      });

      expect(manifest).toBeDefined();
      expect(manifest.schemaVersion).toBeGreaterThan(0);
      expect(manifest.centyVersion).toBeDefined();
      expect(manifest.createdAt).toBeDefined();
      expect(manifest.updatedAt).toBeDefined();
    });

    it('should have valid timestamps', async () => {
      const manifest = await project.client.getManifest({
        projectPath: project.path,
      });

      // Verify timestamps are valid ISO dates
      expect(() => new Date(manifest.createdAt)).not.toThrow();
      expect(() => new Date(manifest.updatedAt)).not.toThrow();
    });
  });

  describe('GetProjectVersion', () => {
    it('should get project version info', async () => {
      const versionInfo = await project.client.getProjectVersion({
        projectPath: project.path,
      });

      expect(versionInfo).toBeDefined();
      expect(versionInfo.projectVersion).toBeDefined();
      expect(versionInfo.daemonVersion).toBeDefined();
      expect(versionInfo.comparison).toBeDefined();
    });

    it('should indicate version comparison status', async () => {
      const versionInfo = await project.client.getProjectVersion({
        projectPath: project.path,
      });

      // comparison should be one of: equal, project_behind, project_ahead
      expect(['equal', 'project_behind', 'project_ahead']).toContain(
        versionInfo.comparison
      );
    });
  });
});
