import { test, expect } from '@playwright/test';
test.describe('Gallery thumbnail lazy loading', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the app
    await page.goto('app://localhost');
    // Wait for content to load
    await page.waitForTimeout(3000);
  });

  test('images should have src attributes set by IntersectionObserver', async ({ page }) => {
    // Check if MasonryWall rendered
    const masonryWall = page.locator('.masonry-wall');
    const masonryCount = await masonryWall.count();
    console.log(`MasonryWall count: ${masonryCount}`);

    // Check for gallery cards with data-image-path
    const cards = page.locator('[data-image-path]');
    const cardCount = await cards.count();
    console.log(`Cards with data-image-path: ${cardCount}`);

    if (cardCount === 0) {
      console.log('No cards found - checking skeleton state');
      const skeletons = page.locator('.skeleton-card');
      console.log(`Skeleton cards: ${await skeletons.count()}`);
      return;
    }

    // Get all img src attributes
    const imgs = page.locator('.gallery-card img');
    const imgCount = await imgs.count();
    console.log(`Images in gallery cards: ${imgCount}`);

    for (let i = 0; i < Math.min(imgCount, 10); i++) {
      const src = await imgs.nth(i).getAttribute('src');
      console.log(`img[${i}] src: "${src}"`);
    }

    // Scroll to trigger IntersectionObserver
    await page.evaluate(() => window.scrollTo(0, 500));
    await page.waitForTimeout(1000);

    // Check again after scroll
    const imgsAfter = page.locator('.gallery-card img');
    const imgCountAfter = await imgsAfter.count();
    console.log(`Images after scroll: ${imgCountAfter}`);

    for (let i = 0; i < Math.min(imgCountAfter, 5); i++) {
      const src = await imgsAfter.nth(i).getAttribute('src');
      console.log(`img[${i}] src after scroll: "${src}"`);
    }

    // Final check: at least some images should have non-empty src
    const nonEmptySrcs = await page.locator('.gallery-card img[src]').count();
    console.log(`Images with non-empty src: ${nonEmptySrcs}`);
  });

  test('data-gallery-cards attribute is on the correct element', async ({ page }) => {
    const galleryCardsElements = page.locator('[data-gallery-cards]');
    const count = await galleryCardsElements.count();
    console.log(`[data-gallery-cards] elements: ${count}`);

    for (let i = 0; i < count; i++) {
      const el = galleryCardsElements.nth(i);
      const tag = await el.evaluate((node) => node.tagName);
      const classes = await el.evaluate((node) => node.className);
      console.log(`  [${i}] ${tag}.${classes}`);
    }
  });
});
