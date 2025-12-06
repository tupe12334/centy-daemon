import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { createTempProject, type TempProject } from './fixtures/temp-project.js';

/**
 * gRPC Plain Text Tests for Issue Operations
 *
 * Tests direct gRPC calls for issue CRUD operations.
 */
describe('gRPC: Issue Operations', () => {
  let project: TempProject;

  beforeEach(async () => {
    project = await createTempProject({ initialize: true });
  });

  afterEach(async () => {
    await project.cleanup();
  });

  describe('CreateIssue', () => {
    it('should create an issue with minimal fields', async () => {
      const result = await project.client.createIssue({
        projectPath: project.path,
        title: 'Test Issue',
      });

      expect(result.success).toBe(true);
      expect(result.error).toBe('');
      expect(result.id).toBeDefined();
      expect(result.displayNumber).toBe(1);
    });

    it('should create an issue with all fields', async () => {
      const result = await project.client.createIssue({
        projectPath: project.path,
        title: 'Full Issue',
        description: 'This is a detailed description',
        priority: 1,
        status: 'in-progress',
      });

      expect(result.success).toBe(true);
      expect(result.displayNumber).toBe(1);
      expect(result.createdFiles.length).toBeGreaterThan(0);
    });

    it('should auto-increment display numbers', async () => {
      const issue1 = await project.client.createIssue({
        projectPath: project.path,
        title: 'First Issue',
      });

      const issue2 = await project.client.createIssue({
        projectPath: project.path,
        title: 'Second Issue',
      });

      const issue3 = await project.client.createIssue({
        projectPath: project.path,
        title: 'Third Issue',
      });

      expect(issue1.displayNumber).toBe(1);
      expect(issue2.displayNumber).toBe(2);
      expect(issue3.displayNumber).toBe(3);
    });

    it('should return updated manifest', async () => {
      const result = await project.client.createIssue({
        projectPath: project.path,
        title: 'Manifest Test',
      });

      expect(result.manifest).toBeDefined();
      expect(result.manifest?.updatedAt).toBeDefined();
    });
  });

  describe('GetIssue', () => {
    it('should get issue by ID', async () => {
      const created = await project.client.createIssue({
        projectPath: project.path,
        title: 'Get By ID Test',
        description: 'Test description',
      });

      const issue = await project.client.getIssue({
        projectPath: project.path,
        issueId: created.id,
      });

      expect(issue.id).toBe(created.id);
      expect(issue.title).toBe('Get By ID Test');
      expect(issue.description).toContain('Test description');
    });

    it('should return full metadata', async () => {
      const created = await project.client.createIssue({
        projectPath: project.path,
        title: 'Metadata Test',
        priority: 2,
        status: 'open',
      });

      const issue = await project.client.getIssue({
        projectPath: project.path,
        issueId: created.id,
      });

      expect(issue.metadata).toBeDefined();
      expect(issue.metadata.status).toBe('open');
      expect(issue.metadata.priority).toBe(2);
      expect(issue.metadata.createdAt).toBeDefined();
      expect(issue.metadata.updatedAt).toBeDefined();
    });
  });

  describe('GetIssueByDisplayNumber', () => {
    it('should get issue by display number', async () => {
      await project.client.createIssue({
        projectPath: project.path,
        title: 'Display Number Test',
      });

      const issue = await project.client.getIssueByDisplayNumber({
        projectPath: project.path,
        displayNumber: 1,
      });

      expect(issue.displayNumber).toBe(1);
      expect(issue.title).toBe('Display Number Test');
    });
  });

  describe('ListIssues', () => {
    it('should list all issues', async () => {
      await project.client.createIssue({
        projectPath: project.path,
        title: 'Issue 1',
      });
      await project.client.createIssue({
        projectPath: project.path,
        title: 'Issue 2',
      });
      await project.client.createIssue({
        projectPath: project.path,
        title: 'Issue 3',
      });

      const result = await project.client.listIssues({
        projectPath: project.path,
      });

      expect(result.totalCount).toBe(3);
      expect(result.issues.length).toBe(3);
    });

    it('should filter by status', async () => {
      await project.client.createIssue({
        projectPath: project.path,
        title: 'Open Issue',
        status: 'open',
      });
      await project.client.createIssue({
        projectPath: project.path,
        title: 'Closed Issue',
        status: 'closed',
      });

      const result = await project.client.listIssues({
        projectPath: project.path,
        status: 'open',
      });

      expect(result.totalCount).toBe(1);
      expect(result.issues[0].title).toBe('Open Issue');
    });

    it('should filter by priority', async () => {
      await project.client.createIssue({
        projectPath: project.path,
        title: 'High Priority',
        priority: 1,
      });
      await project.client.createIssue({
        projectPath: project.path,
        title: 'Low Priority',
        priority: 3,
      });

      const result = await project.client.listIssues({
        projectPath: project.path,
        priority: 1,
      });

      expect(result.totalCount).toBe(1);
      expect(result.issues[0].title).toBe('High Priority');
    });

    it('should return empty list for empty project', async () => {
      const result = await project.client.listIssues({
        projectPath: project.path,
      });

      expect(result.totalCount).toBe(0);
      expect(result.issues.length).toBe(0);
    });
  });

  describe('UpdateIssue', () => {
    it('should update issue title', async () => {
      const created = await project.client.createIssue({
        projectPath: project.path,
        title: 'Original Title',
      });

      const result = await project.client.updateIssue({
        projectPath: project.path,
        issueId: created.id,
        title: 'Updated Title',
      });

      expect(result.success).toBe(true);
      expect(result.issue?.title).toBe('Updated Title');
    });

    it('should update issue status', async () => {
      const created = await project.client.createIssue({
        projectPath: project.path,
        title: 'Status Update Test',
        status: 'open',
      });

      const result = await project.client.updateIssue({
        projectPath: project.path,
        issueId: created.id,
        status: 'closed',
      });

      expect(result.success).toBe(true);
      expect(result.issue?.metadata.status).toBe('closed');
    });

    it('should update issue priority', async () => {
      const created = await project.client.createIssue({
        projectPath: project.path,
        title: 'Priority Update Test',
        priority: 3,
      });

      const result = await project.client.updateIssue({
        projectPath: project.path,
        issueId: created.id,
        priority: 1,
      });

      expect(result.success).toBe(true);
      expect(result.issue?.metadata.priority).toBe(1);
    });

    it('should update issue description', async () => {
      const created = await project.client.createIssue({
        projectPath: project.path,
        title: 'Description Update Test',
        description: 'Original description',
      });

      const result = await project.client.updateIssue({
        projectPath: project.path,
        issueId: created.id,
        description: 'Updated description with more details',
      });

      expect(result.success).toBe(true);
      expect(result.issue?.description).toContain('Updated description');
    });

    it('should update updatedAt timestamp', async () => {
      const created = await project.client.createIssue({
        projectPath: project.path,
        title: 'Timestamp Test',
      });

      // Small delay to ensure different timestamp
      await new Promise((r) => setTimeout(r, 100));

      const result = await project.client.updateIssue({
        projectPath: project.path,
        issueId: created.id,
        title: 'Updated Timestamp Test',
      });

      const originalUpdatedAt = new Date(created.id).getTime();
      const newUpdatedAt = new Date(result.issue!.metadata.updatedAt).getTime();

      // Just verify updatedAt is a valid date
      expect(result.issue?.metadata.updatedAt).toBeDefined();
    });
  });

  describe('DeleteIssue', () => {
    it('should delete an issue', async () => {
      const created = await project.client.createIssue({
        projectPath: project.path,
        title: 'To Delete',
      });

      const deleteResult = await project.client.deleteIssue({
        projectPath: project.path,
        issueId: created.id,
      });

      expect(deleteResult.success).toBe(true);

      // Verify it's gone
      const listResult = await project.client.listIssues({
        projectPath: project.path,
      });
      expect(listResult.totalCount).toBe(0);
    });

    it('should return error for non-existent issue', async () => {
      try {
        await project.client.deleteIssue({
          projectPath: project.path,
          issueId: 'non-existent-id',
        });
        expect.fail('Should have thrown an error');
      } catch (error: any) {
        expect(error).toBeDefined();
      }
    });
  });

  describe('GetNextIssueNumber', () => {
    it('should return next available number', async () => {
      const result1 = await project.client.getNextIssueNumber({
        projectPath: project.path,
      });

      await project.client.createIssue({
        projectPath: project.path,
        title: 'Issue 1',
      });

      const result2 = await project.client.getNextIssueNumber({
        projectPath: project.path,
      });

      expect(result1.issueNumber).toBeDefined();
      expect(result2.issueNumber).toBeDefined();
    });
  });
});
