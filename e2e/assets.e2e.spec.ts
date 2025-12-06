import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { createTempProject, type TempProject, testData } from './fixtures/temp-project.js';

/**
 * gRPC Plain Text Tests for Asset Operations
 *
 * Tests direct gRPC calls for asset management.
 */
describe('gRPC: Asset Operations', () => {
  let project: TempProject;
  let issueId: string;

  beforeEach(async () => {
    project = await createTempProject({ initialize: true });

    // Create an issue to attach assets to
    const result = await project.client.createIssue({
      projectPath: project.path,
      title: 'Asset Test Issue',
    });
    issueId = result.id;
  });

  afterEach(async () => {
    await project.cleanup();
  });

  describe('AddAsset', () => {
    it('should add a PNG asset to an issue', async () => {
      const pngData = testData.createTestPng();

      const result = await project.client.addAsset({
        projectPath: project.path,
        issueId,
        filename: 'test-image.png',
        data: pngData,
      });

      expect(result.success).toBe(true);
      expect(result.error).toBe('');
      expect(result.asset).toBeDefined();
      expect(result.asset?.filename).toBe('test-image.png');
      expect(result.asset?.mimeType).toContain('image');
    });

    it('should add a JPEG asset', async () => {
      const jpegData = testData.createTestJpeg();

      const result = await project.client.addAsset({
        projectPath: project.path,
        issueId,
        filename: 'photo.jpg',
        data: jpegData,
      });

      expect(result.success).toBe(true);
      expect(result.asset?.filename).toBe('photo.jpg');
    });

    it('should reject unsupported file types', async () => {
      const textData = testData.createTestText('Hello, World!');

      const result = await project.client.addAsset({
        projectPath: project.path,
        issueId,
        filename: 'readme.txt',
        data: textData,
      });

      // Daemon only supports image files, not text files
      expect(result.success).toBe(false);
      expect(result.error).toContain('Unsupported');
    });

    it('should add a shared asset', async () => {
      const pngData = testData.createTestPng();

      const result = await project.client.addAsset({
        projectPath: project.path,
        filename: 'shared-logo.png',
        data: pngData,
        isShared: true,
      });

      expect(result.success).toBe(true);
      expect(result.asset?.isShared).toBe(true);
    });

    it('should return asset hash', async () => {
      const pngData = testData.createTestPng();

      const result = await project.client.addAsset({
        projectPath: project.path,
        issueId,
        filename: 'hash-test.png',
        data: pngData,
      });

      expect(result.asset?.hash).toBeDefined();
      expect(result.asset?.hash.length).toBeGreaterThan(0);
    });

    it('should return asset size', async () => {
      const pngData = testData.createTestPng();

      const result = await project.client.addAsset({
        projectPath: project.path,
        issueId,
        filename: 'size-test.png',
        data: pngData,
      });

      expect(Number(result.asset?.size)).toBe(pngData.length);
    });
  });

  describe('ListAssets', () => {
    it('should list assets for an issue', async () => {
      // Add some assets
      await project.client.addAsset({
        projectPath: project.path,
        issueId,
        filename: 'file1.png',
        data: testData.createTestPng(),
      });
      await project.client.addAsset({
        projectPath: project.path,
        issueId,
        filename: 'file2.png',
        data: testData.createTestPng(),
      });

      const result = await project.client.listAssets({
        projectPath: project.path,
        issueId,
      });

      expect(result.totalCount).toBe(2);
      expect(result.assets.length).toBe(2);
    });

    it('should return empty list for issue without assets', async () => {
      const result = await project.client.listAssets({
        projectPath: project.path,
        issueId,
      });

      expect(result.totalCount).toBe(0);
      expect(result.assets.length).toBe(0);
    });

    it('should include shared assets when requested', async () => {
      // Add issue-specific asset
      await project.client.addAsset({
        projectPath: project.path,
        issueId,
        filename: 'issue-asset.png',
        data: testData.createTestPng(),
      });

      // Add shared asset
      await project.client.addAsset({
        projectPath: project.path,
        filename: 'shared-asset.png',
        data: testData.createTestPng(),
        isShared: true,
      });

      // List with include shared
      const result = await project.client.listAssets({
        projectPath: project.path,
        issueId,
        includeShared: true,
      });

      expect(result.totalCount).toBe(2);
    });
  });

  describe('ListSharedAssets', () => {
    it('should list only shared assets', async () => {
      // Add issue-specific asset
      await project.client.addAsset({
        projectPath: project.path,
        issueId,
        filename: 'issue-only.png',
        data: testData.createTestPng(),
      });

      // Add shared assets
      await project.client.addAsset({
        projectPath: project.path,
        filename: 'shared1.png',
        data: testData.createTestPng(),
        isShared: true,
      });
      await project.client.addAsset({
        projectPath: project.path,
        filename: 'shared2.png',
        data: testData.createTestPng(),
        isShared: true,
      });

      const result = await project.client.listSharedAssets({
        projectPath: project.path,
      });

      expect(result.totalCount).toBe(2);
      result.assets.forEach((asset) => {
        expect(asset.isShared).toBe(true);
      });
    });
  });

  describe('GetAsset', () => {
    it('should get asset data', async () => {
      const originalData = testData.createTestPng();

      await project.client.addAsset({
        projectPath: project.path,
        issueId,
        filename: 'get-test.png',
        data: originalData,
      });

      const result = await project.client.getAsset({
        projectPath: project.path,
        issueId,
        filename: 'get-test.png',
      });

      expect(result.success).toBe(true);
      expect(result.data).toBeDefined();
      expect(Buffer.from(result.data).length).toBe(originalData.length);
    });

    it('should get shared asset', async () => {
      const originalData = testData.createTestPng();

      await project.client.addAsset({
        projectPath: project.path,
        filename: 'shared-get-test.png',
        data: originalData,
        isShared: true,
      });

      const result = await project.client.getAsset({
        projectPath: project.path,
        filename: 'shared-get-test.png',
        isShared: true,
      });

      expect(result.success).toBe(true);
      expect(result.asset?.isShared).toBe(true);
    });
  });

  describe('DeleteAsset', () => {
    it('should delete an asset', async () => {
      await project.client.addAsset({
        projectPath: project.path,
        issueId,
        filename: 'to-delete.png',
        data: testData.createTestPng(),
      });

      const deleteResult = await project.client.deleteAsset({
        projectPath: project.path,
        issueId,
        filename: 'to-delete.png',
      });

      expect(deleteResult.success).toBe(true);
      expect(deleteResult.filename).toBe('to-delete.png');

      // Verify it's gone
      const listResult = await project.client.listAssets({
        projectPath: project.path,
        issueId,
      });
      expect(listResult.totalCount).toBe(0);
    });

    it('should delete a shared asset', async () => {
      await project.client.addAsset({
        projectPath: project.path,
        filename: 'shared-to-delete.png',
        data: testData.createTestPng(),
        isShared: true,
      });

      const deleteResult = await project.client.deleteAsset({
        projectPath: project.path,
        filename: 'shared-to-delete.png',
        isShared: true,
      });

      expect(deleteResult.success).toBe(true);
      expect(deleteResult.wasShared).toBe(true);
    });
  });
});
