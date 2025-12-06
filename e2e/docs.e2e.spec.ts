import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { createTempProject, type TempProject } from './fixtures/temp-project.js';

/**
 * gRPC Plain Text Tests for Documentation Operations
 *
 * Tests direct gRPC calls for doc CRUD operations.
 */
describe('gRPC: Doc Operations', () => {
  let project: TempProject;

  beforeEach(async () => {
    project = await createTempProject({ initialize: true });
  });

  afterEach(async () => {
    await project.cleanup();
  });

  describe('CreateDoc', () => {
    it('should create a doc with title only', async () => {
      const result = await project.client.createDoc({
        projectPath: project.path,
        title: 'Getting Started',
      });

      expect(result.success).toBe(true);
      expect(result.error).toBe('');
      expect(result.slug).toBe('getting-started');
    });

    it('should create a doc with content', async () => {
      const result = await project.client.createDoc({
        projectPath: project.path,
        title: 'API Reference',
        content: '# API\n\nThis is the API documentation.',
      });

      expect(result.success).toBe(true);
      expect(result.slug).toBe('api-reference');
    });

    it('should create a doc with custom slug', async () => {
      const result = await project.client.createDoc({
        projectPath: project.path,
        title: 'My Guide',
        slug: 'custom-slug',
      });

      expect(result.success).toBe(true);
      expect(result.slug).toBe('custom-slug');
    });

    it('should return created file path', async () => {
      const result = await project.client.createDoc({
        projectPath: project.path,
        title: 'File Path Test',
      });

      expect(result.createdFile).toBeDefined();
      expect(result.createdFile).toContain('.centy/docs/');
      expect(result.createdFile).toContain('.md');
    });
  });

  describe('GetDoc', () => {
    it('should get doc by slug', async () => {
      await project.client.createDoc({
        projectPath: project.path,
        title: 'Test Doc',
        content: '# Test\n\nThis is test content.',
      });

      const doc = await project.client.getDoc({
        projectPath: project.path,
        slug: 'test-doc',
      });

      expect(doc.slug).toBe('test-doc');
      expect(doc.title).toBe('Test Doc');
      expect(doc.content).toContain('test content');
    });

    it('should return doc metadata', async () => {
      await project.client.createDoc({
        projectPath: project.path,
        title: 'Metadata Test',
      });

      const doc = await project.client.getDoc({
        projectPath: project.path,
        slug: 'metadata-test',
      });

      expect(doc.metadata).toBeDefined();
      expect(doc.metadata.createdAt).toBeDefined();
      expect(doc.metadata.updatedAt).toBeDefined();
    });
  });

  describe('ListDocs', () => {
    it('should list all docs', async () => {
      await project.client.createDoc({
        projectPath: project.path,
        title: 'Doc One',
      });
      await project.client.createDoc({
        projectPath: project.path,
        title: 'Doc Two',
      });
      await project.client.createDoc({
        projectPath: project.path,
        title: 'Doc Three',
      });

      const result = await project.client.listDocs({
        projectPath: project.path,
      });

      expect(result.totalCount).toBe(3);
      expect(result.docs.length).toBe(3);
    });

    it('should return empty list for empty project', async () => {
      const result = await project.client.listDocs({
        projectPath: project.path,
      });

      expect(result.totalCount).toBe(0);
      expect(result.docs.length).toBe(0);
    });

    it('should return docs with all fields populated', async () => {
      await project.client.createDoc({
        projectPath: project.path,
        title: 'Full Doc',
        content: '# Content here',
      });

      const result = await project.client.listDocs({
        projectPath: project.path,
      });

      const doc = result.docs[0];
      expect(doc.slug).toBeDefined();
      expect(doc.title).toBeDefined();
      expect(doc.metadata).toBeDefined();
    });
  });

  describe('UpdateDoc', () => {
    it('should update doc title', async () => {
      await project.client.createDoc({
        projectPath: project.path,
        title: 'Original Title',
      });

      const result = await project.client.updateDoc({
        projectPath: project.path,
        slug: 'original-title',
        title: 'Updated Title',
      });

      expect(result.success).toBe(true);
      expect(result.doc?.title).toBe('Updated Title');
    });

    it('should update doc content', async () => {
      await project.client.createDoc({
        projectPath: project.path,
        title: 'Content Test',
        content: 'Original content',
      });

      const result = await project.client.updateDoc({
        projectPath: project.path,
        slug: 'content-test',
        content: 'Updated content with more information',
      });

      expect(result.success).toBe(true);
      expect(result.doc?.content).toContain('Updated content');
    });

    it('should rename doc (change slug)', async () => {
      await project.client.createDoc({
        projectPath: project.path,
        title: 'Old Slug',
      });

      const result = await project.client.updateDoc({
        projectPath: project.path,
        slug: 'old-slug',
        newSlug: 'new-slug',
      });

      expect(result.success).toBe(true);
      expect(result.doc?.slug).toBe('new-slug');

      // Verify old slug no longer exists
      try {
        await project.client.getDoc({
          projectPath: project.path,
          slug: 'old-slug',
        });
        expect.fail('Should have thrown an error');
      } catch {
        // Expected
      }
    });
  });

  describe('DeleteDoc', () => {
    it('should delete a doc', async () => {
      await project.client.createDoc({
        projectPath: project.path,
        title: 'To Delete',
      });

      const deleteResult = await project.client.deleteDoc({
        projectPath: project.path,
        slug: 'to-delete',
      });

      expect(deleteResult.success).toBe(true);

      // Verify it's gone
      const listResult = await project.client.listDocs({
        projectPath: project.path,
      });
      expect(listResult.totalCount).toBe(0);
    });

    it('should return error for non-existent doc', async () => {
      try {
        await project.client.deleteDoc({
          projectPath: project.path,
          slug: 'non-existent',
        });
        expect.fail('Should have thrown an error');
      } catch (error: any) {
        expect(error).toBeDefined();
      }
    });
  });
});
