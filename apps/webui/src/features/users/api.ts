import api from "../../lib/api";

export const usersApi = {
  changeUserPassword: (...args: Parameters<typeof api.changeUserPassword>) =>
    api.changeUserPassword(...args),
  createUser: (...args: Parameters<typeof api.createUser>) =>
    api.createUser(...args),
  deleteUser: (...args: Parameters<typeof api.deleteUser>) =>
    api.deleteUser(...args),
  getSession: (...args: Parameters<typeof api.getSession>) =>
    api.getSession(...args),
  listUsers: (...args: Parameters<typeof api.listUsers>) => api.listUsers(...args),
  updateUser: (...args: Parameters<typeof api.updateUser>) =>
    api.updateUser(...args),
};
