import { defaultClient } from "../../services/api";
import { EntityGraphApi } from "../../services/entityGraphApi";
import type { ActivityTimelineQuery } from "../entity-graph/types";
import type { UserFriendlyActivityResponse } from "./types";
import {
  normalizeUserFriendlyActivity,
  toUserFriendlyActivity,
} from "./userActivityModel";

function queryString(params?: object) {
  const query = new URLSearchParams();
  for (const [key, value] of Object.entries(params ?? {})) {
    if (value !== undefined && value !== null && value !== "") {
      query.set(key, String(value));
    }
  }
  const suffix = query.toString();
  return suffix ? `?${suffix}` : "";
}

export const UserActivityApi = {
  async list(
    params?: ActivityTimelineQuery,
  ): Promise<UserFriendlyActivityResponse> {
    try {
      const response =
        await defaultClient.fetchApi<UserFriendlyActivityResponse>(
          `/user-friendly-activity${queryString(params)}`,
        );
      return {
        ...response,
        items: (response.items ?? []).map(normalizeUserFriendlyActivity),
      };
    } catch {
      const timeline = await EntityGraphApi.getActivity(params);
      return {
        schema_version: "user-friendly-activity-list.v1",
        tenant_id: timeline.tenant_id,
        generated_at: timeline.generated_at,
        source: "dashboard-timeline-fallback",
        items: timeline.items
          .map(toUserFriendlyActivity)
          .map(normalizeUserFriendlyActivity),
        next_cursor: timeline.next_cursor,
      };
    }
  },
  async clearLocalHistory(): Promise<{
    status: string;
    scope: string;
    observation_events: number;
    decision_logs: number;
    decisions: number;
  }> {
    return defaultClient.fetchApi("/user-friendly-activity", {
      method: "DELETE",
    });
  },
};
