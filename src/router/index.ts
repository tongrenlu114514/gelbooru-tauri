import { createRouter, createWebHistory } from 'vue-router';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: () => import('@/views/Home.vue'),
    },
    {
      path: '/downloads',
      name: 'downloads',
      component: () => import('@/views/Downloads.vue'),
    },
    {
      path: '/gallery',
      name: 'gallery',
      component: () => import('@/views/Gallery.vue'),
    },
    {
      path: '/favorite-tags',
      name: 'favorite-tags',
      component: () => import('@/views/FavoriteTags.vue'),
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/Settings.vue'),
    },
  ],
});

export default router;
