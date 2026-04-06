import { ApiClientError } from "./client";
import { SyncApiClient } from "./sync";

export class ApiClient extends SyncApiClient {}

export { ApiClientError };

export const api = new ApiClient();
export default api;
