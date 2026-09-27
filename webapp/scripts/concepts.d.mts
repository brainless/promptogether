export interface LoadedConcept {
  slug: string;
  name: string;
  definition: string;
  url: string | null;
  aliases: string[];
  posts: { slug: string; title: string; date: string }[];
}

export function loadConcepts(dbPath: string, posts: readonly { slug: string; title: string; date: string }[]): LoadedConcept[];
