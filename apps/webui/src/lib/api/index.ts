import * as configApi from "./config";
import * as jobsApi from "./jobs";
import * as packagesApi from "./packages";
import * as syncApi from "./sync";
import * as usersApi from "./users";

export { ApiClientError, API_BASE } from "./client";

export const api = {
  ...configApi,
  ...jobsApi,
  ...packagesApi,
  ...syncApi,
  ...usersApi,
};

export default api;
