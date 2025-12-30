import { createRouter, createWebHistory } from 'vue-router';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'game',
      component: () => import('../components/game.vue'),
    },
    {
      path: '/my-team',
      name: 'my-team',
      component: () => import('../components/Team.vue'),
    },
    {
      path: '/teams',
      name: 'all-teams',
      component: () => import('../components/AllTeams.vue'),
    },
    {
      path: '/teams/:teamId',
      name: 'team',
      component: () => import('../components/Team.vue'),
      props: true,
    },
    {
      path: '/races',
      name: 'races',
      component: () => import('../components/Races.vue'),
    },
    {
      path: '/races/:raceId',
      name: 'race',
      component: () => import('../components/Races.vue'),
      props: true,
    },
    {
      path: '/drivers/:driverId',
      name: 'driver',
      component: () => import('../components/DriverDetail.vue'),
      props: true,
    },
    {
      path: '/market',
      name: 'market',
      component: () => import('../components/Market.vue'),
    },
    {
      path: '/login',
      name: 'login',
      component: () => import('../components/LoginForm.vue'),
    },
    {
      path: '/register',
      name: 'register',
      component: () => import('../components/RegisterForm.vue'),
    },
  ],
});

export default router;
