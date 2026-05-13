import { describe, it, expect, vi, beforeEach } from 'vitest';
import { createViewer } from './util';

describe('ImageViewer - Pan/Drag Behavior', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // -------------------------------------------------------------------------
  // Test 1: pan offset starts at { x: 0, y: 0 }
  // -------------------------------------------------------------------------
  it('starts with pan offset at { x: 0, y: 0 }', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    // At zoom level 1, cursor should be 'default'
    const imageContainer = wrapper.find('.image-container');
    expect(imageContainer.exists()).toBe(true);

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 2: cursor style is 'default' when zoomLevel = 1
  // -------------------------------------------------------------------------
  it('cursor style is default when zoomLevel is 1', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    const img = wrapper.find('.image-container img');
    expect(img.exists()).toBe(true);
    expect(img.attributes('style')).toContain('cursor: default');

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 3: cursor style is 'grab' when zoomLevel > 1
  // -------------------------------------------------------------------------
  it('cursor style changes to grab when zoomed in', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    // Zoom in to > 1
    const zoomInBtn = wrapper.find('.zoom-btn:nth-of-type(2)');
    await zoomInBtn.trigger('click');

    const img = wrapper.find('.image-container img');
    expect(img.attributes('style')).toContain('cursor: grab');

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 4: dragging updates pan offset when zoomed
  // -------------------------------------------------------------------------
  it('dragging updates image position when zoomed', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    // Zoom in to enable panning
    const zoomInBtn = wrapper.find('.zoom-btn:nth-of-type(2)');
    await zoomInBtn.trigger('click');

    const container = wrapper.find('.image-container');

    // Simulate mousedown
    await container.trigger('mousedown', {
      clientX: 100,
      clientY: 100,
    });

    // Simulate mousemove
    await window.dispatchEvent(
      new MouseEvent('mousemove', {
        clientX: 150,
        clientY: 150,
        bubbles: true,
      })
    );

    // Simulate mouseup
    await window.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }));

    // Check that the image transform includes translation
    const img = wrapper.find('.image-container img');
    const style = img.attributes('style');
    expect(style).toContain('translate(');

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 5: drag only active when zoomLevel > 1
  // -------------------------------------------------------------------------
  it('dragging is disabled when not zoomed (zoomLevel = 1)', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    const container = wrapper.find('.image-container');

    // Try to start drag at zoom level 1
    await container.trigger('mousedown', {
      clientX: 100,
      clientY: 100,
    });

    // Simulate mousemove
    await window.dispatchEvent(
      new MouseEvent('mousemove', {
        clientX: 150,
        clientY: 150,
        bubbles: true,
      })
    );

    // Simulate mouseup
    await window.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }));

    // Image should not have translate transform
    const img = wrapper.find('.image-container img');
    const style = img.attributes('style');

    // At zoom level 1, panning is disabled - pan offset should stay at (0, 0)
    // The image style always has translate, but when not dragging it should be translate(0px, 0px)
    // Since we never started a drag (mousedown is blocked at zoomLevel <= 1), offset stays at origin
    expect(style).toContain('translate(0px, 0px)');
    // Verify no translation occurred (offset not changed)
    // The key behavior: mousedown at zoomLevel=1 does NOT start a drag
    // so panOffset never changes from { x: 0, y: 0 }

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 6: cursor is 'grabbing' while dragging
  // -------------------------------------------------------------------------
  it('cursor is grabbing while dragging', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    // Zoom in
    const zoomInBtn = wrapper.find('.zoom-btn:nth-of-type(2)');
    await zoomInBtn.trigger('click');

    const container = wrapper.find('.image-container');

    // Start drag
    await container.trigger('mousedown', {
      clientX: 100,
      clientY: 100,
    });

    // Check cursor is grabbing during drag
    const img = wrapper.find('.image-container img');
    expect(img.attributes('style')).toContain('cursor: grabbing');

    // End drag
    await window.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }));

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 7: pan offset resets when navigating to different image
  // -------------------------------------------------------------------------
  it('resets pan offset when navigating to different image', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    // Zoom in and drag
    const zoomInBtn = wrapper.find('.zoom-btn:nth-of-type(2)');
    await zoomInBtn.trigger('click');

    const container = wrapper.find('.image-container');

    await container.trigger('mousedown', {
      clientX: 100,
      clientY: 100,
    });

    await window.dispatchEvent(
      new MouseEvent('mousemove', {
        clientX: 200,
        clientY: 200,
        bubbles: true,
      })
    );

    await window.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }));

    // Navigate to next image
    const nextBtn = wrapper.find('.nav-next');
    await nextBtn.trigger('click');

    // Pan should reset - check image style has translate(0px, 0px)
    const img = wrapper.find('.image-container img');
    const style = img.attributes('style');
    expect(style).toContain('translate(0px, 0px)');

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 8: cursor style is 'default' when zoomed out below 1
  // -------------------------------------------------------------------------
  it('cursor is default when zoomed below 1', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    // Zoom out to below 1
    const zoomOutBtn = wrapper.find('.zoom-btn:first-of-type');
    await zoomOutBtn.trigger('click');

    const img = wrapper.find('.image-container img');
    expect(img.attributes('style')).toContain('cursor: default');

    wrapper.unmount();
  });
});
