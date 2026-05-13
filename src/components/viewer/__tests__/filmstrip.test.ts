import { mount } from '@vue/test-utils';
import { describe, it, expect } from 'vitest';
import Filmstrip from '../Filmstrip.vue';

vi.mock('@tauri-apps/api/core', () => ({
  convertFileSrc: (path: string) => `asset://localhost/${path}`,
}));

function makeImage(index: number) {
  return { path: `/images/img${index}.jpg`, name: `image ${index}` };
}

function makeGallery(count: number) {
  return Array.from({ length: count }, (_, i) => makeImage(i));
}

describe('Filmstrip', () => {
  it('renders 9 thumbnails when images length >= 9', () => {
    const images = makeGallery(20);
    const wrapper = mount(Filmstrip, {
      props: { images, currentIndex: 5 },
    });
    const thumbs = wrapper.findAll('.filmstrip-thumb');
    expect(thumbs).toHaveLength(9);
  });

  it('renders centered range around currentIndex', () => {
    const images = makeGallery(20);
    const wrapper = mount(Filmstrip, {
      props: { images, currentIndex: 6 },
    });
    const thumbs = wrapper.findAll('.filmstrip-thumb');
    expect(thumbs.length).toBe(9);
  });

  it('shows correct range centered on currentIndex=5', () => {
    const images = makeGallery(20);
    const wrapper = mount(Filmstrip, {
      props: { images, currentIndex: 5 },
    });
    // start=max(0,5-4)=1, end=min(19,5+4)=9 → 9 items at indices 1-9
    const thumbs = wrapper.findAll('.filmstrip-thumb');
    expect(thumbs).toHaveLength(9);
  });

  it('first thumbnail has active class when at currentIndex=5', () => {
    const images = makeGallery(20);
    const wrapper = mount(Filmstrip, {
      props: { images, currentIndex: 5 },
    });
    const active = wrapper.findAll('.filmstrip-thumb.active');
    expect(active).toHaveLength(1);
  });

  it('clicking thumbnail emits select with correct index', async () => {
    const images = makeGallery(20);
    const wrapper = mount(Filmstrip, {
      props: { images, currentIndex: 5 },
    });
    const thumbs = wrapper.findAll('.filmstrip-thumb');
    // currentIndex=5 → start=1, end=9 → first visible is global index 1
    await thumbs[0].trigger('click');
    expect(wrapper.emitted('select')).toBeTruthy();
    const emitted = wrapper.emitted<[number]>('select')!;
    expect(emitted[0][0]).toBe(1); // first visible is at global index 1
  });

  it('clicking non-active thumbnail navigates to correct index', async () => {
    const images = makeGallery(20);
    const wrapper = mount(Filmstrip, {
      props: { images, currentIndex: 5 },
    });
    const thumbs = wrapper.findAll('.filmstrip-thumb');
    // currentIndex=5 → start=1, end=9 → last visible is global index 9
    await thumbs[8].trigger('click');
    const emitted = wrapper.emitted<[number]>('select')!;
    expect(emitted[0][0]).toBe(9); // last visible is at global index 9
  });

  it('visibleThumbnails updates when currentIndex changes', async () => {
    const images = makeGallery(20);
    const wrapper = mount(Filmstrip, {
      props: { images, currentIndex: 3 },
    });
    await wrapper.setProps({ currentIndex: 10 });
    const thumbs = wrapper.findAll('.filmstrip-thumb');
    expect(thumbs).toHaveLength(9);
  });

  it('handles edge case when currentIndex = 0', () => {
    const images = makeGallery(20);
    const wrapper = mount(Filmstrip, {
      props: { images, currentIndex: 0 },
    });
    // start=max(0,0-4)=0, end=min(19,0+4)=4 → 5 items (0,1,2,3,4)
    const thumbs = wrapper.findAll('.filmstrip-thumb');
    expect(thumbs).toHaveLength(5);
  });

  it('handles edge case when currentIndex = images.length - 1', () => {
    const images = makeGallery(20);
    const wrapper = mount(Filmstrip, {
      props: { images, currentIndex: 19 },
    });
    // start=max(0,19-4)=15, end=min(19,19+4)=19 → 5 items (15-19)
    const thumbs = wrapper.findAll('.filmstrip-thumb');
    expect(thumbs).toHaveLength(5);
  });

  it('handles case when images.length < 9', () => {
    const images = makeGallery(5);
    const wrapper = mount(Filmstrip, {
      props: { images, currentIndex: 2 },
    });
    // All 5 thumbnails shown
    const thumbs = wrapper.findAll('.filmstrip-thumb');
    expect(thumbs).toHaveLength(5);
  });

  it('shows only available thumbnails when near start', () => {
    const images = makeGallery(10);
    const wrapper = mount(Filmstrip, {
      props: { images, currentIndex: 1 },
    });
    // start=max(0,1-4)=0, end=min(9,1+4)=5 → 6 items (0-5)
    const thumbs = wrapper.findAll('.filmstrip-thumb');
    expect(thumbs).toHaveLength(6);
  });

  it('shows only available thumbnails when near end', () => {
    const images = makeGallery(10);
    const wrapper = mount(Filmstrip, {
      props: { images, currentIndex: 8 },
    });
    // start=max(0,8-4)=4, end=min(9,8+4)=9 → 6 items (4-9)
    const thumbs = wrapper.findAll('.filmstrip-thumb');
    expect(thumbs).toHaveLength(6);
  });
});