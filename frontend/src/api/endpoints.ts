// Single source of truth for API URL paths.
export const endpoints = {
  login: import.meta.env.VITE_OAUTH_LOGIN_PATH,
  logout: import.meta.env.VITE_OAUTH_LOGOUT_PATH,
  userInfo: '/user/info',
  spin: '/spin',
  loan: (amount: number) => `/loan/${amount}`,
  adminUserList: '/admin/userlist',
  adminUpdateUser: '/admin/update/user',
  adminDeleteUser: (id: number) => `/admin/delete/user/${id}`,
} as const