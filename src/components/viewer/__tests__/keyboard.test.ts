import { describe, it, expect, vi, beforeEach } from 'vitest';
import { createViewer } from './util';

describe('ImageViewer - Keyboard Behavior', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // -------------------------------------------------------------------------
  // Test 1: ArrowLeft navigates to previous image
  // -------------------------------------------------------------------------
  it('ArrowLeft navigates to previous image', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 1 },
    });

    expect(wrapper.find('.image-counter').text()).toContain('2 /');

    const event = new KeyboardEvent('keydown', { key: 'ArrowLeft', bubbles: true });
    window.dispatchEvent(event);

    await wrapper.vm.$nextTick();

    expect(wrapper.find('.image-counter').text()).toContain('1 /');

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 2: ArrowRight navigates to next image
  // -------------------------------------------------------------------------
  it('ArrowRight navigates to next image', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    expect(wrapper.find('.image-counter').text()).toContain('1 /');

    const event = new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true });
    window.dispatchEvent(event);

    await wrapper.vm.$nextTick();

    expect(wrapper.find('.image-counter').text()).toContain('2 /');

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 3: ArrowLeft disabled at index 0
  // -------------------------------------------------------------------------
  it('ArrowLeft does nothing at index 0', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    const prevBtn = wrapper.find('.nav-prev');
    expect(prevBtn.attributes('disabled')).toBeDefined();

    const event = new KeyboardEvent('keydown', { key: 'ArrowLeft', bubbles: true });
    window.dispatchEvent(event);

    await wrapper.vm.$nextTick();

    // Counter should still show 1/
    expect(wrapper.find('.image-counter').text()).toContain('1 /');

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 4: ArrowRight disabled at last image
  // -------------------------------------------------------------------------
  it('ArrowRight does nothing at last image', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 2 },
    });

    const nextBtn = wrapper.find('.nav-next');
    expect(nextBtn.attributes('disabled')).toBeDefined();

    const event = new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true });
    window.dispatchEvent(event);

    await wrapper.vm.$nextTick();

    // Counter should still show 3/
    expect(wrapper.find('.image-counter').text()).toContain('3 /');

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 5: Escape emits update:visible false
  // -------------------------------------------------------------------------
  it('Escape emits update:visible false', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    const event = new KeyboardEvent('keydown', { key: 'Escape', bubbles: true });
    window.dispatchEvent(event);

    await wrapper.vm.$nextTick();

    expect(wrapper.emitted('update:visible')).toBeDefined();
    const updateEvents = wrapper.emitted('update:visible') as unknown[][];
    const lastEvent = updateEvents[updateEvents.length - 1];
    expect(lastEvent).toContain(false);

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 6: '+' key triggers zoom in
  // -------------------------------------------------------------------------
  it('plus key triggers zoom in', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    const event = new KeyboardEvent('keydown', { key: '+', bubbles: true });
    window.dispatchEvent(event);

    await wrapper.vm.$nextTick();

    expect(wrapper.find('.zoom-level').text()).toBe('120%');

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 7: '=' key triggers zoom in (same as '+')
  // -------------------------------------------------------------------------
  it('equals key triggers zoom in', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    const event = new KeyboardEvent('keydown', { key: '=', bubbles: true });
    window.dispatchEvent(event);

    await wrapper.vm.$nextTick();

    expect(wrapper.find('.zoom-level').text()).toBe('120%');

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 8: '-' key triggers zoom out
  // -------------------------------------------------------------------------
  it('minus key triggers zoom out', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    const event = new KeyboardEvent('keydown', { key: '-', bubbles: true });
    window.dispatchEvent(event);

    await wrapper.vm.$nextTick();

    expect(wrapper.find('.zoom-level').text()).toBe('80%');

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 9: '0' key resets zoom
  // -------------------------------------------------------------------------
  it('zero key resets zoom', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 0 },
    });

    // Zoom in first
    const zoomInBtn = wrapper.find('.zoom-btn:nth-of-type(2)');
    await zoomInBtn.trigger('click');
    expect(wrapper.find('.zoom-level').text()).toBe('120%');

    // Press 0
    const event = new KeyboardEvent('keydown', { key: '0', bubbles: true });
    window.dispatchEvent(event);

    await wrapper.vm.$nextTick();

    expect(wrapper.find('.zoom-level').text()).toBe('100%');

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 10: keyboard events ignored when visible=false
  // -------------------------------------------------------------------------
  it('keyboard events are ignored when visible is false', async () => {
    const wrapper = await createViewer({
      props: { visible: false, initialIndex: 0 },
    });

    // When visible=false, the viewer is not rendered (v-if="visible")
    // The overlay should not be present
    const overlay = wrapper.find('.viewer-overlay');
    expect(overlay.exists()).toBe(false);

    // Try to navigate - should have no effect (viewer not rendered)
    const event = new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true });
    window.dispatchEvent(event);

    await wrapper.vm.$nextTick();

    // Viewer is still not rendered - no counter exists
    expect(wrapper.find('.image-counter').exists()).toBe(false);

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 11: Delete key emits delete event
  // -------------------------------------------------------------------------
  it('delete button emits delete event with current index', async () => {
    const wrapper = await createViewer({
      props: { visible: true, initialIndex: 1 },
    });

    const deleteBtn = wrapper.find('.delete-btn');
    await deleteBtn.trigger('click');

    expect(wrapper.emitted('delete')).toBeDefined();
    const deleteEvents = wrapper.emitted('delete') as unknown[][];
    expect(deleteEvents[0]).toContain(1);

    wrapper.unmount();
  });
});