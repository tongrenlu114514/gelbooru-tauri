import { mount } from '@vue/test-utils';
import { nextTick } from 'vue';

// Note: @tauri-apps/api/core and naive-ui are already mocked in src/tests/setup.ts
// via module-level spies. No need to re-mock here.

import type { Component } from 'vue';

export interface TestImageInfo {
  path: string;
  name: string;
}

const defaultImages: TestImageInfo[] = [
  { path: '/test/a.jpg', name: 'a.jpg' },
  { path: '/test/b.jpg', name: 'b.jpg' },
  { path: '/test/c.jpg', name: 'c.jpg' },
];

export interface ViewerMountOptions {
  props?: {
    images?: TestImageInfo[];
    visible?: boolean;
    initialIndex?: number;
  };
  global?: {
    stubs?: Record<string, boolean | Component>;
  };
}

export async function createViewer(options: ViewerMountOptions = {}) {
  const { props = {} } = options;

  // Dynamic import to avoid hoisting issues
  const { default: ImageViewer } = await import('../ImageViewer.vue');

  const wrapper = mount(ImageViewer as unknown as Component, {
    props: {
      images: defaultImages,
      visible: true,
      initialIndex: 0,
      ...props,
    },
    global: {
      stubs: {
        'n-icon': true,
        'n-text': true,
        Teleport: true,
      },
      ...options.global,
    },
  });

  await nextTick();

  return wrapper;
}

export { defaultImages };