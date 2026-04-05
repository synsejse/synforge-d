import { ApiClientError } from "./client";
import { UserApiClient } from "./users";

export class ApiClient extends UserApiClient {}

export { ApiClientError };

export const api = new ApiClient();
export default api;
