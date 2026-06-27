import helpTopics from "../data/contextual-help.compact.json";

export interface ContextualHelpTopic {
  id: string;
  title: string;
  summary: string;
  guidance: string[];
  appliesTo: string[];
  sourceDoc: string;
  sourceAnchor?: string;
  relatedTopicIds?: string[];
}

const TOPICS = helpTopics as ContextualHelpTopic[];

export function getHelpTopic(id?: string) {
  if (!id) return undefined;
  return TOPICS.find((topic) => topic.id === id);
}

export function getHelpTopics() {
  return TOPICS;
}
