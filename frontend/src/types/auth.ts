/** User profile as returned by GET /api/v1/me. */
export interface UserProfile {
  id: string;
  email: string;
  name: string;
  preferred_username: string;
  first_seen_at: string;
  last_login_at: string;
}

/** Project summary as returned by GET /api/v1/projects. */
export interface ProjectSummary {
  id: string;
  name: string;
  updated_at: string;
  has_result: boolean;
}

/** Full project response as returned by GET /api/v1/projects/:id. */
export interface ProjectResponse {
  id: string;
  name: string;
  project_data: unknown;
  result_data: unknown | null;
  created_at: string;
  updated_at: string;
}
