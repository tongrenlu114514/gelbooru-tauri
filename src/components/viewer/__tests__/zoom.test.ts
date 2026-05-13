import { describe, it, expect, vi, beforeEach } from 'vitest';
import { createViewer } from './util';

describe('ImageViewer - Zoom Behavior', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // -------------------------------------------------------------------------
  // Test 1: zoom level starts at 1 (100%)
  // -------------------------------------------------------------------------
  it('starts with zoom level at 1 (100%)', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    // Find the zoom level display
    const zoomText = wrapper.find('.zoom-level');
    expect(zoomText.exists()).toBe(true);
    expect(zoomText.text()).toBe('100%');

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 2: zoom buttons work correctly
  // -------------------------------------------------------------------------
  it('zoom in button increases zoom level by 0.2', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    const zoomInBtn = wrapper.find('.zoom-btn:nth-of-type(2)'); // + button
    await zoomInBtn.trigger('click');

    const zoomText = wrapper.find('.zoom-level');
    expect(zoomText.text()).toBe('120%');

    wrapper.unmount();
  });

  it('zoom out button decreases zoom level by 0.2', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    const zoomOutBtn = wrapper.find('.zoom-btn:first-of-type'); // - button
    await zoomOutBtn.trigger('click');

    const zoomText = wrapper.find('.zoom-level');
    expect(zoomText.text()).toBe('80%');

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 3: zoom level clamped at max (500%)
  // -------------------------------------------------------------------------
  it('clamps zoom level at 5 (500% max)', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    // Click zoom in multiple times
    const zoomInBtn = wrapper.find('.zoom-btn:nth-of-type(2)');
    for (let i = 0; i < 25; i++) {
      await zoomInBtn.trigger('click');
    }

    const zoomText = wrapper.find('.zoom-level');
    expect(zoomText.text()).toBe('500%');

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 4: zoom level clamped at min (50%)
  // -------------------------------------------------------------------------
  it('clamps zoom level at 0.5 (50% min)', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    // Click zoom out multiple times
    const zoomOutBtn = wrapper.find('.zoom-btn:first-of-type');
    for (let i = 0; i < 25; i++) {
      await zoomOutBtn.trigger('click');
    }

    const zoomText = wrapper.find('.zoom-level');
    expect(zoomText.text()).toBe('50%');

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 5: reset zoom button resets to 100%
  // -------------------------------------------------------------------------
  it('reset zoom button sets zoom back to 100%', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    // First zoom in
    const zoomInBtn = wrapper.find('.zoom-btn:nth-of-type(2)');
    await zoomInBtn.trigger('click');
    expect(wrapper.find('.zoom-level').text()).toBe('120%');

    // Then reset
    const resetBtn = wrapper.find('.reset-zoom-btn');
    await resetBtn.trigger('click');

    expect(wrapper.find('.zoom-level').text()).toBe('100%');

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 6: zoom resets when navigating to different image
  // -------------------------------------------------------------------------
  it('resets zoom when navigating to different image', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    // Zoom in
    const zoomInBtn = wrapper.find('.zoom-btn:nth-of-type(2)');
    await zoomInBtn.trigger('click');
    expect(wrapper.find('.zoom-level').text()).toBe('120%');

    // Navigate to next image
    const nextBtn = wrapper.find('.nav-next');
    await nextBtn.trigger('click');

    // Zoom should reset to 100%
    expect(wrapper.find('.zoom-level').text()).toBe('100%');

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 7: keyboard zoom shortcuts
  // -------------------------------------------------------------------------
  it('keyboard "+" key triggers zoom in', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    // Simulate keyboard event
    const event = new KeyboardEvent('keydown', { key: '+', bubbles: true });
    window.dispatchEvent(event);

    await wrapper.vm.$nextTick();

    expect(wrapper.find('.zoom-level').text()).toBe('120%');

    wrapper.unmount();
  });

  it('keyboard "-" key triggers zoom out', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    const event = new KeyboardEvent('keydown', { key: '-', bubbles: true });
    window.dispatchEvent(event);

    await wrapper.vm.$nextTick();

    expect(wrapper.find('.zoom-level').text()).toBe('80%');

    wrapper.unmount();
  });

  it('keyboard "0" key resets zoom', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    // First zoom in
    const zoomInBtn = wrapper.find('.zoom-btn:nth-of-type(2)');
    await zoomInBtn.trigger('click');
    expect(wrapper.find('.zoom-level').text()).toBe('120%');

    // Then press 0
    const event = new KeyboardEvent('keydown', { key: '0', bubbles: true });
    window.dispatchEvent(event);

    await wrapper.vm.$nextTick();

    expect(wrapper.find('.zoom-level').text()).toBe('100%');

    wrapper.unmount();
  });
});
